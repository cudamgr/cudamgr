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
fn test_doctor_command_execution() {
    let output = Command::new("cargo")
        .args(&["run", "--", "doctor"])
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // The doctor command is now implemented.
    // It might pass (Exit Code 0) on a valid machine or fail (Exit Code 1) on CI without a GPU.
    // However, in BOTH cases, it should print the "System Information" report.

    // We check that the report logic actually ran by looking for headers from the report output.
    let report_ran =
        stdout.contains("System Information") || stdout.contains("Compatibility Summary");

    if !report_ran {
        println!("STDOUT: {}", stdout);
        println!("STDERR: {}", stderr);
    }

    assert!(
        report_ran,
        "Doctor command did not generate a system report"
    );
}
