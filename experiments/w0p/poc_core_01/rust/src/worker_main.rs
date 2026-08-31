//! RigForge POC-CORE-01 Rust worker
//! RESEARCH ONLY / W0-P / NON-PRODUCTION

use rigforge_poc_core_rs::*;
use std::ffi::CString;
use std::io::{self, BufRead, Write};

fn b64_encode(input: &[u8]) -> String {
    const T: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::new();
    let mut i = 0;
    while i < input.len() {
        let n = (input.len() - i).min(3);
        let mut a3 = [0u8; 3];
        a3[..n].copy_from_slice(&input[i..i + n]);
        i += n;
        let v = ((a3[0] as u32) << 16) | ((a3[1] as u32) << 8) | (a3[2] as u32);
        out.push(T[((v >> 18) & 63) as usize] as char);
        out.push(T[((v >> 12) & 63) as usize] as char);
        out.push(if n > 1 {
            T[((v >> 6) & 63) as usize] as char
        } else {
            '='
        });
        out.push(if n > 2 {
            T[(v & 63) as usize] as char
        } else {
            '='
        });
    }
    out
}

fn b64_val(c: u8) -> Option<u8> {
    match c {
        b'A'..=b'Z' => Some(c - b'A'),
        b'a'..=b'z' => Some(c - b'a' + 26),
        b'0'..=b'9' => Some(c - b'0' + 52),
        b'+' => Some(62),
        b'/' => Some(63),
        _ => None,
    }
}

fn b64_decode(input: &str) -> Option<Vec<u8>> {
    let mut out = Vec::new();
    let mut val: i32 = 0;
    let mut bits: i32 = -8;
    for c in input.bytes() {
        if c == b'=' || c == b'\n' || c == b'\r' {
            continue;
        }
        let d = b64_val(c)? as i32;
        val = (val << 6) + d;
        bits += 6;
        if bits >= 0 {
            out.push(((val >> bits) & 0xFF) as u8);
            bits -= 8;
        }
    }
    Some(out)
}

fn json_escape(s: &str) -> String {
    let mut o = String::new();
    for c in s.chars() {
        match c {
            '\\' | '"' => {
                o.push('\\');
                o.push(c);
            }
            '\n' => o.push_str("\\n"),
            _ => o.push(c),
        }
    }
    o
}

fn extract_string_field<'a>(json: &'a str, key: &str) -> Option<&'a str> {
    let pat = format!("\"{key}\":\"");
    let start = json.find(&pat)? + pat.len();
    let bytes = json.as_bytes();
    let mut i = start;
    let mut out_start = start;
    // values are taken as a slice of the original if unescaped; we need owned if escapes
    while i < bytes.len() {
        if bytes[i] == b'\\' {
            return extract_owned(json, start).map(|_| json.get(0..0).unwrap()); // fallback
        }
        if bytes[i] == b'"' {
            return Some(&json[out_start..i]);
        }
        i += 1;
    }
    None
}

fn extract_owned(json: &str, start: usize) -> Option<String> {
    let bytes = json.as_bytes();
    let mut i = start;
    let mut s = String::new();
    while i < bytes.len() {
        if bytes[i] == b'\\' && i + 1 < bytes.len() {
            s.push(bytes[i + 1] as char);
            i += 2;
            continue;
        }
        if bytes[i] == b'"' {
            return Some(s);
        }
        s.push(bytes[i] as char);
        i += 1;
    }
    None
}

fn extract_field(json: &str, key: &str) -> Option<String> {
    let pat = format!("\"{key}\":\"");
    let p = json.find(&pat)? + pat.len();
    extract_owned(json, p)
}

fn fail(code: &str, msg: &str) -> String {
    format!(
        "{{\"ok\":false,\"payload_b64\":\"\",\"diag\":[{{\"severity\":\"error\",\"code\":\"{code}\",\"message\":\"{}\",\"location\":\"\"}}]}}",
        json_escape(msg)
    )
}

fn read_str(
    f: unsafe extern "C" fn(*mut Asset, i32, *mut i8, usize, *mut usize) -> i32,
    a: *mut Asset,
    index: i32,
) -> String {
    let mut n = 0usize;
    unsafe { f(a, index, std::ptr::null_mut(), 0, &mut n) };
    let mut buf = vec![0i8; n + 1];
    unsafe { f(a, index, buf.as_mut_ptr(), buf.len(), &mut n) };
    unsafe { std::ffi::CStr::from_ptr(buf.as_ptr()) }
        .to_string_lossy()
        .into_owned()
}

fn diag_json(a: *mut Asset) -> String {
    let n = unsafe { rf_poc_diagnostic_count(a) };
    let mut out = String::from("[");
    for i in 0..n {
        let mut sev = 0i32;
        let mut cl = 0usize;
        let mut ml = 0usize;
        let mut ll = 0usize;
        unsafe {
            rf_poc_diagnostic(
                a,
                i,
                &mut sev,
                std::ptr::null_mut(),
                0,
                &mut cl,
                std::ptr::null_mut(),
                0,
                &mut ml,
                std::ptr::null_mut(),
                0,
                &mut ll,
            );
        }
        let mut code = vec![0i8; cl + 1];
        let mut msg = vec![0i8; ml + 1];
        let mut loc = vec![0i8; ll + 1];
        unsafe {
            rf_poc_diagnostic(
                a,
                i,
                &mut sev,
                code.as_mut_ptr(),
                code.len(),
                &mut cl,
                msg.as_mut_ptr(),
                msg.len(),
                &mut ml,
                loc.as_mut_ptr(),
                loc.len(),
                &mut ll,
            );
        }
        let code_s = unsafe { std::ffi::CStr::from_ptr(code.as_ptr()) }
            .to_string_lossy()
            .into_owned();
        let msg_s = unsafe { std::ffi::CStr::from_ptr(msg.as_ptr()) }
            .to_string_lossy()
            .into_owned();
        let loc_s = unsafe { std::ffi::CStr::from_ptr(loc.as_ptr()) }
            .to_string_lossy()
            .into_owned();
        let sev_s = match sev {
            RF_POC_SEV_WARN => "warn",
            RF_POC_SEV_ERROR => "error",
            _ => "info",
        };
        if i > 0 {
            out.push(',');
        }
        out.push_str(&format!(
            "{{\"severity\":\"{sev_s}\",\"code\":\"{}\",\"message\":\"{}\",\"location\":\"{}\"}}",
            json_escape(&code_s),
            json_escape(&msg_s),
            json_escape(&loc_s)
        ));
    }
    out.push(']');
    out
}

fn main() {
    let stdin = io::stdin();
    let mut line = String::new();
    let mut out = String::new();
    if stdin.lock().read_line(&mut line).unwrap_or(0) == 0 {
        out = fail("POC_BAD_OP", "empty stdin");
    } else {
        let op = extract_field(&line, "op").unwrap_or_default();
        if op != "load" {
            out = fail("POC_BAD_OP", "expected op load");
        } else if let Some(b64) = extract_field(&line, "payload_b64") {
            match b64_decode(&b64).and_then(|p| String::from_utf8(p).ok()) {
                Some(path) if !path.is_empty() => {
                    let cpath = match CString::new(path) {
                        Ok(c) => c,
                        Err(_) => {
                            out = fail("POC_PARSE", "path contained NUL");
                            print_line(&out);
                            return;
                        }
                    };
                    let mut st = RF_POC_OK;
                    let a = unsafe { rf_poc_load(cpath.as_ptr(), &mut st) };
                    if a.is_null() {
                        let (code, msg) = match st {
                            RF_POC_ERR_NULL => ("POC_NULL", "null path"),
                            RF_POC_ERR_IO => ("POC_IO", "file not found or unreadable"),
                            _ => ("POC_PARSE", "load failed"),
                        };
                        out = fail(code, msg);
                    } else {
                        let jc = unsafe { rf_poc_joint_count(a) };
                        let mc = unsafe { rf_poc_motion_count(a) };
                        let dc = unsafe { rf_poc_diagnostic_count(a) };
                        let mut t0 = 0.0;
                        let mut t1 = 0.0;
                        unsafe { rf_poc_motion_time_range(a, 0, &mut t0, &mut t1) };
                        let inner = format!(
                            "{{\"joint_count\":{jc},\"motion_count\":{mc},\"diag_count\":{dc},\"t0\":{t0:.1},\"t1\":{t1:.1}}}"
                        );
                        let payload = b64_encode(inner.as_bytes());
                        let diags = diag_json(a);
                        unsafe { rf_poc_destroy(a) };
                        out = format!(
                            "{{\"ok\":true,\"payload_b64\":\"{payload}\",\"diag\":{diags}}}"
                        );
                    }
                }
                _ => out = fail("POC_BAD_B64", "invalid payload_b64"),
            }
        } else {
            out = fail("POC_BAD_B64", "missing payload_b64");
        }
    }
    print_line(&out);
    let _ = extract_string_field;
    let _ = read_str;
}

fn print_line(s: &str) {
    let mut stdout = io::stdout();
    let _ = writeln!(stdout, "{s}");
    let _ = stdout.flush();
}
