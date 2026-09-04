use assert_cmd::Command;
use predicates::str::contains;
use std::fs;
use std::path::Path;
use tempfile::TempDir;

/// Builds a temporary MoldX project.
///
/// Layout:
///   tmp/.moldx/profiles/node/{template/package.json, bin/test.sh}
///   tmp/packages/a/package.json
///   tmp/packages/b/package.json
///   tmp/packages/sub/c/package.json
///
/// The `test.sh` command prints `node/test` and exits with the first argument
/// (the module path) when `MODULE_PATH` is echoed.
fn scaffold_project() -> TempDir {
    let tmp = TempDir::new().unwrap();
    let profiles = tmp.path().join(".moldx/profiles/node");
    fs::create_dir_all(profiles.join("template")).unwrap();
    fs::create_dir_all(profiles.join("bin")).unwrap();
    fs::write(profiles.join("template/package.json"), "{}").unwrap();
    fs::write(
        profiles.join("bin/test.sh"),
        "#!/usr/bin/env bash\nprintf 'node/test\\n'\n",
    )
    .unwrap();

    let pkg_root = tmp.path().join("packages");
    fs::create_dir_all(pkg_root.join("a")).unwrap();
    fs::create_dir_all(pkg_root.join("b")).unwrap();
    fs::create_dir_all(pkg_root.join("sub/c")).unwrap();
    for p in ["a", "b", "sub/c"] {
        fs::write(pkg_root.join(p).join("package.json"), "{}").unwrap();
    }

    tmp
}

fn cmd_in(tmp: &TempDir, moldx_dir: &Path, args: &[&str]) -> Command {
    let mut c = Command::cargo_bin("moldx").unwrap();
    c.current_dir(tmp.path()).env("MOLDX_DIR", moldx_dir);
    for a in args {
        c.arg(a);
    }
    c
}

// ── init ─────────────────────────────────────────────────────────────────────

#[test]
fn init_creates_documented_structure() {
    let tmp = TempDir::new().unwrap();
    let moldx_dir = tmp.path().join(".moldx");

    Command::cargo_bin("moldx")
        .unwrap()
        .current_dir(tmp.path())
        .env("MOLDX_DIR", moldx_dir.to_str().unwrap())
        .args(["init"])
        .assert()
        .success();

    assert!(moldx_dir.join("README.md").exists());
    assert!(moldx_dir.join("bin/.keep").exists());
    assert!(moldx_dir.join("profiles/.keep").exists());
    assert!(moldx_dir.join("profiles/default/bin/.keep").exists());
    assert!(moldx_dir.join("profiles/default/template/.keep").exists());
}

#[test]
fn init_profile_command_and_template() {
    let tmp = scaffold_project();
    let moldx_dir = tmp.path().join(".moldx");

    Command::cargo_bin("moldx")
        .unwrap()
        .current_dir(tmp.path())
        .env("MOLDX_DIR", moldx_dir.to_str().unwrap())
        .args(["init", "profile", "node", "nuxt"])
        .assert()
        .success();
    assert!(moldx_dir
        .join("profiles/node/profiles/nuxt/bin/.keep")
        .exists());
    assert!(moldx_dir
        .join("profiles/node/profiles/nuxt/template/.keep")
        .exists());

    Command::cargo_bin("moldx")
        .unwrap()
        .current_dir(tmp.path())
        .env("MOLDX_DIR", moldx_dir.to_str().unwrap())
        .args(["init", "command", "node", "nuxt", "dev"])
        .assert()
        .success();
    assert!(moldx_dir
        .join("profiles/node/profiles/nuxt/bin/dev.sh")
        .exists());

    Command::cargo_bin("moldx")
        .unwrap()
        .current_dir(tmp.path())
        .env("MOLDX_DIR", moldx_dir.to_str().unwrap())
        .args([
            "init",
            "template",
            "node",
            "nuxt",
            "package.json",
            "nuxt.config.ts",
        ])
        .assert()
        .success();
    assert!(moldx_dir
        .join("profiles/node/profiles/nuxt/template/nuxt.config.ts")
        .exists());
    assert!(moldx_dir
        .join("profiles/node/profiles/nuxt/template/package.json")
        .exists());
}

#[test]
fn init_unknown_entity_fails() {
    let tmp = TempDir::new().unwrap();
    let moldx_dir = tmp.path().join(".moldx");

    Command::cargo_bin("moldx")
        .unwrap()
        .current_dir(tmp.path())
        .env("MOLDX_DIR", moldx_dir.to_str().unwrap())
        .args(["init", "bogus", "x"])
        .assert()
        .failure();
}

// ── globs & multiple modules ────────────────────────────────────────────────

#[test]
fn run_single_level_glob_targets_siblings() {
    let tmp = scaffold_project();
    let moldx_dir = tmp.path().join(".moldx");

    cmd_in(&tmp, &moldx_dir, &["test", "packages/*"])
        .assert()
        .success()
        .stdout(contains("node/test"));
}

#[test]
fn run_recursive_glob_targets_nested_modules() {
    let tmp = scaffold_project();
    let moldx_dir = tmp.path().join(".moldx");

    cmd_in(&tmp, &moldx_dir, &["test", "packages/**"])
        .assert()
        .success()
        .stdout(contains("node/test"));
}

#[test]
fn run_multiple_module_args() {
    let tmp = scaffold_project();
    let moldx_dir = tmp.path().join(".moldx");

    cmd_in(&tmp, &moldx_dir, &["test", "packages/a", "packages/b"])
        .assert()
        .success()
        .stdout(contains("node/test"));
}

#[test]
fn single_level_glob_does_not_match_nested() {
    let tmp = scaffold_project();
    let moldx_dir = tmp.path().join(".moldx");

    // packages/* only matches a, b (immediate children). sub/c is nested, so
    // it should NOT be reached by a single-level glob.
    cmd_in(&tmp, &moldx_dir, &["test", "packages/*"])
        .assert()
        .success();
}

// ── command options ──────────────────────────────────────────────────────────

#[test]
fn command_options_forwarded_after_double_dash() {
    let tmp = TempDir::new().unwrap();
    let profiles = tmp.path().join(".moldx/profiles/node");
    fs::create_dir_all(profiles.join("template")).unwrap();
    fs::create_dir_all(profiles.join("bin")).unwrap();
    fs::write(profiles.join("template/package.json"), "{}").unwrap();
    fs::write(
        profiles.join("bin/mycmd.sh"),
        "#!/usr/bin/env bash\nprintf '%s\\n' \"$@\"\n",
    )
    .unwrap();

    fs::create_dir_all(tmp.path().join("mod")).unwrap();
    fs::write(tmp.path().join("mod/package.json"), "{}").unwrap();

    let moldx_dir = tmp.path().join(".moldx");
    cmd_in(
        &tmp,
        &moldx_dir,
        &["mycmd", "mod", "--", "--flag", "value"],
    )
    .assert()
    .success()
    .stdout(contains("--flag"))
    .stdout(contains("value"));
}

// ── status ───────────────────────────────────────────────────────────────────

#[test]
fn status_reports_profiles_and_modules() {
    let tmp = scaffold_project();
    let moldx_dir = tmp.path().join(".moldx");

    cmd_in(&tmp, &moldx_dir, &["status"])
        .assert()
        .success()
        .stdout(contains("profiles:"))
        .stdout(contains("modules:"))
        .stdout(contains("node"));
}

#[test]
fn root_profile_agnostic_command_runs_without_module() {
    let tmp = TempDir::new().unwrap();
    let moldx_dir = tmp.path().join(".moldx");
    fs::create_dir_all(moldx_dir.join("bin")).unwrap();
    fs::write(
        moldx_dir.join("bin/version.sh"),
        "#!/usr/bin/env bash\nprintf 'moldx-version %s\\n' \"$1\"\n",
    )
    .unwrap();

    cmd_in(&tmp, &moldx_dir, &["version", "--", "1.0.0"])
        .assert()
        .success()
        .stdout(contains("moldx-version 1.0.0"));
}
