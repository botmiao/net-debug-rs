use std::process::Command;

fn bin_path() -> String {
    format!("{}/target/debug/netd", env!("CARGO_MANIFEST_DIR"))
}

#[test]
fn test_help_flag() {
    let output = Command::new(bin_path())
        .arg("--help")
        .output()
        .expect("Failed to execute netd");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("netd"));
    assert!(stdout.contains("tcp"));
}

#[test]
fn test_tcp_server_help() {
    let output = Command::new(bin_path())
        .args(["tcp", "server", "--help"])
        .output()
        .expect("Failed to execute netd");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("ADDRESS") || stdout.contains("address"));
}

#[test]
fn test_tcp_client_help() {
    let output = Command::new(bin_path())
        .args(["tcp", "client", "--help"])
        .output()
        .expect("Failed to execute netd");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("LOCAL") || stdout.contains("REMOTE"));
}

#[test]
fn test_invalid_args_returns_error() {
    let output = Command::new(bin_path())
        .args(["nonexistent"])
        .output()
        .expect("Failed to execute netd");

    assert!(!output.status.success());
}

#[test]
fn test_version_flag() {
    let output = Command::new(bin_path())
        .arg("--version")
        .output()
        .expect("Failed to execute netd");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("netd"));
}
