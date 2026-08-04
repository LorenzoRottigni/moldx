use assert_cmd::Command;
use predicates::str::contains;
use std::path::PathBuf;

fn playground() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("playground")
}

fn moldx_dir() -> PathBuf {
    playground().join(".moldx")
}

fn module(name: &str) -> PathBuf {
    playground().join("modules").join(name)
}

fn moldx() -> Command {
    let mut cmd = Command::cargo_bin("moldx").unwrap();
    cmd.env("MOLDX_DIR", moldx_dir().to_str().unwrap());
    cmd
}

// ── detect ────────────────────────────────────────────────────────────────────

#[test]
fn detect_docker_module() {
    moldx()
        .args(["detect", module("auth-service").to_str().unwrap()])
        .assert()
        .success()
        .stdout(contains("docker"));
}

#[test]
fn detect_node_module() {
    moldx()
        .args(["detect", module("api-server").to_str().unwrap()])
        .assert()
        .success()
        .stdout(contains("node"));
}

#[test]
fn detect_rust_module() {
    moldx()
        .args(["detect", module("worker").to_str().unwrap()])
        .assert()
        .success()
        .stdout(contains("rust"));
}

#[test]
fn detect_multi_strategy_module_reports_all() {
    moldx()
        .args(["detect", module("multi-strategy").to_str().unwrap()])
        .assert()
        .success()
        .stdout(contains("docker"))
        .stdout(contains("node"))
        .stdout(contains("rust"));
}

// ── list ─────────────────────────────────────────────────────────────────────

#[test]
fn list_discovers_all_modules() {
    moldx()
        .args(["list", playground().to_str().unwrap()])
        .assert()
        .success()
        .stdout(contains("auth-service"))
        .stdout(contains("api-server"))
        .stdout(contains("worker"))
        .stdout(contains("multi-strategy"));
}

#[test]
fn list_shows_strategy_names() {
    moldx()
        .args(["list", playground().to_str().unwrap()])
        .assert()
        .success()
        .stdout(contains("docker"))
        .stdout(contains("node"))
        .stdout(contains("rust"));
}

// ── run (moldx <strategy> <command> <path>) ───────────────────────────────────

#[test]
fn run_without_strategy_uses_first_detected() {
    // auth-service only has docker, so it should auto-select docker/build
    moldx()
        .args(["build", module("auth-service").to_str().unwrap()])
        .assert()
        .success()
        .stdout(contains("docker/build"));
}

#[test]
fn run_without_strategy_fails_for_unknown_command() {
    moldx()
        .args(["nonexistent_cmd", module("auth-service").to_str().unwrap()])
        .assert()
        .failure()
        .stderr(contains("not found"));
}

#[test]
fn run_docker_build_succeeds() {
    moldx()
        .args(["docker", "build", module("auth-service").to_str().unwrap()])
        .assert()
        .success()
        .stdout(contains("docker/build"));
}

#[test]
fn run_node_test_succeeds() {
    moldx()
        .args(["node", "test", module("api-server").to_str().unwrap()])
        .assert()
        .success()
        .stdout(contains("node/test"));
}

#[test]
fn run_rust_build_succeeds() {
    moldx()
        .args(["rust", "build", module("worker").to_str().unwrap()])
        .assert()
        .success()
        .stdout(contains("rust/build"));
}

#[test]
fn run_command_on_multi_strategy_module() {
    moldx()
        .args(["docker", "logs", module("multi-strategy").to_str().unwrap()])
        .assert()
        .success()
        .stdout(contains("docker/logs"));
}

// ── validation failures ───────────────────────────────────────────────────────

#[test]
fn run_fails_for_unavailable_strategy() {
    // auth-service has only docker; node should be rejected
    moldx()
        .args(["node", "build", module("auth-service").to_str().unwrap()])
        .assert()
        .failure()
        .stderr(contains("not available"));
}

#[test]
fn run_fails_for_unknown_command() {
    moldx()
        .args([
            "docker",
            "nonexistent_cmd",
            module("auth-service").to_str().unwrap(),
        ])
        .assert()
        .failure()
        .stderr(contains("variant"));
}

#[test]
fn run_fails_for_nonexistent_path() {
    moldx()
        .args(["docker", "build", "/nonexistent/path/to/module"])
        .assert()
        .failure()
        .stderr(contains("does not exist"));
}

#[test]
fn run_fails_with_too_few_arguments() {
    moldx()
        .args(["docker", "build"]) // missing path
        .assert()
        .failure();
}

#[test]
fn run_agnostic_command_without_strategy_succeeds() {
    moldx()
        .args(["diff", module("auth-service").to_str().unwrap()])
        .assert()
        .success()
        .stdout(contains("agnostic/diff"));
}

// ── help ──────────────────────────────────────────────────────────────────────

#[test]
fn help_flag_exits_successfully() {
    moldx().arg("--help").assert().success();
}

#[test]
fn detect_help_shows_subcommand() {
    moldx().args(["detect", "--help"]).assert().success();
}
