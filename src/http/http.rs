use crate::http::{GlobalHttpClient, Http, HttpRequestData, HttpResponse};
use crate::widget::{WidgetId};
use alloc::boxed::Box;
use alloc::string::String;
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
use embedded_io_async::{Write};

const CHANNEL_DEPTH: usize = 4;

type HttpChannel = Channel<CriticalSectionRawMutex, HttpRequestData, CHANNEL_DEPTH>;
type CompletedQueue = Mutex<RefCell<Vec<(WidgetId, HttpResponse)>>>;

pub struct HttpService {
    spawner: Spawner,
    stack: Stack<'static>,
    completed: &'static CompletedQueue,
}

impl HttpService {
    pub fn new(spawner: Spawner, stack: Stack<'static>) -> Self {
        let completed: &'static CompletedQueue =
            Box::leak(Box::new(Mutex::new(RefCell::new(Vec::new()))));
        Self {
            spawner,
            stack,
            completed,
        }
    }
}

impl GlobalHttpClient for HttpService {
    fn poll(&mut self) -> Vec<(WidgetId, HttpResponse)> {
        critical_section::with(|cs| self.completed.borrow(cs).borrow_mut().drain(..).collect())
    }

    fn scoped(&self, widget_id: WidgetId) -> Box<dyn Http> {
        let channel: &'static HttpChannel = Box::leak(Box::new(Channel::new()));
        self.spawner
            .spawn(widget_http_task(
                self.stack,
                widget_id,
                channel,
                self.completed,
            ).unwrap());
        Box::new(WidgetHttpClient { channel })
    }
}

pub struct WidgetHttpClient {
    channel: &'static HttpChannel,
}

impl Http for WidgetHttpClient {
    fn send_request(&mut self, req: HttpRequestData) {
        let id = req.id;
        if self.channel.try_send(req).is_err() {
            log::warn!("HTTP request queue full, dropping request {}", id);
        }
    }
}

fn parse_url(url: &str) -> Result<(&str, u16, &str), &'static str> {
    let (is_https, rest) = if let Some(r) = url.strip_prefix("https://") {
        (true, r)
    } else if let Some(r) = url.strip_prefix("http://") {
        (false, r)
    } else {
        return Err("unsupported scheme");
    };

    let default_port: u16 = if is_https { 443 } else { 80 };

    let (host_port, path) = match rest.find('/') {
        Some(i) => (&rest[..i], &rest[i..]),
        None => (rest, "/"),
    };

    let (host, port) = match host_port.rfind(':') {
        Some(i) => {
            let p = host_port[i + 1..]
                .parse::<u16>()
                .map_err(|_| "invalid port")?;
            (&host_port[..i], p)
        }
        None => (host_port, default_port),
    };

    Ok((host, port, path))
}

async fn execute_request(
    stack: Stack<'static>,
    rx_buf: &mut [u8],
    tx_buf: &mut [u8],
    req: &HttpRequestData,
) -> Result<(String, String), &'static str> {
    let (host, port, path) = parse_url(&req.url)?;

    let addrs = stack
        .dns_query(host, DnsQueryType::A)
        .await
        .map_err(|_| "dns query failed")?;
    let addr = addrs.first().copied().ok_or("no dns result")?;

    let mut socket = TcpSocket::new(stack, rx_buf, tx_buf);
    socket.set_timeout(Some(Duration::from_secs(10)));

    socket
        .connect(embassy_net::IpEndpoint::new(addr, port))
        .await
        .map_err(|_| "connect failed")?;

    // Build HTTP/1.0 request (connection closes after response, no chunked encoding)
    let mut request = String::new();
    request.push_str(req.method.as_str());
    request.push(' ');
    request.push_str(path);
    request.push_str(" HTTP/1.0\r\nHost: ");
    request.push_str(host);
    request.push_str("\r\nConnection: close\r\n");

    if !req.headers.is_empty() {
        request.push_str(req.headers.as_str());
        if !req.headers.ends_with("\r\n") {
            request.push_str("\r\n");
        }
    }

    if !req.body.is_empty() {
        write!(request, "Content-Length: {}\r\n", req.body.len()).ok();
    }

    request.push_str("\r\n");

    if !req.body.is_empty() {
        request.push_str(req.body.as_str());
    }

    socket
        .write_all(request.as_bytes())
        .await
        .map_err(|_| "write failed")?;
    socket.flush().await.map_err(|_| "flush failed")?;

    // Read until connection closes (HTTP/1.0)
    let mut response: Vec<u8> = Vec::new();
    let mut chunk = [0u8; 256];
    loop {
        match socket.read(&mut chunk).await {
            Ok(0) | Err(_) => break,
            Ok(n) => response.extend_from_slice(&chunk[..n]),
        }
    }
    socket.abort();

    // Skip headers — find \r\n\r\n separator
    let body_start = response
        .windows(4)
        .position(|w| w == b"\r\n\r\n")
        .map(|i| i + 4)
        .unwrap_or(0);

    Ok((String::from_utf8_lossy(&response[..body_start]).into_owned(), String::from_utf8_lossy(&response[body_start..]).into_owned()))
}

#[embassy_executor::task]
async fn widget_http_task(
    stack: Stack<'static>,
    widget_id: WidgetId,
    channel: &'static HttpChannel,
    completed: &'static CompletedQueue,
) {
    // Each task owns its own heap-allocated socket buffers.
    let rx_buf: &'static mut [u8] = Box::leak(Box::new([0u8; 4096]));
    let tx_buf: &'static mut [u8] = Box::leak(Box::new([0u8; 4096]));

    stack.wait_config_up().await;

    loop {
        let req = channel.receive().await;

        match execute_request(stack, rx_buf, tx_buf, &req).await {
            Ok((headers, body)) => {
                critical_section::with(|cs| {
                    completed.borrow(cs).borrow_mut().push((
                        widget_id,
                        HttpResponse {
                            request_id: req.id,
                            headers: Some(headers),
                            body: Some(body),
                        },
                    ));
                });
            }
            Err(e) => {
                critical_section::with(|cs| {
                    completed.borrow(cs).borrow_mut().push((
                        widget_id,
                        HttpResponse {
                            request_id: req.id,
                            headers: None,
                            body: None,
                        },
                    ));
                });
                log::error!("HTTP request to {} failed: {}", req.url, e);
            }
        }
    }
}
