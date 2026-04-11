use std::io::{Read, Write, BufRead, BufReader, Cursor};
use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;
use std::sync::Arc;
use std::thread;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let port = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(3805u16);
    let static_dir = args.get(2).map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("deploy/380z"));

    let nonce: String = (0..16).map(|_| {
        let c = rand::random::<u8>() % 36;
        if c < 10 { (b'0' + c) as char } else { (b'a' + c - 10) as char }
    }).collect();

    println!("RM 380Z Server");
    println!("  http://localhost:{}/", port);
    println!("  Static: {}", static_dir.display());
    println!("  Nonce: {}", nonce);
    println!();

    let listener = TcpListener::bind(format!("0.0.0.0:{}", port)).expect("Failed to bind");
    let nonce = Arc::new(nonce);
    let static_dir = Arc::new(static_dir);

    for stream in listener.incoming().flatten() {
        let nonce = nonce.clone();
        let static_dir = static_dir.clone();
        thread::spawn(move || handle_connection(stream, &static_dir, &nonce));
    }
}

fn handle_connection(stream: TcpStream, static_dir: &PathBuf, nonce: &str) {
    // Peek at the first line to get the path without consuming for WebSocket
    let mut reader = BufReader::new(stream);
    let mut first_line = String::new();
    if reader.read_line(&mut first_line).is_err() { return; }

    let path = first_line.split_whitespace().nth(1).unwrap_or("/").to_string();

    if path == "/ws" {
        // WebSocket: read remaining headers then do handshake
        let mut headers = first_line.clone();
        loop {
            let mut line = String::new();
            if reader.read_line(&mut line).is_err() || line == "\r\n" || line == "\n" {
                headers.push_str(&line);
                break;
            }
            headers.push_str(&line);
        }
        // Reconstruct stream with buffered data for tungstenite
        let stream = reader.into_inner();
        // Create a custom Read that prepends the already-read headers
        let cursor = Cursor::new(headers.into_bytes());
        let combined = CombinedReader { first: cursor, second: stream };
        handle_websocket(combined, nonce);
        return;
    }

    // Regular HTTP: read remaining headers (we don't need them)
    loop {
        let mut line = String::new();
        if reader.read_line(&mut line).is_err() || line == "\r\n" || line == "\n" { break; }
    }
    let mut stream = reader.into_inner();

    if path == "/" || path == "/index.html" {
        serve_index(&mut stream, static_dir, nonce);
    } else {
        serve_file(&mut stream, static_dir, &path);
    }
}

/// A reader that reads from first, then second.
struct CombinedReader<R1: Read, R2: Read + Write> {
    first: R1,
    second: R2,
}

impl<R1: Read, R2: Read + Write> Read for CombinedReader<R1, R2> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let n = self.first.read(buf)?;
        if n > 0 { return Ok(n); }
        self.second.read(buf)
    }
}

impl<R1: Read, R2: Read + Write> Write for CombinedReader<R1, R2> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> { self.second.write(buf) }
    fn flush(&mut self) -> std::io::Result<()> { self.second.flush() }
}

fn serve_index(stream: &mut TcpStream, static_dir: &PathBuf, nonce: &str) {
    let index_path = static_dir.join("index.html");
    let Ok(mut html) = std::fs::read_to_string(&index_path) else { send_404(stream); return; };

    let ws_config = format!(
        "<script>window.RM380Z_NONCE='{}';window.RM380Z_WS='ws://'+location.host+'/ws';</script>",
        nonce
    );
    html = html.replace("</head>", &format!("{}</head>", ws_config));

    let resp = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        html.len(), html
    );
    let _ = stream.write_all(resp.as_bytes());
}

fn serve_file(stream: &mut TcpStream, static_dir: &PathBuf, path: &str) {
    let clean = path.trim_start_matches('/');
    let file_path = static_dir.join(clean);
    if !file_path.starts_with(static_dir.as_path()) { send_404(stream); return; }

    let Ok(data) = std::fs::read(&file_path) else { send_404(stream); return; };

    let content_type = match file_path.extension().and_then(|e| e.to_str()) {
        Some("html") => "text/html",
        Some("js") => "application/javascript",
        Some("wasm") => "application/wasm",
        Some("com" | "COM") => "application/octet-stream",
        Some("css") => "text/css",
        _ => "application/octet-stream",
    };

    let header = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: {}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        content_type, data.len()
    );
    let _ = stream.write_all(header.as_bytes());
    let _ = stream.write_all(&data);
}

fn send_404(stream: &mut TcpStream) {
    let _ = stream.write_all(b"HTTP/1.1 404 Not Found\r\nConnection: close\r\n\r\n404");
}

fn handle_websocket<S: Read + Write>(stream: S, nonce: &str) {
    let mut ws = match tungstenite::accept(stream) {
        Ok(ws) => ws,
        Err(e) => { eprintln!("WS accept error: {}", e); return; }
    };

    // First message must be the nonce
    match ws.read() {
        Ok(tungstenite::Message::Text(msg)) => {
            if msg.trim() != nonce {
                eprintln!("WS bad nonce");
                let _ = ws.close(None);
                return;
            }
        }
        _ => { let _ = ws.close(None); return; }
    }

    let _ = ws.write(tungstenite::Message::Text("OK".into()));
    eprintln!("WS authenticated");

    loop {
        match ws.read() {
            Ok(tungstenite::Message::Text(prompt)) => {
                let prompt = prompt.trim().to_string();
                if prompt.is_empty() { continue; }
                eprintln!("WS: claude -p {:?}", &prompt[..prompt.len().min(60)]);

                let result = std::process::Command::new("claude")
                    .arg("-p")
                    .arg(&prompt)
                    .output();

                let response = match result {
                    Ok(output) if output.status.success() =>
                        String::from_utf8_lossy(&output.stdout).to_string(),
                    Ok(output) =>
                        format!("ERROR: {}", String::from_utf8_lossy(&output.stderr)),
                    Err(e) => format!("ERROR: {}", e),
                };

                if ws.write(tungstenite::Message::Text(response)).is_err() { break; }
            }
            Ok(tungstenite::Message::Close(_)) => break,
            Err(_) => break,
            _ => {}
        }
    }
    eprintln!("WS disconnected");
}
