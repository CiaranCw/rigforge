//! RigForge POC-CORE-01 Rust research implementation
//!
//! RESEARCH ONLY / W0-P / NON-PRODUCTION
//! POC ONLY — NOT CANONICAL
//!
//! See ../ALGORITHM.md for the shared extraction procedure.
//! Joint names use a two-pass length query; do not truncate to a fixed buffer.

#![allow(non_camel_case_types)]
#![allow(unused_unsafe)]

use std::collections::HashSet;
use std::ffi::CStr;
use std::os::raw::c_char;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::{Mutex, OnceLock};

pub const RF_POC_OK: i32 = 0;
pub const RF_POC_ERR_NULL: i32 = -1;
pub const RF_POC_ERR_INVALID_INDEX: i32 = -2;
pub const RF_POC_ERR_IO: i32 = -3;
pub const RF_POC_ERR_PARSE: i32 = -4;
pub const RF_POC_ERR_BUFFER: i32 = -5;
pub const RF_POC_ERR_INVALID_HANDLE: i32 = -6;
pub const RF_POC_SEV_INFO: i32 = 0;
pub const RF_POC_SEV_WARN: i32 = 1;
pub const RF_POC_SEV_ERROR: i32 = 2;

const ASSET_ID: &str = "poc_core_01_asset";

#[repr(C)]
struct PocUfbxScene {
    _private: [u8; 0],
}

extern "C" {
    fn poc_ufbx_load(path: *const c_char, out_kind: *mut i32) -> *mut PocUfbxScene;
    fn poc_ufbx_free(s: *mut PocUfbxScene);
    fn poc_ufbx_node_count(s: *mut PocUfbxScene) -> usize;
    fn poc_ufbx_node_is_root(s: *mut PocUfbxScene, index: usize) -> i32;
    fn poc_ufbx_node_name(
        s: *mut PocUfbxScene,
        index: usize,
        buf: *mut c_char,
        buf_len: usize,
    ) -> usize;
    fn poc_ufbx_parent_index(s: *mut PocUfbxScene, index: usize) -> i32;
    fn poc_ufbx_parent_is_root(s: *mut PocUfbxScene, index: usize) -> i32;
    fn poc_ufbx_local_t(s: *mut PocUfbxScene, index: usize, out_xyz: *mut f64);
    fn poc_ufbx_local_r(s: *mut PocUfbxScene, index: usize, out_xyzw: *mut f64);
    fn poc_ufbx_local_s(s: *mut PocUfbxScene, index: usize, out_xyz: *mut f64);
}

#[derive(Clone, Copy, Default)]
struct Vec3 {
    x: f64,
    y: f64,
    z: f64,
}

#[derive(Clone, Copy)]
struct Quat {
    x: f64,
    y: f64,
    z: f64,
    w: f64,
}

impl Default for Quat {
    fn default() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            w: 1.0,
        }
    }
}

struct TrsKey {
    time: f64,
    t: Vec3,
}

struct RotKey {
    time: f64,
    q: Quat,
}

struct Joint {
    name: String,
    parent: i32,
    t: Vec3,
    r: Quat,
    s: Vec3,
}

struct Track {
    joint: i32,
    t_keys: Vec<TrsKey>,
    r_keys: Vec<RotKey>,
}

struct Motion {
    name: String,
    t0: f64,
    t1: f64,
    tracks: Vec<Track>,
}

struct Diagnostic {
    severity: i32,
    code: String,
    message: String,
    location: String,
}

pub struct Asset {
    joints: Vec<Joint>,
    motions: Vec<Motion>,
    diags: Vec<Diagnostic>,
    id: String,
}

fn live_set() -> &'static Mutex<HashSet<usize>> {
    // Research harness only: pointer-address live registry.
    // Validates immediate post-destroy misuse on the single-threaded tested path.
    // NOT a production stale-handle / ABA / concurrent query-destroy design.
    static S: OnceLock<Mutex<HashSet<usize>>> = OnceLock::new();
    S.get_or_init(|| Mutex::new(HashSet::new()))
}

fn is_live(p: *mut Asset) -> bool {
    if p.is_null() {
        return false;
    }
    live_set()
        .lock()
        .map(|g| g.contains(&(p as usize)))
        .unwrap_or(false)
}

fn insert_live(p: *mut Asset) {
    if let Ok(mut g) = live_set().lock() {
        g.insert(p as usize);
    }
}

fn remove_live(p: *mut Asset) -> bool {
    live_set()
        .lock()
        .map(|mut g| g.remove(&(p as usize)))
        .unwrap_or(false)
}

fn copy_str(s: &str, buf: *mut c_char, buf_len: usize, out_len: *mut usize) -> i32 {
    if !out_len.is_null() {
        unsafe {
            *out_len = s.len();
        }
    }
    if buf.is_null() {
        return RF_POC_OK;
    }
    if buf_len < s.len() + 1 {
        return RF_POC_ERR_BUFFER;
    }
    unsafe {
        std::ptr::copy_nonoverlapping(s.as_ptr(), buf as *mut u8, s.len());
        *buf.add(s.len()) = 0;
    }
    RF_POC_OK
}

fn attach_synth(a: &mut Asset) {
    let n = a.joints.len() as i32;
    let mut tracks = Vec::new();
    for i in 0..n {
        let j = &a.joints[i as usize];
        let mut tr = Track {
            joint: i,
            t_keys: Vec::new(),
            r_keys: Vec::new(),
        };
        tr.t_keys.push(TrsKey {
            time: 0.0,
            t: j.t,
        });
        let mut t1 = j.t;
        t1.x += 0.1 * (i as f64);
        tr.t_keys.push(TrsKey { time: 1.0, t: t1 });
        if i != n - 1 {
            tr.r_keys.push(RotKey {
                time: 0.0,
                q: j.r,
            });
            tr.r_keys.push(RotKey {
                time: 1.0,
                q: j.r,
            });
        }
        tracks.push(tr);
    }
    a.motions.push(Motion {
        name: "poc_walk".to_string(),
        t0: 0.0,
        t1: 1.0,
        tracks,
    });
}

fn attach_diags(a: &mut Asset, path: &str) {
    a.diags.push(Diagnostic {
        severity: RF_POC_SEV_INFO,
        code: "POC_LOAD".to_string(),
        message: "ufbx load ok".to_string(),
        location: path.to_string(),
    });
    a.diags.push(Diagnostic {
        severity: RF_POC_SEV_INFO,
        code: "POC_JOINTS".to_string(),
        message: format!("{} joints extracted", a.joints.len()),
        location: String::new(),
    });
    a.diags.push(Diagnostic {
        severity: RF_POC_SEV_WARN,
        code: "POC_SYNTH".to_string(),
        message: "native FBX animation unused; synthetic research motion attached".to_string(),
        location: String::new(),
    });
    let last = a.joints.last().map(|j| j.name.clone()).unwrap_or_default();
    a.diags.push(Diagnostic {
        severity: RF_POC_SEV_INFO,
        code: "POC_EMPTY_ROT".to_string(),
        message: "last joint rotation keys empty by design".to_string(),
        location: last,
    });
}

fn load_inner(path: *const c_char, out_status: *mut i32) -> *mut Asset {
    unsafe {
        if !out_status.is_null() {
            *out_status = RF_POC_OK;
        }
    }
    if path.is_null() {
        unsafe {
            if !out_status.is_null() {
                *out_status = RF_POC_ERR_NULL;
            }
        }
        return std::ptr::null_mut();
    }
    let mut kind: i32 = 0;
    let scene = unsafe { poc_ufbx_load(path, &mut kind) };
    if scene.is_null() {
        unsafe {
            if !out_status.is_null() {
                *out_status = if kind == 1 {
                    RF_POC_ERR_IO
                } else {
                    RF_POC_ERR_PARSE
                };
            }
        }
        return std::ptr::null_mut();
    }

    let path_str = unsafe { CStr::from_ptr(path) }
        .to_string_lossy()
        .into_owned();

    let count = unsafe { poc_ufbx_node_count(scene) };
    let mut raw_indices: Vec<usize> = Vec::new();
    for i in 0..count {
        if unsafe { poc_ufbx_node_is_root(scene, i) } != 0 {
            continue;
        }
        raw_indices.push(i);
    }

    let mut joints: Vec<Joint> = Vec::new();
    for &i in &raw_indices {
        let nlen = unsafe { poc_ufbx_node_name(scene, i, std::ptr::null_mut(), 0) };
        let name = if nlen == 0 {
            "unnamed".to_string()
        } else {
            let mut buf = vec![0u8; nlen + 1];
            let got = unsafe {
                poc_ufbx_node_name(scene, i, buf.as_mut_ptr() as *mut c_char, buf.len())
            };
            let n = got.min(nlen);
            String::from_utf8_lossy(&buf[..n]).into_owned()
        };
        let mut t = [0.0f64; 3];
        let mut r = [0.0f64; 4];
        let mut s = [0.0f64; 3];
        unsafe {
            poc_ufbx_local_t(scene, i, t.as_mut_ptr());
            poc_ufbx_local_r(scene, i, r.as_mut_ptr());
            poc_ufbx_local_s(scene, i, s.as_mut_ptr());
        }
        joints.push(Joint {
            name,
            parent: -1,
            t: Vec3 {
                x: t[0],
                y: t[1],
                z: t[2],
            },
            r: Quat {
                x: r[0],
                y: r[1],
                z: r[2],
                w: r[3],
            },
            s: Vec3 {
                x: s[0],
                y: s[1],
                z: s[2],
            },
        });
    }

    for (ji, &raw) in raw_indices.iter().enumerate() {
        let parent = if unsafe { poc_ufbx_parent_is_root(scene, raw) } != 0 {
            -1
        } else {
            let p = unsafe { poc_ufbx_parent_index(scene, raw) };
            raw_indices
                .iter()
                .position(|&x| x as i32 == p)
                .map(|v| v as i32)
                .unwrap_or(-1)
        };
        joints[ji].parent = parent;
    }

    unsafe { poc_ufbx_free(scene) };

    let mut asset = Asset {
        joints,
        motions: Vec::new(),
        diags: Vec::new(),
        id: ASSET_ID.to_string(),
    };
    attach_synth(&mut asset);
    attach_diags(&mut asset, &path_str);
    let p = Box::into_raw(Box::new(asset));
    insert_live(p);
    p
}

fn asset_ref<'a>(p: *mut Asset) -> Option<&'a Asset> {
    if !is_live(p) {
        return None;
    }
    Some(unsafe { &*p })
}

/// PoC-only C ABI unwind boundary. Not a production panic policy.
/// Residual: invalid pointers remain UB; catch_unwind does not make FFI memory-safe.
/// Mutex poison is treated as not-live and does not panic.
fn abi_status<F: FnOnce() -> i32>(f: F) -> i32 {
    match catch_unwind(AssertUnwindSafe(f)) {
        Ok(v) => v,
        Err(_) => RF_POC_ERR_PARSE,
    }
}

fn abi_count<F: FnOnce() -> i32>(f: F) -> i32 {
    match catch_unwind(AssertUnwindSafe(f)) {
        Ok(v) => v,
        Err(_) => -1,
    }
}

#[no_mangle]
pub unsafe extern "C" fn rf_poc_load(path: *const c_char, out_status: *mut i32) -> *mut Asset {
    match catch_unwind(AssertUnwindSafe(|| load_inner(path, out_status))) {
        Ok(p) => p,
        Err(_) => {
            if !out_status.is_null() {
                *out_status = RF_POC_ERR_PARSE;
            }
            std::ptr::null_mut()
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn rf_poc_destroy(asset: *mut Asset) {
    let _ = catch_unwind(AssertUnwindSafe(|| {
        if asset.is_null() {
            return;
        }
        if !remove_live(asset) {
            return;
        }
        drop(Box::from_raw(asset));
    }));
}

#[no_mangle]
pub unsafe extern "C" fn rf_poc_asset_id(
    asset: *mut Asset,
    buf: *mut c_char,
    buf_len: usize,
    out_len: *mut usize,
) -> i32 {
    abi_status(|| unsafe {
    if asset.is_null() {
        return RF_POC_ERR_NULL;
    }
    match asset_ref(asset) {
        Some(a) => copy_str(&a.id, buf, buf_len, out_len),
        None => RF_POC_ERR_INVALID_HANDLE,
    }
    })
}

#[no_mangle]
pub unsafe extern "C" fn rf_poc_joint_count(asset: *mut Asset) -> i32 {
    abi_count(|| unsafe {
    match asset_ref(asset) {
        Some(a) => a.joints.len() as i32,
        None => -1,
    }
    })
}

#[no_mangle]
pub unsafe extern "C" fn rf_poc_joint_parent(
    asset: *mut Asset,
    index: i32,
    out_parent: *mut i32,
) -> i32 {
    abi_status(|| unsafe {
    if asset.is_null() {
        return RF_POC_ERR_NULL;
    }
    let Some(a) = asset_ref(asset) else {
        return RF_POC_ERR_INVALID_HANDLE;
    };
    if out_parent.is_null() {
        return RF_POC_ERR_NULL;
    }
    if index < 0 || index as usize >= a.joints.len() {
        return RF_POC_ERR_INVALID_INDEX;
    }
    *out_parent = a.joints[index as usize].parent;
    RF_POC_OK
    })
}

#[no_mangle]
pub unsafe extern "C" fn rf_poc_joint_name(
    asset: *mut Asset,
    index: i32,
    buf: *mut c_char,
    buf_len: usize,
    out_len: *mut usize,
) -> i32 {
    abi_status(|| unsafe {
    if asset.is_null() {
        return RF_POC_ERR_NULL;
    }
    let Some(a) = asset_ref(asset) else {
        return RF_POC_ERR_INVALID_HANDLE;
    };
    if index < 0 || index as usize >= a.joints.len() {
        return RF_POC_ERR_INVALID_INDEX;
    }
    copy_str(&a.joints[index as usize].name, buf, buf_len, out_len)
    })
}

#[no_mangle]
pub unsafe extern "C" fn rf_poc_joint_rest_translation(
    asset: *mut Asset,
    index: i32,
    out_xyz: *mut f64,
) -> i32 {
    abi_status(|| unsafe {
    if asset.is_null() || out_xyz.is_null() {
        return RF_POC_ERR_NULL;
    }
    let Some(a) = asset_ref(asset) else {
        return RF_POC_ERR_INVALID_HANDLE;
    };
    if index < 0 || index as usize >= a.joints.len() {
        return RF_POC_ERR_INVALID_INDEX;
    }
    let t = a.joints[index as usize].t;
    *out_xyz = t.x;
    *out_xyz.add(1) = t.y;
    *out_xyz.add(2) = t.z;
    RF_POC_OK
    })
}

#[no_mangle]
pub unsafe extern "C" fn rf_poc_joint_rest_rotation(
    asset: *mut Asset,
    index: i32,
    out_xyzw: *mut f64,
) -> i32 {
    abi_status(|| unsafe {
    if asset.is_null() || out_xyzw.is_null() {
        return RF_POC_ERR_NULL;
    }
    let Some(a) = asset_ref(asset) else {
        return RF_POC_ERR_INVALID_HANDLE;
    };
    if index < 0 || index as usize >= a.joints.len() {
        return RF_POC_ERR_INVALID_INDEX;
    }
    let r = a.joints[index as usize].r;
    *out_xyzw = r.x;
    *out_xyzw.add(1) = r.y;
    *out_xyzw.add(2) = r.z;
    *out_xyzw.add(3) = r.w;
    RF_POC_OK
    })
}

#[no_mangle]
pub unsafe extern "C" fn rf_poc_joint_rest_scale(
    asset: *mut Asset,
    index: i32,
    out_xyz: *mut f64,
) -> i32 {
    abi_status(|| unsafe {
    if asset.is_null() || out_xyz.is_null() {
        return RF_POC_ERR_NULL;
    }
    let Some(a) = asset_ref(asset) else {
        return RF_POC_ERR_INVALID_HANDLE;
    };
    if index < 0 || index as usize >= a.joints.len() {
        return RF_POC_ERR_INVALID_INDEX;
    }
    let s = a.joints[index as usize].s;
    *out_xyz = s.x;
    *out_xyz.add(1) = s.y;
    *out_xyz.add(2) = s.z;
    RF_POC_OK
    })
}

#[no_mangle]
pub unsafe extern "C" fn rf_poc_motion_count(asset: *mut Asset) -> i32 {
    abi_count(|| unsafe {
    match asset_ref(asset) {
        Some(a) => a.motions.len() as i32,
        None => -1,
    }
    })
}

#[no_mangle]
pub unsafe extern "C" fn rf_poc_motion_name(
    asset: *mut Asset,
    motion_index: i32,
    buf: *mut c_char,
    buf_len: usize,
    out_len: *mut usize,
) -> i32 {
    abi_status(|| unsafe {
    if asset.is_null() {
        return RF_POC_ERR_NULL;
    }
    let Some(a) = asset_ref(asset) else {
        return RF_POC_ERR_INVALID_HANDLE;
    };
    if motion_index < 0 || motion_index as usize >= a.motions.len() {
        return RF_POC_ERR_INVALID_INDEX;
    }
    copy_str(&a.motions[motion_index as usize].name, buf, buf_len, out_len)
    })
}

#[no_mangle]
pub unsafe extern "C" fn rf_poc_motion_time_range(
    asset: *mut Asset,
    motion_index: i32,
    out_t0: *mut f64,
    out_t1: *mut f64,
) -> i32 {
    abi_status(|| unsafe {
    if asset.is_null() || out_t0.is_null() || out_t1.is_null() {
        return RF_POC_ERR_NULL;
    }
    let Some(a) = asset_ref(asset) else {
        return RF_POC_ERR_INVALID_HANDLE;
    };
    if motion_index < 0 || motion_index as usize >= a.motions.len() {
        return RF_POC_ERR_INVALID_INDEX;
    }
    *out_t0 = a.motions[motion_index as usize].t0;
    *out_t1 = a.motions[motion_index as usize].t1;
    RF_POC_OK
    })
}

#[no_mangle]
pub unsafe extern "C" fn rf_poc_track_count(asset: *mut Asset, motion_index: i32) -> i32 {
    abi_count(|| unsafe {
    match asset_ref(asset) {
        Some(a) => {
            if motion_index < 0 || motion_index as usize >= a.motions.len() {
                -1
            } else {
                a.motions[motion_index as usize].tracks.len() as i32
            }
        }
        None => -1,
    }
    })
}

#[no_mangle]
pub unsafe extern "C" fn rf_poc_track_joint(
    asset: *mut Asset,
    motion_index: i32,
    track_index: i32,
    out_joint: *mut i32,
) -> i32 {
    abi_status(|| unsafe {
    if asset.is_null() || out_joint.is_null() {
        return RF_POC_ERR_NULL;
    }
    let Some(a) = asset_ref(asset) else {
        return RF_POC_ERR_INVALID_HANDLE;
    };
    if motion_index < 0 || motion_index as usize >= a.motions.len() {
        return RF_POC_ERR_INVALID_INDEX;
    }
    let m = &a.motions[motion_index as usize];
    if track_index < 0 || track_index as usize >= m.tracks.len() {
        return RF_POC_ERR_INVALID_INDEX;
    }
    *out_joint = m.tracks[track_index as usize].joint;
    RF_POC_OK
    })
}

#[no_mangle]
pub unsafe extern "C" fn rf_poc_track_translation_key_count(
    asset: *mut Asset,
    motion_index: i32,
    track_index: i32,
) -> i32 {
    abi_count(|| unsafe {
    let Some(a) = asset_ref(asset) else {
        return -1;
    };
    if motion_index < 0 || motion_index as usize >= a.motions.len() {
        return -1;
    }
    let m = &a.motions[motion_index as usize];
    if track_index < 0 || track_index as usize >= m.tracks.len() {
        return -1;
    }
    m.tracks[track_index as usize].t_keys.len() as i32
    })
}

#[no_mangle]
pub unsafe extern "C" fn rf_poc_track_rotation_key_count(
    asset: *mut Asset,
    motion_index: i32,
    track_index: i32,
) -> i32 {
    abi_count(|| unsafe {
    let Some(a) = asset_ref(asset) else {
        return -1;
    };
    if motion_index < 0 || motion_index as usize >= a.motions.len() {
        return -1;
    }
    let m = &a.motions[motion_index as usize];
    if track_index < 0 || track_index as usize >= m.tracks.len() {
        return -1;
    }
    m.tracks[track_index as usize].r_keys.len() as i32
    })
}

#[no_mangle]
pub unsafe extern "C" fn rf_poc_track_translation_key(
    asset: *mut Asset,
    motion_index: i32,
    track_index: i32,
    key_index: i32,
    out_time: *mut f64,
    out_xyz: *mut f64,
) -> i32 {
    abi_status(|| unsafe {
    if asset.is_null() || out_time.is_null() || out_xyz.is_null() {
        return RF_POC_ERR_NULL;
    }
    let Some(a) = asset_ref(asset) else {
        return RF_POC_ERR_INVALID_HANDLE;
    };
    if motion_index < 0 || motion_index as usize >= a.motions.len() {
        return RF_POC_ERR_INVALID_INDEX;
    }
    let m = &a.motions[motion_index as usize];
    if track_index < 0 || track_index as usize >= m.tracks.len() {
        return RF_POC_ERR_INVALID_INDEX;
    }
    let tr = &m.tracks[track_index as usize];
    if key_index < 0 || key_index as usize >= tr.t_keys.len() {
        return RF_POC_ERR_INVALID_INDEX;
    }
    let k = &tr.t_keys[key_index as usize];
    *out_time = k.time;
    *out_xyz = k.t.x;
    *out_xyz.add(1) = k.t.y;
    *out_xyz.add(2) = k.t.z;
    RF_POC_OK
    })
}

#[no_mangle]
pub unsafe extern "C" fn rf_poc_track_rotation_key(
    asset: *mut Asset,
    motion_index: i32,
    track_index: i32,
    key_index: i32,
    out_time: *mut f64,
    out_xyzw: *mut f64,
) -> i32 {
    abi_status(|| unsafe {
    if asset.is_null() || out_time.is_null() || out_xyzw.is_null() {
        return RF_POC_ERR_NULL;
    }
    let Some(a) = asset_ref(asset) else {
        return RF_POC_ERR_INVALID_HANDLE;
    };
    if motion_index < 0 || motion_index as usize >= a.motions.len() {
        return RF_POC_ERR_INVALID_INDEX;
    }
    let m = &a.motions[motion_index as usize];
    if track_index < 0 || track_index as usize >= m.tracks.len() {
        return RF_POC_ERR_INVALID_INDEX;
    }
    let tr = &m.tracks[track_index as usize];
    if key_index < 0 || key_index as usize >= tr.r_keys.len() {
        return RF_POC_ERR_INVALID_INDEX;
    }
    let k = &tr.r_keys[key_index as usize];
    *out_time = k.time;
    *out_xyzw = k.q.x;
    *out_xyzw.add(1) = k.q.y;
    *out_xyzw.add(2) = k.q.z;
    *out_xyzw.add(3) = k.q.w;
    RF_POC_OK
    })
}

#[no_mangle]
pub unsafe extern "C" fn rf_poc_diagnostic_count(asset: *mut Asset) -> i32 {
    abi_count(|| unsafe {
    match asset_ref(asset) {
        Some(a) => a.diags.len() as i32,
        None => -1,
    }
    })
}

#[no_mangle]
pub unsafe extern "C" fn rf_poc_diagnostic(
    asset: *mut Asset,
    index: i32,
    out_severity: *mut i32,
    code_buf: *mut c_char,
    code_len: usize,
    code_out: *mut usize,
    msg_buf: *mut c_char,
    msg_len: usize,
    msg_out: *mut usize,
    loc_buf: *mut c_char,
    loc_len: usize,
    loc_out: *mut usize,
) -> i32 {
    abi_status(|| unsafe {
    if asset.is_null() || out_severity.is_null() {
        return RF_POC_ERR_NULL;
    }
    let Some(a) = asset_ref(asset) else {
        return RF_POC_ERR_INVALID_HANDLE;
    };
    if index < 0 || index as usize >= a.diags.len() {
        return RF_POC_ERR_INVALID_INDEX;
    }
    let d = &a.diags[index as usize];
    *out_severity = d.severity;
    let st = copy_str(&d.code, code_buf, code_len, code_out);
    if st != RF_POC_OK {
        return st;
    }
    let st = copy_str(&d.message, msg_buf, msg_len, msg_out);
    if st != RF_POC_OK {
        return st;
    }
    copy_str(&d.location, loc_buf, loc_len, loc_out)
    })
}
