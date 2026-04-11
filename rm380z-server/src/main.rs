use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;

/// Simple server: serves static files + handles WebSocket for claude -p.
/// One port, no TLS needed (served from localhost).

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let port = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(8380u16);
    let static_dir = args.get(2).map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("deploy/380z"));

    // Generate a nonce for this server session
    let nonce: String = (0..16).map(|_| {
        let c = rand::random::<u8>() % 36;
        if c < 10 { (b'0' + c) as char } else { (b'a' + c - 10) as char }
    }).collect();

    println!("RM 380Z Server");
    println!("  http://localhost:{}/", port);
    println!("  Static: {}", static_dir.display());
    println!("  Nonce: {}", nonce);
    println!();

    let listener = TcpListener::bind(format!("0.0.0.0:{}", port))
        .expect("Failed to bind");

    let nonce = Arc::new(nonce);
    let static_dir = Arc::new(static_dir);

    for stream in listener.incoming().flatten() {
        let nonce = nonce.clone();
        let static_dir = static_dir.clone();
        thread::spawn(move || {
            handle_connection(stream, &static_dir, &nonce);
        });
    }
}

fn handle_connection(mut stream: TcpStream, static_dir: &PathBuf, nonce: &str) {
    let mut buf = [0u8; 4096];
    let n = match stream.read(&mut buf) {
        Ok(n) if n > 0 => n,
        _ => return,
    };
    let request = String::from_utf8_lossy(&buf[..n]);

    // Check for WebSocket upgrade
    if request.contains("Upgrade: websocket") || request.contains("upgrade: websocket") {
        handle_websocket(stream, &request, nonce);
        return;
    }

    // Parse HTTP request line
    let first_line = request.lines().next().unwrap_or("");
    let path = first_line.split_whitespace().nth(1).unwrap_or("/");

    // Serve index.html with nonce injected
    if path == "/" || path == "/index.html" {
        serve_index(&mut stream, static_dir, nonce);
    } else {
        serve_file(&mut stream, static_dir, path);
    }
}

fn serve_index(stream: &mut TcpStream, static_dir: &PathBuf, nonce: &str) {
    let index_path = static_dir.join("index.html");
    let Ok(mut html) = std::fs::read_to_string(&index_path) else {
        send_404(stream);
        return;
    };

    // Inject the nonce and WebSocket URL into the page
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

    // Security: don't serve outside static_dir
    if !file_path.starts_with(static_dir.as_path()) {
        send_404(stream);
        return;
    }

    let Ok(data) = std::fs::read(&file_path) else {
        send_404(stream);
        return;
    };

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

fn handle_websocket(stream: TcpStream, request: &str, nonce: &str) {
    // Accept the WebSocket upgrade
    let mut ws = match tungstenite::accept(stream) {
        Ok(ws) => ws,
        Err(e) => { eprintln!("WS accept error: {}", e); return; }
    };

    // First message must be the nonce
    match ws.read() {
        Ok(tungstenite::Message::Text(msg)) => {
            if msg.trim() != nonce {
                eprintln!("WS bad nonce: {:?}", msg);
                let _ = ws.write(tungstenite::Message::Text("ERROR: bad nonce".into()));
                let _ = ws.close(None);
                return;
            }
        }
        _ => {
            eprintln!("WS expected nonce, got something else");
            let _ = ws.close(None);
            return;
        }
    }

    let _ = ws.write(tungstenite::Message::Text("OK".into()));
    eprintln!("WS authenticated");

    // Message loop: receive prompts, run claude -p, send responses
    loop {
        match ws.read() {
            Ok(tungstenite::Message::Text(prompt)) => {
                let prompt = prompt.trim().to_string();
                if prompt.is_empty() { continue; }
                eprintln!("WS prompt: {:?}", &prompt[..prompt.len().min(60)]);

                // Run claude -p
                let result = std::process::Command::new("claude")
                    .arg("-p")
                    .arg(&prompt)
                    .output();

                let response = match result {
                    Ok(output) if output.status.success() => {
                        String::from_utf8_lossy(&output.stdout).to_string()
                    }
                    Ok(output) => {
                        format!("ERROR: {}", String::from_utf8_lossy(&output.stderr))
                    }
                    Err(e) => format!("ERROR: {}", e),
                };

                if ws.write(tungstenite::Message::Text(response)).is_err() {
                    break;
                }
            }
            Ok(tungstenite::Message::Close(_)) => break,
            Err(_) => break,
            _ => {}
        }
    }
    eprintln!("WS disconnected");
}
