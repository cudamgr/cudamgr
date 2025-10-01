use std::process::Command;

#[test]
fn test_cli_help() {
    let output = Command::new("cargo")
        .args(&["run", "--", "--help"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("cudamgr"));
    assert!(stdout.contains("doctor"));
    assert!(stdout.contains("install"));
    assert!(stdout.contains("use"));
    assert!(stdout.contains("list"));
    assert!(stdout.contains("uninstall"));
    assert!(stdout.contains("logs"));
}

#[test]
fn test_doctor_help() {
    let output = Command::new("cargo")
        .args(&["run", "--", "doctor", "--help"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Check system compatibility"));
    assert!(stdout.contains("verbose"));
}

#[test]
fn test_install_help() {
    let output = Command::new("cargo")
        .args(&["run", "--", "install", "--help"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Install a specific CUDA version"));
    assert!(stdout.contains("force"));
    assert!(stdout.contains("skip-driver"));
}

#[test]
fn test_doctor_command_not_implemented() {
    let output = Command::new("cargo")
        .args(&["run", "--", "doctor"])
        .output()
        .expect("Failed to execute command");

    // Should fail with "not implemented" error
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("not yet implemented") || stderr.contains("not implemented"));
}