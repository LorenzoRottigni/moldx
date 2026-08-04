# moldx

**Technology-agnostic orchestration engine** that standardizes submodule lifecycle
management through user-defined shell-based strategies.

Unlike Nx, Lerna, or Turborepo — which are JavaScript-first — moldx is built around
plain shell scripts and a project-specific detector, so it works equally well for
Rust services, Docker containers, Python packages, Go binaries, and anything else
that can be driven by a shell command.

---

## Table of Contents

1. [Quick Start](#quick-start)
2. [Concepts](#concepts)
3. [Configuration](#configuration)
4. [CLI Reference](#cli-reference)
5. [User Interfaces](#user-interfaces)
6. [How It Works — Behind the Scenes](#how-it-works--behind-the-scenes)
7. [Project Structure](#project-structure)
8. [Playground](#playground)
9. [Development](#development)

---

## Install

```bash
curl -fsSL https://raw.githubusercontent.com/LorenzoRottigni/moldx/main/install.sh | bash
```

## Quick Start

```bash
# 1. Build
cargo build --release

# 2. Bootstrap your project
mkdir -p .moldx/commands/build

# 3. Write detector.sh — prints one strategy name per line
cat > .moldx/detector.sh <<'EOF'
#!/usr/bin/env bash
TARGET="$1"
[ -f "$TARGET/Dockerfile" ]   && echo "docker"
[ -f "$TARGET/package.json" ] && echo "node"
[ -f "$TARGET/Cargo.toml" ]   && echo "rust"
EOF

# 4. Write a command script
cat > .moldx/commands/build/docker.sh <<'EOF'
#!/usr/bin/env bash
echo "Building $1..."
EOF

# 4b. Optional strategy-agnostic command
cat > .moldx/commands/diff.sh <<'EOF'
#!/usr/bin/env bash
echo "Diff for $1"
EOF

# 5. Run
moldx docker build ./services/my-service   # explicit strategy
moldx build ./services/my-service          # strategy auto-selected
moldx diff ./services/my-service           # strategy-agnostic command
```

---

## Concepts

| Term | Description |
|------|-------------|
| **Module** | Any directory in your project that `detector.sh` recognises as workable |
| **Strategy** | A named technology category (e.g. `docker`, `node`, `rust`). One module can have multiple strategies |
| **Command** | A named operation defined as a shell script under `.moldx/commands/` (e.g. `build`, `deploy`, `test`) |
| **Detector** | `.moldx/detector.sh` — receives an absolute module path as `$1`, prints zero or more strategy names |

A module is eligible for a strategy when `detector.sh` prints that strategy's name for
that path. A command is available when the corresponding `.sh` file exists inside the
command directory (either as agnostic command or strategy variant).

---

## Configuration

Create a `.moldx/` directory at the root of your project:

```
<project-root>/
  .moldx/
    detector.sh                  ← prints strategy names to stdout
    commands/
      <command>.sh               ← strategy-agnostic command
      <command>/
        <strategy>.sh            ← strategy-specific variant
```

### detector.sh contract

- Receives the **absolute path** of the module being tested as `$1`
- Prints **one strategy name per line** to stdout for every matching strategy
- A non-zero exit code is treated as "no strategies" (not an error)
- Must complete within **10 seconds** (configurable via `DETECTOR_TIMEOUT` in source)

```bash
#!/usr/bin/env bash
TARGET="$1"
[ -f "$TARGET/Dockerfile" ]   && echo "docker"
[ -f "$TARGET/package.json" ] && echo "node"
[ -f "$TARGET/Cargo.toml" ]   && echo "rust"
```

### Command script contract

- Located at one of:
  - `.moldx/commands/<command>.sh` (strategy-agnostic)
  - `.moldx/commands/<command>/<strategy>.sh` (strategy variant)
- Receives the **absolute module path** as `$1`
- Exit code is forwarded to the caller unchanged

```bash
#!/usr/bin/env bash
set -euo pipefail
MODULE_PATH="$1"
docker build -t my-image "$MODULE_PATH"
```

### Overrides

| Method | Overrides |
|--------|-----------|
| `--moldx-dir <path>` | Location of the `.moldx/` directory |
| `--commands-dir <path>` | Location of the `commands/` directory |
| `MOLDX_DIR=<path>` env var | Same as `--moldx-dir` |
| `MOLDX_COMMANDS_DIR=<path>` env var | Same as `--commands-dir` |

---

## CLI Reference

```
moldx [OPTIONS] <COMMAND>
```

### Global options

| Flag | Env var | Description |
|------|---------|-------------|
| `--moldx-dir <dir>` | `MOLDX_DIR` | Override `.moldx/` directory location |
| `--commands-dir <dir>` | `MOLDX_COMMANDS_DIR` | Override commands directory |

### Subcommands

#### `moldx [strategy] <command> <path>`

Run a command on a module. Strategy is optional; when omitted, moldx first tries
detected strategy variants for that command and then falls back to the agnostic
script.

```bash
moldx docker build ./services/auth      # explicit strategy
moldx build ./services/auth             # auto-detect
moldx k8s deploy ./services/api
moldx diff ./services/api               # agnostic
```

Validation sequence:
1. Path is canonicalized — error if it does not exist
2. `.moldx/` directory is located (walks up from the module path)
3. Strategy and command names are validated against path traversal
4. `detector.sh` is called to discover strategy variants for this module
5. Strategy resolution is performed (explicit strategy, then detected variants, then agnostic)
6. Script existence is checked — error with available variants listed
7. Script is executed with inherited stdio; exit code is forwarded

#### `moldx detect <path>`

Print all strategies detected for a module.

```bash
moldx detect ./services/auth
# Detected strategies for /abs/path/to/auth:
#   - docker
```

#### `moldx list [path] [--depth <n>]`

Discover and list all modules under a root directory (default depth: 3).

```bash
moldx list ./services
moldx list --depth 5
```

#### `moldx ui`

Launch the interactive terminal UI. Requires a `.moldx/` directory to be reachable
from the current working directory.

```bash
MOLDX_DIR=$PWD/.moldx moldx ui
```

---

## User Interfaces

### Terminal UI (`moldx ui`)

Three-panel layout driven by ratatui:

```
┌─ Modules ──────────────┬─ Commands ──────────────────┬─ Running ──────────────┐
│ > auth-service         │ [docker] build              │ #1 docker/build        │
│   api-server           │ [docker] deploy             │    PID 12345  Running  │
│   worker               │ [docker] start              ├────────────────────────┤
│                        │ [node]   install            │ stdout/stderr output   │
│                        │ [node]   test               │ of selected process    │
└────────────────────────┴─────────────────────────────┴────────────────────────┘
│ Tab: panel  ↑↓: nav  Enter: run/select  k: kill  r: refresh  q: quit          │
```

| Key | Action |
|-----|--------|
| `Tab` / `Shift+Tab` | Cycle focus between the three panels |
| `↑` / `↓` | Navigate the focused list |
| `Enter` | Select a module (Modules panel) or run a command (Commands panel) |
| `k` | Kill the selected running process |
| `r` | Re-scan modules in the background |
| `q` or `Ctrl+C` | Quit |


## How It Works — Behind the Scenes

### Config resolution

Every subcommand starts with a **config resolution** step. `MoldxConfig::resolve` walks
upward from the given path (similar to how `git` finds `.git/`) until it finds a
`.moldx/` directory. The resolved config carries four paths:

```
root            — directory containing .moldx/
moldx_dir       — .moldx/ itself
detector_path   — .moldx/detector.sh
commands_dir    — .moldx/commands/
```

If `MOLDX_DIR` is set, the upward walk is skipped entirely.

### Strategy detection

`detect_strategies(detector_path, target)` runs:

```bash
bash .moldx/detector.sh <absolute-target-path>
```

under a **10-second timeout**. Every non-empty trimmed line of stdout is returned as a
strategy name. A non-zero exit code returns an empty list without error — this allows
the detector to simply not print anything for paths it doesn't recognise.

### Module discovery (`moldx list` / UI scan)

`discover_modules` walks the directory tree with `walkdir` (respecting `max_depth`),
skips hidden directories and known build artefact folders (`target`, `node_modules`),
then calls `detect_strategies` on every candidate directory **in parallel** using a
`tokio::task::JoinSet` bounded by a `Semaphore(8)` — so at most 8 detector processes
run concurrently regardless of repo size.

```
walkdir entries → filter dirs → JoinSet (≤8 parallel) → detect_strategies
                                                         → if non-empty → Module
results sorted by path for deterministic output
```

### Command execution — two modes

**Foreground (`execute_blocking`)** — used by `moldx [strategy] <command> <path>`:

```
bash <resolved-script> <abs-module-path>
```

where `<resolved-script>` is either:
- `.moldx/commands/<command>/<strategy>.sh`
- `.moldx/commands/<command>.sh`

- stdio is **inherited** from moldx — output streams directly to the terminal
- the script's **exact exit code is forwarded** to the caller via `std::process::exit`

**Background (`run_and_track`)** — used by the TUI:

- the child bash process is placed in its **own process group** (`process_group(0)`)
  so that `kill -TERM -<pgid>` terminates the entire subprocess tree, not just bash
- stdout and stderr are captured line-by-line into the `AppState` output buffer
- process lifecycle transitions: `Running → Completed(code) | Failed(msg) | Killed`

### Process state machine

```
add_process()
     │
     ▼
  Running  ──── child exits 0 ────► Completed(i32)
     │
     ├──── child exits ≠ 0 ─────► Failed(String)
     │
     └──── kill_process() ──────► Killed
```

`AppState` is an `Arc<Mutex<Inner>>` so it can be cloned cheaply and shared across
the executor task and the TUI render loop — all running concurrently on the tokio
runtime.

### Output buffering

Each tracked process holds a `Vec<String>` of captured output lines.  
The buffer is capped at **500 lines** — the oldest line is dropped when the limit is
reached, so memory is bounded regardless of how verbose the command is.

### Security hardening

- **Path traversal prevention** — strategy and command names are validated before
  being used as path components. Names containing `/`, `\`, `.`, or `..` are
  rejected with an explicit error.
- **Shell injection** — all arguments are passed via `.arg()` to `tokio::process::Command`,
  never interpolated into a shell string, so paths with spaces or special characters
  are handled safely.

### TUI event loop

```
initial module scan (blocking eprintln + await)
         │
         ▼
   setup_terminal()
         │
         ▼
   loop {
     terminal.draw(|f| draw(f, &app))
     tokio::select! {
       tick (500 ms) → app.tick()       // check for finished background scan
       crossterm event → app.handle_key()  // keyboard input
     }
   }
         │
         ▼
   restore_terminal()
```

A panic hook restores the terminal before printing the panic message so the shell is
never left in raw/alternate-screen mode.

---

## Project Structure

```
moldx/
├── src/
│   ├── main.rs          Entry point and CLI dispatch
│   ├── cli.rs           Clap argument definitions
│   ├── config.rs        .moldx/ discovery and MoldxConfig
│   ├── detector.rs      detect_strategies + discover_modules
│   ├── executor.rs      execute_blocking + run_and_track
│   ├── state.rs         AppState (process registry)
│   └── ui/
│       ├── mod.rs       UI module
│       ├── tui.rs       Ratatui terminal UI
├── playground/
│   ├── .moldx/
│   │   ├── detector.sh
│   │   └── commands/
│   │       ├── build/   docker.sh node.sh rust.sh
│   │       ├── deploy/  docker.sh
│   │       ├── diff.sh  strategy-agnostic
│   │       ├── install/ node.sh
│   │       ├── logs/    docker.sh
│   │       ├── start/   docker.sh node.sh
│   │       ├── stop/    docker.sh
│   │       └── test/    node.sh rust.sh
│   └── modules/
│       ├── auth-service/   Dockerfile  → docker
│       ├── api-server/     package.json → node
│       ├── worker/         Cargo.toml  → rust
│       └── multi-strategy/ all three   → docker + node + rust
├── tests/
│   └── e2e.rs           End-to-end tests using assert_cmd
└── Cargo.toml
```

### Key dependencies

| Crate | Purpose |
|-------|---------|
| `clap` | CLI parsing with derive macros and env-var support |
| `tokio` | Async runtime for concurrent process execution |
| `ratatui` + `crossterm` | Terminal UI rendering and input |
| `walkdir` | Recursive directory traversal |
| `anyhow` | Ergonomic error propagation |
| `serde` + `serde_json` | JSON serialisation |

---

## Playground

The `playground/` directory is a fully-wired moldx project you can use immediately:

```bash
# Terminal UI
MOLDX_DIR=$PWD/playground/.moldx cargo run -- ui

# CLI commands
MOLDX_DIR=$PWD/playground/.moldx cargo run -- docker build playground/modules/auth-service
MOLDX_DIR=$PWD/playground/.moldx cargo run -- build playground/modules/auth-service
MOLDX_DIR=$PWD/playground/.moldx cargo run -- diff playground/modules/auth-service
MOLDX_DIR=$PWD/playground/.moldx cargo run -- list playground/
MOLDX_DIR=$PWD/playground/.moldx cargo run -- detect playground/modules/multi-strategy
```

---

## Development

```bash
# Run all tests (unit + E2E)
cargo test

# Run only unit tests
cargo test --lib

# Run only E2E tests
cargo test --test e2e

# Check for warnings
cargo clippy

# Release build
cargo build --release
```

### Test coverage

| Suite | Count | What it covers |
|-------|-------|----------------|
| Unit — `config` | 5 | `.moldx/` discovery, override precedence |
| Unit — `detector` | 8 | Script parsing, timeout, empty/error cases, command listing |
| Unit — `state` | 12 | Process lifecycle, output buffer bounds, Arc sharing |
| E2E | 18 | Full binary invocation: detect, list, run, error cases, optional strategy |

