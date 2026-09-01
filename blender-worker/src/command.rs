//! Accepted Blender process command. Do not weaken safety flags.

use std::ffi::OsString;
use std::path::Path;
use std::process::Command;

pub const BACKGROUND: &str = "--background";
pub const FACTORY_STARTUP: &str = "--factory-startup";
pub const DISABLE_AUTOEXEC: &str = "--disable-autoexec";
pub const PYTHON_EXIT_CODE: &str = "--python-exit-code";
pub const PYTHON: &str = "--python";

pub fn blender_argv(
    blender: &Path,
    script: &Path,
    mode: &str,
    job_json: &Path,
) -> Vec<OsString> {
    vec![
        blender.as_os_str().to_os_string(),
        BACKGROUND.into(),
        FACTORY_STARTUP.into(),
        DISABLE_AUTOEXEC.into(),
        PYTHON_EXIT_CODE.into(),
        "1".into(),
        PYTHON.into(),
        script.as_os_str().to_os_string(),
        "--".into(),
        mode.into(),
        job_json.as_os_str().to_os_string(),
    ]
}

pub fn blender_command(
    blender: &Path,
    script: &Path,
    mode: &str,
    job_json: &Path,
) -> Command {
    let argv = blender_argv(blender, script, mode, job_json);
    let mut cmd = Command::new(&argv[0]);
    cmd.args(&argv[1..]);
    cmd
}

pub fn assert_safety_flags(argv: &[OsString]) -> bool {
    let text: Vec<String> = argv
        .iter()
        .map(|a| a.to_string_lossy().into_owned())
        .collect();
    text.windows(2).any(|w| w[0] == BACKGROUND)
        && text.iter().any(|a| a == FACTORY_STARTUP)
        && text.iter().any(|a| a == DISABLE_AUTOEXEC)
        && text.windows(2).any(|w| w[0] == PYTHON_EXIT_CODE && w[1] == "1")
        && text.iter().any(|a| a == PYTHON)
}
