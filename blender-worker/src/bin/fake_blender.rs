//! Test-only executable that speaks the Blender command shape without bpy.

use std::fs;
use std::path::PathBuf;

use serde_json::{json, Value};

fn after_dash(args: &[String]) -> &[String] {
    if let Some(idx) = args.iter().position(|a| a == "--") {
        &args[idx + 1..]
    } else {
        &[]
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let rest = after_dash(&args);
    if rest.len() < 2 {
        eprintln!("fake blender missing -- mode job.json");
        std::process::exit(1);
    }
    let mode = rest[0].as_str();
    let job_path = PathBuf::from(&rest[1]);
    let job: Value = serde_json::from_str(&fs::read_to_string(&job_path).expect("job json"))
        .expect("job json parse");
    let behavior = job
        .get("test_behavior")
        .and_then(Value::as_str)
        .unwrap_or("success");
    let outputs = job.get("outputs").cloned().unwrap_or(json!({}));
    let result_path = PathBuf::from(outputs["result_envelope"].as_str().unwrap_or("result.json"));
    let staged = PathBuf::from(outputs["staged_blend"].as_str().unwrap_or("staged.blend"));
    let reopen_path =
        PathBuf::from(outputs["reopen_verification"].as_str().unwrap_or("reopen.json"));
    let job_spec_id = job["job_spec_id"].as_str().unwrap_or("missing").to_string();
    let attempt_id = job["attempt_id"].as_str().unwrap_or("missing").to_string();
    let expected_version = job["expected_backend"]["version"]
        .as_str()
        .unwrap_or("5.2.1 LTS");
    let expected_build = job["expected_backend"]["build"]
        .as_str()
        .unwrap_or("9e2066aef7ef");
    let expected_adapter = job["expected_backend"]["adapter_version"]
        .as_str()
        .unwrap_or("rigforge-blender-worker/0.1.2");

    if mode != "reopen" && behavior == "unicode_stderr" {
        eprint!("{}", "é漢".repeat(900));
    }

    if mode == "reopen" {
        match behavior {
            "reopen_invalid_envelope" => {
                fs::write(reopen_path, b"{not-json").unwrap();
                return;
            }
            "reopen_delete_staged" => {
                let _ = fs::remove_file(&staged);
            }
            "reopen_missing_envelope" => return,
            "reopen_missing_root_scale_audit" => {
                let payload = json!({
                    "status": "SUCCESS",
                    "target_present": true,
                    "baked_action": "CharacterArmatureAction",
                    "frame_start": 1,
                    "frame_end": 61,
                    "bone_count": 32,
                    "diagnostics": ["fake reopen missing root_scale_audit"]
                });
                fs::write(reopen_path, serde_json::to_vec_pretty(&payload).unwrap()).unwrap();
                return;
            }
            "reopen_fail_root_scale_audit" => {
                let payload = json!({
                    "status": "SUCCESS",
                    "target_present": true,
                    "baked_action": "CharacterArmatureAction",
                    "frame_start": 1,
                    "frame_end": 61,
                    "bone_count": 32,
                    "root_scale_audit": "FAIL",
                    "diagnostics": ["fake reopen FAIL root_scale_audit"]
                });
                fs::write(reopen_path, serde_json::to_vec_pretty(&payload).unwrap()).unwrap();
                return;
            }
            _ => {}
        }
        let payload = json!({
            "status": "SUCCESS",
            "target_present": true,
            "baked_action": "CharacterArmatureAction",
            "frame_start": 1,
            "frame_end": 61,
            "bone_count": 32,
            "root_scale_audit": "PASS",
            "diagnostics": ["fake reopen"]
        });
        fs::write(reopen_path, serde_json::to_vec_pretty(&payload).unwrap()).unwrap();
        return;
    }

    let envelope = |status: &str, class: Option<&str>, spec: &str, version: &str, build: &str, adapter: &str, staged_val: Option<&str>, extra: Value| {
        let mut payload = json!({
            "schema": "rigforge.blender_worker.envelope.v1",
            "status": status,
            "failure_class": class,
            "job_spec_id": spec,
            "attempt_id": attempt_id,
            "backend": {
                "kind": "Blender",
                "version": version,
                "build": build
            },
            "adapter_version": adapter,
            "staged_blend": staged_val,
            "measurements": extra["measurements"],
            "diagnostics": extra["diagnostics"],
            "errors": extra["errors"]
        });
        if payload["measurements"].is_null() {
            payload["measurements"] = json!([]);
        }
        if payload["diagnostics"].is_null() {
            payload["diagnostics"] = json!([]);
        }
        if payload["errors"].is_null() {
            payload["errors"] = json!([]);
        }
        payload
    };

    match behavior {
        "missing_envelope" => {}
        "invalid_envelope" => {
            fs::write(&result_path, b"{not-json").unwrap();
        }
        "job_spec_mismatch" => {
            fs::create_dir_all(staged.parent().unwrap()).ok();
            fs::write(&staged, b"fake-blend").unwrap();
            let payload = envelope(
                "SUCCESS",
                None,
                "00000000-0000-7000-8000-000000000000",
                expected_version,
                expected_build,
                expected_adapter,
                Some(staged.to_str().unwrap()),
                json!({}),
            );
            fs::write(&result_path, serde_json::to_vec_pretty(&payload).unwrap()).unwrap();
        }
        "backend_mismatch" => {
            fs::create_dir_all(staged.parent().unwrap()).ok();
            fs::write(&staged, b"fake-blend").unwrap();
            let payload = envelope(
                "SUCCESS",
                None,
                &job_spec_id,
                expected_version,
                "deadbeefdead",
                expected_adapter,
                Some(staged.to_str().unwrap()),
                json!({}),
            );
            fs::write(&result_path, serde_json::to_vec_pretty(&payload).unwrap()).unwrap();
        }
        "backend_version_mismatch" => {
            fs::create_dir_all(staged.parent().unwrap()).ok();
            fs::write(&staged, b"fake-blend").unwrap();
            let payload = envelope(
                "SUCCESS",
                None,
                &job_spec_id,
                "2.9.0",
                expected_build,
                expected_adapter,
                Some(staged.to_str().unwrap()),
                json!({}),
            );
            fs::write(&result_path, serde_json::to_vec_pretty(&payload).unwrap()).unwrap();
        }
        "adapter_version_mismatch" => {
            fs::create_dir_all(staged.parent().unwrap()).ok();
            fs::write(&staged, b"fake-blend").unwrap();
            let payload = envelope(
                "SUCCESS",
                None,
                &job_spec_id,
                expected_version,
                expected_build,
                "other-adapter/9.9.9",
                Some(staged.to_str().unwrap()),
                json!({}),
            );
            fs::write(&result_path, serde_json::to_vec_pretty(&payload).unwrap()).unwrap();
        }
        "source_digest_mismatch" => {
            let payload = envelope(
                "FAIL",
                Some("source_digest_mismatch"),
                &job_spec_id,
                expected_version,
                expected_build,
                expected_adapter,
                None,
                json!({"errors": ["character digest mismatch"]}),
            );
            fs::write(&result_path, serde_json::to_vec_pretty(&payload).unwrap()).unwrap();
            std::process::exit(1);
        }
        "unsupported_policy" => {
            let payload = envelope(
                "FAIL",
                Some("unsupported_policy"),
                &job_spec_id,
                expected_version,
                expected_build,
                expected_adapter,
                None,
                json!({"errors": ["unsupported Policy/ik_policy"]}),
            );
            fs::write(&result_path, serde_json::to_vec_pretty(&payload).unwrap()).unwrap();
            std::process::exit(1);
        }
        "fail_structured" => {
            let payload = envelope(
                "FAIL",
                Some("structured_worker_fail"),
                &job_spec_id,
                expected_version,
                expected_build,
                expected_adapter,
                None,
                json!({"errors": ["structured FAIL"]}),
            );
            fs::write(&result_path, serde_json::to_vec_pretty(&payload).unwrap()).unwrap();
            std::process::exit(1);
        }
        "nonzero_exit" => {
            let payload = envelope(
                "FAIL",
                Some("worker_script_exception"),
                &job_spec_id,
                expected_version,
                expected_build,
                expected_adapter,
                None,
                json!({"errors": ["python exception"]}),
            );
            fs::write(&result_path, serde_json::to_vec_pretty(&payload).unwrap()).unwrap();
            std::process::exit(1);
        }
        "scale_audit_fail" => {
            fs::create_dir_all(staged.parent().unwrap()).ok();
            fs::write(&staged, b"fake-blend-bytes").unwrap();
            let payload = envelope(
                "SUCCESS",
                None,
                &job_spec_id,
                expected_version,
                expected_build,
                expected_adapter,
                Some(staged.to_str().unwrap()),
                json!({
                    "measurements": [
                        {"name": "phase_execute", "value": "PASS"},
                        {"name": "rotation_only_audit", "value": "PASS"},
                        {"name": "loop_closure", "value": "PASS"},
                        {"name": "quaternion_policy", "value": "PASS"},
                        {"name": "keep_target_rest_scale", "value": "FAIL"},
                        {"name": "root_scale_audit", "value": "FAIL"}
                    ],
                    "diagnostics": ["fake blender scale audit fail"]
                }),
            );
            fs::write(&result_path, serde_json::to_vec_pretty(&payload).unwrap()).unwrap();
        }
        "success_without_blend" => {
            let payload = envelope(
                "SUCCESS",
                None,
                &job_spec_id,
                expected_version,
                expected_build,
                expected_adapter,
                None,
                json!({"measurements": [{"name": "phase_execute", "value": "PASS"}]}),
            );
            fs::write(&result_path, serde_json::to_vec_pretty(&payload).unwrap()).unwrap();
        }
        _ => {
            fs::create_dir_all(staged.parent().unwrap()).ok();
            fs::write(&staged, b"fake-blend-bytes").unwrap();
            let payload = envelope(
                "SUCCESS",
                None,
                &job_spec_id,
                expected_version,
                expected_build,
                expected_adapter,
                Some(staged.to_str().unwrap()),
                json!({
                    "measurements": [
                        {"name": "phase_execute", "value": "PASS"},
                        {"name": "rotation_only_audit", "value": "PASS"},
                        {"name": "loop_closure", "value": "PASS"},
                        {"name": "quaternion_policy", "value": "PASS"},
                        {"name": "keep_target_rest_scale", "value": "PASS"},
                        {"name": "root_scale_audit", "value": "PASS"}
                    ],
                    "diagnostics": ["fake blender success"]
                }),
            );
            fs::write(&result_path, serde_json::to_vec_pretty(&payload).unwrap()).unwrap();
        }
    }
}
