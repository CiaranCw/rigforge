//! Local sidecar Preview host. Not Product authority.
//!
//! Serves a pinned offline viewer plus an Application-validated session.
//! WebView / browser runtime is an implementation detail.
//!
//! The host owns its listener thread. Drop / shutdown join that thread.
//! Workbench retains the active host so Preview replace/exit cannot leak
//! a detached listener for the process lifetime.

use std::fs;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::Duration;

use rigforge_app::{PreviewSession, PreviewSubject, RuntimeLayout};

pub const VIEWER_LIBRARY: &str = "@google/model-viewer";
pub const VIEWER_VERSION: &str = "4.3.1";
pub const PAYLOAD_FORMAT: &str = "model/gltf-binary";
pub const VIEWER_SCRIPT_SHA256: &str =
    "283b0672384614b4847636c306fc93fe4b1fcadc76d668b4e47f0ca76bcf033b";

static LIVE_SERVERS: AtomicUsize = AtomicUsize::new(0);

pub fn viewer_root() -> Result<PathBuf, String> {
    RuntimeLayout::resolve()
        .map(|layout| layout.viewer_root())
        .map_err(|err| err.to_string())
}

pub fn vendor_script() -> Result<PathBuf, String> {
    Ok(viewer_root()?.join("vendor").join("model-viewer.min.js"))
}

pub fn live_server_count() -> usize {
    LIVE_SERVERS.load(Ordering::SeqCst)
}

pub struct PreviewHost {
    pub url: String,
    pub port: u16,
    pub session_root: PathBuf,
    stop: Arc<AtomicBool>,
    thread: Option<JoinHandle<()>>,
}

impl std::fmt::Debug for PreviewHost {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PreviewHost")
            .field("url", &self.url)
            .field("port", &self.port)
            .field("session_root", &self.session_root)
            .field("serving", &self.is_serving())
            .finish()
    }
}

impl PreviewHost {
    pub fn serve(session: &PreviewSession) -> Result<Self, String> {
        let viewer = viewer_root()?;
        if !viewer.join("index.html").is_file() {
            return Err("Preview viewer index.html is missing".into());
        }
        verify_vendor_runtime()?;
        let listener = TcpListener::bind("127.0.0.1:0").map_err(|e| e.to_string())?;
        listener
            .set_nonblocking(true)
            .map_err(|e| e.to_string())?;
        let port = listener.local_addr().map_err(|e| e.to_string())?.port();
        let session_root = session.root.clone();
        let viewer_dir = viewer;
        let stop = Arc::new(AtomicBool::new(false));
        let thread_stop = Arc::clone(&stop);
        let thread = thread::spawn(move || {
            LIVE_SERVERS.fetch_add(1, Ordering::SeqCst);
            while !thread_stop.load(Ordering::SeqCst) {
                match listener.accept() {
                    Ok((stream, _)) => {
                        let _ = handle(stream, &viewer_dir, &session_root);
                    }
                    Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => {
                        thread::sleep(Duration::from_millis(20));
                    }
                    Err(_) => break,
                }
            }
            LIVE_SERVERS.fetch_sub(1, Ordering::SeqCst);
        });
        Ok(Self {
            url: format!("http://127.0.0.1:{port}/"),
            port,
            session_root: session.root.clone(),
            stop,
            thread: Some(thread),
        })
    }

    pub fn is_serving(&self) -> bool {
        !self.stop.load(Ordering::SeqCst)
            && self
                .thread
                .as_ref()
                .map(|handle| !handle.is_finished())
                .unwrap_or(false)
    }

    pub fn shutdown(&mut self) {
        self.stop.store(true, Ordering::SeqCst);
        if let Some(handle) = self.thread.take() {
            let _ = handle.join();
        }
    }

    pub fn open_sidecar(&self) -> Result<(), String> {
        open_local_url(&self.url)
    }
}

impl Drop for PreviewHost {
    fn drop(&mut self) {
        self.shutdown();
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

fn sha256_hex(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    let digest = Sha256::digest(bytes);
    digest.iter().map(|b| format!("{b:02x}")).collect()
}

pub fn verify_vendor_runtime() -> Result<(), String> {
    let path = vendor_script()?;
    verify_vendor_file(&path)
}

pub fn verify_vendor_file(path: &Path) -> Result<(), String> {
    if !path.is_file() {
        return Err(
            "Pinned model-viewer runtime is missing; run preview-viewer vendor step".into(),
        );
    }
    let bytes = fs::read(path).map_err(|err| err.to_string())?;
    let found = sha256_hex(&bytes);
    if found != VIEWER_SCRIPT_SHA256 {
        return Err(format!(
            "model-viewer sha256 mismatch: expected {VIEWER_SCRIPT_SHA256} found {found}"
        ));
    }
    Ok(())
}
