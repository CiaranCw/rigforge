//! Local sidecar Preview host. Not Product authority.
//!
//! Serves a pinned offline viewer plus an Application-validated session.
//! WebView / browser runtime is an implementation detail.

use std::fs;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::thread;
use std::time::Duration;

use rigforge_app::{PreviewSession, PreviewSubject};

pub const VIEWER_LIBRARY: &str = "@google/model-viewer";
pub const VIEWER_VERSION: &str = "4.3.1";
pub const PAYLOAD_FORMAT: &str = "model/gltf-binary";

pub fn viewer_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("preview-viewer")
}

pub fn vendor_script() -> PathBuf {
    viewer_root().join("vendor").join("model-viewer.min.js")
}

#[derive(Debug)]
pub struct PreviewHost {
    pub url: String,
    pub port: u16,
    pub session_root: PathBuf,
}

impl PreviewHost {
    pub fn serve(session: &PreviewSession) -> Result<Self, String> {
        let viewer = viewer_root();
        if !viewer.join("index.html").is_file() {
            return Err("Preview viewer index.html is missing".into());
        }
        if !vendor_script().is_file() {
            return Err(
                "Pinned model-viewer runtime is missing; run preview-viewer vendor step".into(),
            );
        }
        let listener = TcpListener::bind("127.0.0.1:0").map_err(|e| e.to_string())?;
        listener
            .set_nonblocking(true)
            .map_err(|e| e.to_string())?;
        let port = listener.local_addr().map_err(|e| e.to_string())?.port();
        let session_root = session.root.clone();
        let viewer_dir = viewer;
        thread::spawn(move || loop {
            match listener.accept() {
                Ok((stream, _)) => {
                    let _ = handle(stream, &viewer_dir, &session_root);
                }
                Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => {
                    thread::sleep(Duration::from_millis(20));
                }
                Err(_) => break,
            }
        });
        Ok(Self {
            url: format!("http://127.0.0.1:{port}/"),
            port,
            session_root: session.root.clone(),
        })
    }

    pub fn open_sidecar(&self) -> Result<(), String> {
        open_local_url(&self.url)
    }
}

fn open_local_url(url: &str) -> Result<(), String> {
    let status = std::process::Command::new("cmd")
        .args(["/C", "start", "", url])
        .status()
        .map_err(|e| e.to_string())?;
    if status.success() {
        Ok(())
    } else {
        Err("failed to open local Preview window".into())
    }
}

fn handle(mut stream: TcpStream, viewer: &Path, session: &Path) -> std::io::Result<()> {
    let _ = stream.set_read_timeout(Some(Duration::from_secs(5)));
    let mut buf = [0u8; 4096];
    let n = stream.read(&mut buf)?;
    let req = String::from_utf8_lossy(&buf[..n]);
    let path = req
        .lines()
        .next()
        .and_then(|line| line.split_whitespace().nth(1))
        .unwrap_or("/");
    let decoded = path.split('?').next().unwrap_or("/");
    let rel = decoded.trim_start_matches('/');
    let (file, content_type) = if rel.is_empty() || rel == "index.html" {
        (viewer.join("index.html"), "text/html; charset=utf-8")
    } else if rel == "app.js" {
        (viewer.join("app.js"), "text/javascript; charset=utf-8")
    } else if rel == "app.css" {
        (viewer.join("app.css"), "text/css; charset=utf-8")
    } else if rel == "vendor/model-viewer.min.js" || rel == "./vendor/model-viewer.min.js" {
        (
            viewer.join("vendor").join("model-viewer.min.js"),
            "text/javascript; charset=utf-8",
        )
    } else if rel == "session.json" {
        (session.join("session.json"), "application/json")
    } else if rel == "payload.glb" {
        (session.join("payload.glb"), "model/gltf-binary")
    } else {
        respond(&mut stream, 404, "text/plain", b"not found")?;
        return Ok(());
    };
    match fs::read(&file) {
        Ok(bytes) => respond(&mut stream, 200, content_type, &bytes),
        Err(_) => respond(&mut stream, 404, "text/plain", b"not found"),
    }
}

fn respond(
    stream: &mut TcpStream,
    status: u16,
    content_type: &str,
    body: &[u8],
) -> std::io::Result<()> {
    let reason = if status == 200 { "OK" } else { "NOT FOUND" };
    let header = format!(
        "HTTP/1.1 {status} {reason}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nCache-Control: no-store\r\nAccess-Control-Allow-Origin: *\r\nConnection: close\r\n\r\n",
        body.len()
    );
    stream.write_all(header.as_bytes())?;
    stream.write_all(body)?;
    Ok(())
}

pub fn preview_kind_label(subject: &PreviewSubject) -> &'static str {
    match subject {
        PreviewSubject::Character { .. } => "Character",
        PreviewSubject::Motion { .. } => "Motion",
        PreviewSubject::DerivedVariant { .. } => "Derived Variant",
    }
}
