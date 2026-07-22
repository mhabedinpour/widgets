use crate::http::{
    GlobalHttpClient, Http, HttpError, HttpRequestData, HttpResponse, ParsedUrl, RequestId,
    parse_url,
};
use crate::widget::WidgetId;
use crate::{use_psram_heap, use_sram_heap};
use alloc::boxed::Box;
use alloc::string::{String, ToString};
use alloc::sync::Arc;
use alloc::vec::Vec;
use core::cell::RefCell;
use core::fmt::Write as FmtWrite;
use critical_section::Mutex;
use embassy_executor::Spawner;
use embassy_net::Stack;
use embassy_net::dns::DnsQueryType;
use embassy_net::tcp::TcpSocket;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;
use embassy_time::Duration;
use embedded_io_async::{Read, Write};
use embedded_tls::{Aes256GcmSha384, TlsConfig, TlsConnection, TlsContext, UnsecureProvider};
use esp_hal::rng::Rng;
// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

const CHANNEL_DEPTH: usize = 4;

/// TCP socket buffer size (receive + transmit) per widget task.
const TCP_BUF_SIZE: usize = 4096;

/// TLS receive buffer. 16 KiB covers the maximum TLS record size and is
/// large enough to hold a server certificate chain during the handshake.
const TLS_RX_BUF_SIZE: usize = 16384;

/// TLS transmit buffer. Outbound records (HTTP requests) are small.
const TLS_TX_BUF_SIZE: usize = 4096;

/// Chunk size used when draining the response stream.
const READ_CHUNK: usize = 256;

// ---------------------------------------------------------------------------
// Internal types
// ---------------------------------------------------------------------------

/// Bundles a caller-assigned sequential ID with the request data so the
/// task can correlate the response without the caller having to supply the id.
struct TaggedRequest {
    id: RequestId,
    data: HttpRequestData,
}

type HttpChannel = Channel<CriticalSectionRawMutex, Option<TaggedRequest>, CHANNEL_DEPTH>;

/// Shared completed-response buffer.
///
/// Wraps `critical_section::Mutex<RefCell<Vec<_>>>` in a newtype so we can
/// implement `Send`.  `critical_section::Mutex` is already unconditionally
/// `Sync` (all access disables interrupts), but the inner `RefCell` makes the
/// whole type `!Send`.  On the single-core ESP32-S3 every access is already
/// serialised by the critical section, so the `Send` impl is sound.
struct CompletedQueue(Mutex<RefCell<Vec<(WidgetId, HttpResponse)>>>);

// SAFETY: All mutations go through `critical_section::with`, which disables
// interrupts and guarantees exclusive access on any single-core target.
unsafe impl Send for CompletedQueue {}

impl CompletedQueue {
    fn new() -> Self {
        Self(Mutex::new(RefCell::new(Vec::new())))
    }

    fn push(&self, item: (WidgetId, HttpResponse)) {
        critical_section::with(|cs| self.0.borrow(cs).borrow_mut().push(item));
    }

    fn drain(&self) -> Vec<(WidgetId, HttpResponse)> {
        critical_section::with(|cs| self.0.borrow(cs).borrow_mut().drain(..).collect())
    }
}

/// Wraps `esp_hal::rng::Rng` and asserts the `CryptoRng` contract.
///
/// On ESP32-S3 the hardware TRNG is seeded from RF/thermal noise and passes
/// NIST SP 800-90B tests when the radio is active — which it always is here
/// because Wi-Fi must be up before any HTTP request is dispatched.
struct EspCryptoRng(Rng);

impl rand_core::RngCore for EspCryptoRng {
    fn next_u32(&mut self) -> u32 {
        self.0.random()
    }

    fn next_u64(&mut self) -> u64 {
        ((self.0.random() as u64) << 32) | (self.0.random() as u64)
    }

    fn fill_bytes(&mut self, dest: &mut [u8]) {
        for chunk in dest.chunks_mut(4) {
            let bytes = self.0.random().to_le_bytes();
            chunk.copy_from_slice(&bytes[..chunk.len()]);
        }
    }

    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), rand_core::Error> {
        self.fill_bytes(dest);
        Ok(())
    }
}

// rand_core 0.6 CryptoRng is a safe marker trait.
// The ESP32-S3 TRNG passes NIST SP 800-90B tests when the RF subsystem is
// active (Wi-Fi is always up before any HTTP request is dispatched).
impl rand_core::CryptoRng for EspCryptoRng {}

// ---------------------------------------------------------------------------
// HttpService — global HTTP client owned by the widget manager
// ---------------------------------------------------------------------------

pub struct HttpService {
    spawner: Spawner,
    stack: Stack<'static>,
    completed: Arc<CompletedQueue>,
}

impl HttpService {
    pub fn new(spawner: Spawner, stack: Stack<'static>) -> Self {
        Self {
            spawner,
            stack,
            completed: Arc::new(CompletedQueue::new()),
        }
    }
}

impl GlobalHttpClient for HttpService {
    fn poll(&mut self) -> Vec<(WidgetId, HttpResponse)> {
        self.completed.drain()
    }

    fn scoped(&self, widget_id: WidgetId) -> Box<dyn Http> {
        let channel = Arc::new(Channel::new());
        self.spawner.spawn(
            widget_http_task(
                self.stack,
                widget_id,
                Arc::clone(&channel),
                Arc::clone(&self.completed),
            )
            .unwrap(),
        );
        Box::new(WidgetHttpClient {
            channel,
            completed: Arc::clone(&self.completed),
            widget_id,
            next_id: 0,
        })
    }
}

// ---------------------------------------------------------------------------
// WidgetHttpClient — scoped client handle handed to each widget
// ---------------------------------------------------------------------------

pub struct WidgetHttpClient {
    channel: Arc<HttpChannel>,
    completed: Arc<CompletedQueue>,
    widget_id: WidgetId,
    next_id: u32,
}

impl Http for WidgetHttpClient {
    fn send_request(&mut self, data: HttpRequestData) -> RequestId {
        self.next_id = self.next_id.wrapping_add(1);
        let id = RequestId(self.next_id);
        if self
            .channel
            .try_send(Some(TaggedRequest { id, data }))
            .is_err()
        {
            log::warn!("HTTP channel full, returning error for request {}", id.0);
            self.completed.push((
                self.widget_id,
                HttpResponse {
                    request_id: id,
                    headers: None,
                    body: None,
                    error: Some(HttpError::ChannelFull),
                },
            ));
        }
        id
    }
}

impl Drop for WidgetHttpClient {
    fn drop(&mut self) {
        // Drain pending requests — no one will ever receive their responses.
        // This guarantees there is always room for the stop sentinel below.
        while self.channel.try_receive().is_ok() {}
        // Send None as the stop sentinel.  The task exits its receive loop on
        // seeing it and then frees its PSRAM buffers.
        let _ = self.channel.try_send(None);
    }
}

// ---------------------------------------------------------------------------
// Shared I/O helpers (work over any embedded-io-async stream)
// ---------------------------------------------------------------------------

/// Serialises and writes an HTTP/1.0 request to `stream`.
async fn write_http_request<S: Write>(
    stream: &mut S,
    parsed: &ParsedUrl<'_>,
    req: &HttpRequestData,
) -> Result<(), HttpError> {
    let mut buf = String::new();

    buf.push_str(req.method.as_str());
    buf.push(' ');
    buf.push_str(parsed.path);
    buf.push_str(" HTTP/1.0\r\nHost: ");
    buf.push_str(parsed.host);
    buf.push_str("\r\nConnection: close\r\n");

    for (name, value) in &req.headers {
        buf.push_str(name.as_str());
        buf.push_str(": ");
        buf.push_str(value.as_str());
        buf.push_str("\r\n");
    }

    if !req.body.is_empty() {
        let has_content_length = req
            .headers
            .iter()
            .any(|(k, _)| k.eq_ignore_ascii_case("content-length"));
        if !has_content_length {
            write!(buf, "Content-Length: {}\r\n", req.body.len()).ok();
        }
        buf.push_str("\r\n");
        buf.push_str(req.body.as_str());
    } else {
        buf.push_str("\r\n");
    }

    stream
        .write_all(buf.as_bytes())
        .await
        .map_err(|_| HttpError::WriteFailed)?;
    stream.flush().await.map_err(|_| HttpError::FlushFailed)
}

/// Reads the full response until EOF (HTTP/1.0 semantics).
///
/// Returns parsed response headers as `(name, value)` pairs — the HTTP
/// status line is excluded — plus the raw body string.
async fn read_http_response<S: Read>(stream: &mut S) -> (Vec<(String, String)>, String) {
    let mut raw: Vec<u8> = Vec::new();
    let mut chunk = [0u8; READ_CHUNK];
    loop {
        match stream.read(&mut chunk).await {
            Ok(0) | Err(_) => break,
            Ok(n) => raw.extend_from_slice(&chunk[..n]),
        }
    }

    let split = raw
        .windows(4)
        .position(|w| w == b"\r\n\r\n")
        .map(|i| i + 4)
        .unwrap_or(0);

    // Parse header section: skip the status line, then split each
    // "Name: value" line into a (String, String) tuple.
    let header_section = core::str::from_utf8(&raw[..split]).unwrap_or("");
    let mut headers: Vec<(String, String)> = Vec::new();
    for line in header_section.lines().skip(1) {
        if let Some(colon) = line.find(':') {
            let name = line[..colon].trim().to_string();
            let value = line[colon + 1..].trim().to_string();
            headers.push((name, value));
        }
    }

    let body = String::from_utf8_lossy(&raw[split..]).into_owned();
    (headers, body)
}

// ---------------------------------------------------------------------------
// Protocol-specific execution
// ---------------------------------------------------------------------------

async fn execute_http_request<'b>(
    stack: Stack<'static>,
    tcp_rx: &'b mut [u8],
    tcp_tx: &'b mut [u8],
    parsed: &ParsedUrl<'_>,
    req: &HttpRequestData,
    addr: embassy_net::IpAddress,
) -> Result<(Vec<(String, String)>, String), HttpError> {
    let mut socket = TcpSocket::new(stack, tcp_rx, tcp_tx);
    socket.set_timeout(Some(Duration::from_secs(10)));
    socket
        .connect(embassy_net::IpEndpoint::new(addr, parsed.port))
        .await
        .map_err(|_| HttpError::ConnectFailed)?;

    write_http_request(&mut socket, parsed, req).await?;
    let result = read_http_response(&mut socket).await;
    socket.abort();
    Ok(result)
}

/// Wraps the TCP socket in a TLS 1.3 layer and performs the request.
///
/// **Certificate verification is skipped** (`NoVerify`) — appropriate for
/// IoT display widgets calling trusted first-party APIs. To enable chain
/// validation, replace `NoVerify` with a `CertVerificationProvider` and
/// embed your CA certificate in the firmware.
async fn execute_https_request<'b>(
    stack: Stack<'static>,
    tcp_rx: &'b mut [u8],
    tcp_tx: &'b mut [u8],
    tls_rx: &'b mut [u8],
    tls_tx: &'b mut [u8],
    parsed: &ParsedUrl<'_>,
    req: &HttpRequestData,
    addr: embassy_net::IpAddress,
) -> Result<(Vec<(String, String)>, String), HttpError> {
    let mut socket = TcpSocket::new(stack, tcp_rx, tcp_tx);
    socket.set_timeout(Some(Duration::from_secs(10)));
    socket
        .connect(embassy_net::IpEndpoint::new(addr, parsed.port))
        .await
        .map_err(|_| HttpError::ConnectFailed)?;

    let config = TlsConfig::new()
        .with_server_name(parsed.host)
        .enable_rsa_signatures();
    let mut tls: TlsConnection<'_, _, Aes256GcmSha384> = TlsConnection::new(socket, tls_rx, tls_tx);

    let rng = EspCryptoRng(Rng::new());
    tls.open(TlsContext::new(
        &config,
        UnsecureProvider::new::<Aes256GcmSha384>(rng),
    ))
    .await
    .map_err(|_| HttpError::TlsHandshakeFailed)?;

    write_http_request(&mut tls, parsed, req).await?;
    let result = read_http_response(&mut tls).await;

    // Best-effort graceful close; peer may have already half-closed.
    let _ = tls.close().await;
    Ok(result)
}

/// DNS-resolves the URL's host, then dispatches to the HTTP or HTTPS path.
async fn execute_request(
    stack: Stack<'static>,
    tcp_rx: &mut [u8],
    tcp_tx: &mut [u8],
    tls_rx: &mut [u8],
    tls_tx: &mut [u8],
    req: &HttpRequestData,
) -> Result<(Vec<(String, String)>, String), HttpError> {
    let parsed = parse_url(&req.url)?;

    let addrs = stack
        .dns_query(parsed.host, DnsQueryType::A)
        .await
        .map_err(|_| HttpError::DnsQueryFailed)?;
    let addr = addrs.first().copied().ok_or(HttpError::NoDnsResult)?;

    if parsed.is_https {
        execute_https_request(stack, tcp_rx, tcp_tx, tls_rx, tls_tx, &parsed, req, addr).await
    } else {
        execute_http_request(stack, tcp_rx, tcp_tx, &parsed, req, addr).await
    }
}

// ---------------------------------------------------------------------------
// Per-widget embassy task
// ---------------------------------------------------------------------------

#[embassy_executor::task(pool_size = 8)]
async fn widget_http_task(
    stack: Stack<'static>,
    widget_id: WidgetId,
    channel: Arc<HttpChannel>,
    completed: Arc<CompletedQueue>,
) {
    use_psram_heap();
    let mut tcp_rx = Box::new([0u8; TCP_BUF_SIZE]);
    let mut tcp_tx = Box::new([0u8; TCP_BUF_SIZE]);
    let mut tls_rx = Box::new([0u8; TLS_RX_BUF_SIZE]);
    let mut tls_tx = Box::new([0u8; TLS_TX_BUF_SIZE]);
    use_sram_heap();

    stack.wait_config_up().await;

    loop {
        let Some(TaggedRequest { id, data: req }) = channel.receive().await else {
            // None is the stop sentinel sent by WidgetHttpClient::drop.
            break;
        };

        let (headers, body, error) = match execute_request(
            stack,
            tcp_rx.as_mut_slice(),
            tcp_tx.as_mut_slice(),
            tls_rx.as_mut_slice(),
            tls_tx.as_mut_slice(),
            &req,
        )
        .await
        {
            Ok((h, b)) => (Some(h), Some(b), None),
            Err(e) => {
                log::error!("HTTP[S] request {} to {} failed: {}", id.0, req.url, e);
                (None, None, Some(e))
            }
        };

        completed.push((
            widget_id,
            HttpResponse {
                request_id: id,
                headers,
                body,
                error,
            },
        ));
    }
}
