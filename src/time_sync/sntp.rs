use core::cell::Cell;
use critical_section::Mutex;
use embassy_net::udp::{PacketMetadata, UdpSocket};
use embassy_net::{IpAddress, IpEndpoint, Ipv4Address};
use embassy_time::{Duration, Instant, Timer, with_timeout};

const SNTP_TIME_OFFSET: u32 = 2_208_988_800;
const SNTP_PACKET_SIZE: usize = 48;

pub struct TimeSyncService {
    // Time state — interior mutability so readers can hold `&self` alongside `run()`.
    time_offset: Mutex<Cell<i64>>,
    /// Monotonic ticks at the moment of the last successful sync.
    /// `u64::MAX` is the sentinel for "never synced".
    last_sync_ticks: Mutex<Cell<u64>>,
    /// Unix timestamp (seconds) recorded at the last successful sync.
    /// `0` before the first sync.
    last_sync_unix: Mutex<Cell<i64>>,
    // Server config.
    server_addr: IpAddress,
    server_port: u16,

    stack: embassy_net::Stack<'static>,
}

// SAFETY: `Mutex<Cell<>>` fields handle their own synchronization.
// Buffer fields (UnsafeCell) are accessed exclusively from `run()`, which is invoked
// exactly once by the NTP task, so there is never concurrent mutable access.
unsafe impl Sync for TimeSyncService {}

impl TimeSyncService {
    /// Create a service using the default NTP server (time.google.com / 216.239.35.0:123).
    pub fn new(stack: embassy_net::Stack<'static>) -> Self {
        Self::with_server(
            stack,
            IpAddress::Ipv4(Ipv4Address::new(216, 239, 35, 0)),
            123,
        )
    }

    /// Create a service pointing at a custom NTP server.
    pub fn with_server(
        stack: embassy_net::Stack<'static>,
        server_addr: IpAddress,
        server_port: u16,
    ) -> Self {
        Self {
            time_offset: Mutex::new(Cell::new(0)),
            last_sync_ticks: Mutex::new(Cell::new(u64::MAX)),
            last_sync_unix: Mutex::new(Cell::new(0)),
            server_addr,
            server_port,
            stack,
        }
    }

    fn update_time(&self, unix_time: i64) {
        let now = Instant::now();
        let offset = unix_time - now.as_secs() as i64;
        critical_section::with(|cs| {
            self.time_offset.borrow(cs).set(offset);
            self.last_sync_ticks.borrow(cs).set(now.as_ticks());
            self.last_sync_unix.borrow(cs).set(unix_time);
        });
    }

    /// The monotonic [`Instant`] at which the last NTP sync completed.
    /// Returns `None` if no sync has occurred yet.
    pub fn get_last_sync_instant(&self) -> Option<Instant> {
        critical_section::with(|cs| {
            let ticks = self.last_sync_ticks.borrow(cs).get();
            if ticks == u64::MAX {
                None
            } else {
                Some(Instant::from_ticks(ticks))
            }
        })
    }

    /// The Unix timestamp (seconds) recorded at the last NTP sync.
    /// Returns `None` if no sync has occurred yet.
    pub fn get_last_sync_unix(&self) -> Option<i64> {
        critical_section::with(|cs| {
            let unix = self.last_sync_unix.borrow(cs).get();
            if unix == 0 { None } else { Some(unix) }
        })
    }

    /// Retrieve the latest synced Unix timestamp computed against the monotonic clock.
    /// Returns `None` if the time has not been synced yet.
    pub fn get_unix_timestamp(&self) -> Option<i64> {
        critical_section::with(|cs| {
            let offset = self.time_offset.borrow(cs).get();
            if offset == 0 {
                None
            } else {
                let now_secs = Instant::now().as_secs() as i64;
                Some(offset + now_secs)
            }
        })
    }

    async fn get_raw_time(&self, socket: &mut UdpSocket<'_>) -> Result<(u32, u32), &'static str> {
        let mut packet = [0u8; SNTP_PACKET_SIZE];
        // LI (2 bit) - 3 (not in sync), VN (3 bit) - 4 (version), mode (3 bit) - 3 (client)
        packet[0] = (3 << 6) | (4 << 3) | 3;

        let endpoint = IpEndpoint::new(self.server_addr, self.server_port);
        if socket.send_to(&packet, endpoint).await.is_err() {
            return Err("Failed to send SNTP request packet");
        }

        let mut response = [0u8; SNTP_PACKET_SIZE];
        let recv_result =
            with_timeout(Duration::from_secs(5), socket.recv_from(&mut response)).await;
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
                Ok((secs, frac))
            }
        }
    }

    async fn wait_for_stack(&self) {
        self.stack.wait_config_up().await;
    }

    async fn sync_once(&self) -> Result<i64, &'static str> {
        let mut rx_meta = [PacketMetadata::EMPTY; 2];
        let mut rx_buffer = [0u8; 1024];
        let mut tx_meta = [PacketMetadata::EMPTY; 2];
        let mut tx_buffer = [0u8; 1024];
        let mut socket = UdpSocket::new(
            self.stack,
            &mut rx_meta,
            &mut rx_buffer,
            &mut tx_meta,
            &mut tx_buffer,
        );
        if let Err(e) = socket.bind(12345) {
            log::error!("TimeSyncService: failed to bind socket: {:?}", e);

            return Err("TimeSyncService: failed to bind socket");
        }

        let (secs, _frac) = self.get_raw_time(&mut socket).await?;
        let unix_time = secs.wrapping_sub(SNTP_TIME_OFFSET) as i64;
        self.update_time(unix_time);
        Ok(unix_time)
    }
}

#[embassy_executor::task]
#[allow(clippy::large_stack_frames)]
pub async fn time_sync_task(service: &'static TimeSyncService) {
    service.wait_for_stack().await;

    loop {
        log::info!("NTP task: Syncing time...");
        match service.sync_once().await {
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
