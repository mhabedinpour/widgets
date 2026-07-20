use core::cell::Cell;
use critical_section::Mutex;
use embassy_executor::Spawner;
use embassy_net::udp::{PacketMetadata, UdpSocket};
use embassy_net::{IpAddress, IpEndpoint, Ipv4Address};
use embassy_time::{Duration, Instant, Timer, with_timeout};
use static_cell::StaticCell;

const SNTP_TIME_OFFSET: u32 = 2_208_988_800;
const SNTP_PACKET_SIZE: usize = 48;

static TIME_OFFSET: Mutex<Cell<i64>> = Mutex::new(Cell::new(0));

fn update_time(unix_time: i64) {
    let now_secs = Instant::now().as_secs() as i64;
    let offset = unix_time - now_secs;
    critical_section::with(|cs| {
        TIME_OFFSET.borrow(cs).set(offset);
    });
}

/// Retrieve the latest synced Unix timestamp computed against the monotonic clock.
/// Returns None if the time has not been synced yet.
pub fn get_unix_timestamp() -> Option<i64> {
    critical_section::with(|cs| {
        let offset = TIME_OFFSET.borrow(cs).get();
        if offset == 0 {
            None
        } else {
            let now_secs = Instant::now().as_secs() as i64;
            Some(offset + now_secs)
        }
    })
}

#[derive(Debug, Clone, Copy)]
pub struct SntpTimestamp {
    pub secs: u32,
    pub frac: u32,
}

pub struct SntpClient<'d> {
    socket: UdpSocket<'d>,
    server_addr: IpAddress,
    server_port: u16,
}

impl<'d> SntpClient<'d> {
    /// Create a new SntpClient with default server (time.google.com / 216.239.35.0:123).
    pub fn new(stack: embassy_net::Stack<'static>) -> Self {
        let default_addr = IpAddress::Ipv4(Ipv4Address::new(216, 239, 35, 0));
        Self::with_server(stack, default_addr, 123)
    }

    /// Create a new SntpClient with a custom server address and port.
    pub fn with_server(
        stack: embassy_net::Stack<'static>,
        server_addr: IpAddress,
        server_port: u16,
    ) -> Self {
        static RX_META: StaticCell<[PacketMetadata; 2]> = StaticCell::new();
        static RX_BUFFER: StaticCell<[u8; 1024]> = StaticCell::new();
        static TX_META: StaticCell<[PacketMetadata; 2]> = StaticCell::new();
        static TX_BUFFER: StaticCell<[u8; 1024]> = StaticCell::new();

        let rx_meta = RX_META.init([PacketMetadata::EMPTY; 2]);
        let rx_buffer = RX_BUFFER.init([0; 1024]);
        let tx_meta = TX_META.init([PacketMetadata::EMPTY; 2]);
        let tx_buffer = TX_BUFFER.init([0; 1024]);

        let mut socket = UdpSocket::new(stack, rx_meta, rx_buffer, tx_meta, tx_buffer);

        if let Err(e) = socket.bind(12345) {
            log::error!("SntpClient: failed to bind socket: {:?}", e);
        }

        Self {
            socket,
            server_addr,
            server_port,
        }
    }

    /// Request raw NTP timestamp from a specific address.
    pub async fn get_raw_time(
        &mut self,
        addr: IpAddress,
        port: u16,
    ) -> Result<SntpTimestamp, &'static str> {
        let mut packet = [0u8; SNTP_PACKET_SIZE];
        // LI (2 bit) - 3 (not in sync), VN (3 bit) - 4 (version), mode (3 bit) - 3 (client)
        packet[0] = (3 << 6) | (4 << 3) | 3;

        let endpoint = IpEndpoint::new(addr, port);
        if self.socket.send_to(&packet, endpoint).await.is_err() {
            return Err("Failed to send SNTP request packet");
        }

        let mut response = [0u8; SNTP_PACKET_SIZE];
        let recv_result = with_timeout(Duration::from_secs(5), self.socket.recv_from(&mut response)).await;
        match recv_result {
            Err(_) => Err("SNTP request timed out"),
            Ok(Err(_)) => Err("Failed to receive SNTP response packet"),
            Ok(Ok((recv, _))) => {
                if recv != SNTP_PACKET_SIZE {
                    return Err("Invalid SNTP packet size received");
                }
                let hdr = response[0];
                let vn = (hdr & 0x38) >> 3;
                if vn != 4 {
                    return Err("Server returned wrong SNTP version");
                }
                let mode = hdr & 0x7;
                if mode != 4 && mode != 5 {
                    return Err("Not a SNTP server reply");
                }

                let secs =
                    u32::from_be_bytes([response[40], response[41], response[42], response[43]]);
                let frac =
                    u32::from_be_bytes([response[44], response[45], response[46], response[47]]);

                Ok(SntpTimestamp { secs, frac })
            }
        }
    }

    /// Request the latest Unix time from the configured NTP server, updating the global system offset.
    pub async fn get_unix_time(&mut self) -> Result<i64, &'static str> {
        let raw_time = self
            .get_raw_time(self.server_addr, self.server_port)
            .await?;
        let unix_time = raw_time.secs.wrapping_sub(SNTP_TIME_OFFSET) as i64;
        update_time(unix_time);
        Ok(unix_time)
    }
}

#[embassy_executor::task]
#[allow(clippy::large_stack_frames)]
async fn ntp_task(stack: embassy_net::Stack<'static>) {
    stack.wait_config_up().await;
    log::info!("NTP task: Network is up, starting NTP sync loop...");

    let mut client = SntpClient::new(stack);

    loop {
        log::info!("NTP task: Syncing time...");
        match client.get_unix_time().await {
            Ok(unix_time) => {
                log::info!("NTP task: Sync successful! Unix timestamp: {}", unix_time);
                Timer::after_secs(3600).await;
            }
            Err(e) => {
                log::error!("NTP task: Sync failed: {}", e);
                Timer::after_secs(10).await;
            }
        }
    }
}

/// Spawn the internal NTP sync task. After the first successful sync, `get_unix_timestamp()`
/// will return the current Unix time derived from the cached offset.
pub fn start(spawner: Spawner, stack: embassy_net::Stack<'static>) {
    spawner.spawn(ntp_task(stack).unwrap());
}
