use std::collections::BTreeMap;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

#[derive(Clone, Debug)]
pub struct CapturedRequest {
    pub method: String,
    pub target: String,
    pub headers: BTreeMap<String, String>,
    pub body: String,
}

pub struct ServiceResponse {
    pub status: u16,
    pub body: String,
}

impl ServiceResponse {
    pub fn json(status: u16, value: serde_json::Value) -> Self {
        Self {
            status,
            body: value.to_string(),
        }
    }

    pub fn truncated_json(body: &str) -> Self {
        Self {
            status: 299,
            body: body.to_owned(),
        }
    }
}

type Handler = Box<dyn Fn(CapturedRequest) -> ServiceResponse + Send>;

pub struct SnoServiceServer {
    base_url: String,
    requests: Arc<Mutex<Vec<CapturedRequest>>>,
    thread: Option<JoinHandle<()>>,
}

impl SnoServiceServer {
    pub fn start(handlers: Vec<Handler>) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind loopback service");
        listener
            .set_nonblocking(true)
            .expect("set loopback listener nonblocking");
        let address = listener.local_addr().expect("read loopback address");
        let requests = Arc::new(Mutex::new(Vec::new()));
        let captured = Arc::clone(&requests);
        let thread = thread::spawn(move || {
            let deadline = Instant::now() + Duration::from_secs(10);
            for handler in handlers {
                let mut stream = loop {
                    match listener.accept() {
                        Ok((stream, _)) => break stream,
                        Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                            assert!(Instant::now() < deadline, "timed out waiting for request");
                            thread::sleep(Duration::from_millis(5));
                        }
                        Err(error) => panic!("accept request: {error}"),
                    }
                };
                stream
                    .set_nonblocking(false)
                    .expect("set request stream blocking");
                stream
                    .set_read_timeout(Some(Duration::from_secs(2)))
                    .expect("set request timeout");
                let request = read_request(&mut stream);
                captured.lock().expect("capture lock").push(request.clone());
                write_response(&mut stream, handler(request));
            }
        });
        Self {
            base_url: format!("http://{address}"),
            requests,
            thread: Some(thread),
        }
    }

    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    pub fn finish(mut self) -> Vec<CapturedRequest> {
        if let Some(thread) = self.thread.take() {
            thread.join().expect("service thread");
        }
        self.requests.lock().expect("capture lock").clone()
    }
}

impl Drop for SnoServiceServer {
    fn drop(&mut self) {
        let Some(thread) = self.thread.take() else {
            return;
        };
        if thread::panicking() {
            if thread.is_finished() {
                let _ = thread.join();
            }
            return;
        }
        match thread.join() {
            Ok(()) => panic!("SnoServiceServer dropped without finish()"),
            Err(payload) => std::panic::resume_unwind(payload),
        }
    }
}

fn read_request(stream: &mut TcpStream) -> CapturedRequest {
    let mut bytes = Vec::new();
    let mut buffer = [0_u8; 4096];
    let header_end = loop {
        let count = stream.read(&mut buffer).expect("read request");
        assert!(count > 0, "connection closed before headers");
        bytes.extend_from_slice(&buffer[..count]);
        if let Some(position) = bytes.windows(4).position(|window| window == b"\r\n\r\n") {
            break position + 4;
        }
    };
    let headers_text = String::from_utf8(bytes[..header_end].to_vec()).expect("UTF-8 headers");
    let mut lines = headers_text.split("\r\n");
    let request_line = lines.next().expect("request line");
    let mut request_parts = request_line.split_whitespace();
    let method = request_parts.next().expect("method").to_owned();
    let target = request_parts.next().expect("target").to_owned();
    let mut headers = BTreeMap::new();
    for line in lines.filter(|line| !line.is_empty()) {
        let (name, value) = line.split_once(':').expect("valid header");
        headers.insert(name.to_ascii_lowercase(), value.trim().to_owned());
    }
    let content_length = headers
        .get("content-length")
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(0);
    while bytes.len() - header_end < content_length {
        let count = stream.read(&mut buffer).expect("read request body");
        assert!(count > 0, "connection closed before body");
        bytes.extend_from_slice(&buffer[..count]);
    }
    CapturedRequest {
        method,
        target,
        headers,
        body: String::from_utf8(bytes[header_end..header_end + content_length].to_vec())
            .expect("UTF-8 body"),
    }
}

fn write_response(stream: &mut TcpStream, response: ServiceResponse) {
    let wire_status = if response.status == 299 {
        200
    } else {
        response.status
    };
    let reason = match wire_status {
        200 => "OK",
        400 => "Bad Request",
        404 => "Not Found",
        409 => "Conflict",
        _ => "Response",
    };
    let content_length = if response.status == 299 {
        response.body.len() + 200
    } else {
        response.body.len()
    };
    write!(
        stream,
        "HTTP/1.1 {} {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        wire_status,
        reason,
        content_length,
        response.body,
    )
    .expect("write response");
    stream.flush().expect("flush response");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dropped_server_propagates_handler_panic() {
        let result = std::panic::catch_unwind(|| {
            let server = SnoServiceServer::start(vec![Box::new(|_| {
                panic!("handler assertion failed");
            })]);
            let address = server.base_url().strip_prefix("http://").unwrap();
            let mut stream = TcpStream::connect(address).unwrap();
            stream
                .write_all(b"GET / HTTP/1.1\r\nHost: localhost\r\n\r\n")
                .unwrap();
            stream.flush().unwrap();
            drop(server);
        });

        assert!(result.is_err());
    }
}
