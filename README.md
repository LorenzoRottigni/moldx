# moldx

Technology-agnostic orchestration for monorepos, powered by shell scripts.

**moldx** treats strategies as isolated, composable directories under `.moldx/strategies/`. Each strategy can expose command scripts, matching templates for discovery, or both. Strategies are automatically detected for any module matching their templates, enabling transparent command availability.

## Features

- 🚀 **Transparent Command Discovery** — Commands from matching strategies are automatically available
- 📦 **Template-Based Matching** — Strategies match modules by file presence (e.g., `Dockerfile` for Docker strategy)
- 🎯 **Agnostic Strategies** — Strategies with no templates are available to all modules
- 🖥️ **Interactive TUI** — Run commands, monitor output, and manage modules in real-time
- 🔧 **Multi-Strategy** — Chain strategies for complex workflows
- 📋 **Scaffolding** — Generate new modules, strategies, and templates from templates

## Quick Start

### Installation

```bash
curl -fsSL https://raw.githubusercontent.com/LorenzoRottigni/moldx/main/install.sh | bash
```

### Basic Setup

```bash
# 1. Create the strategy tree
mkdir -p .moldx/strategies/docker/template
mkdir -p .moldx/strategies/docker/bin
mkdir -p .moldx/strategies/default/bin

# 2. Add a template (for strategy detection)
cat > .moldx/strategies/docker/template/Dockerfile <<'EOF'
FROM alpine:latest
EOF

# 3. Add strategy commands
cat > .moldx/strategies/docker/bin/build.sh <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
echo "[moldx] docker/build -> $1"
docker build -t myapp .
EOF

# 4. Add agnostic commands (available to all modules)
cat > .moldx/strategies/default/bin/info.sh <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
echo "[moldx] info -> $1"
pwd && ls -la
EOF

# 5. Run moldx
moldx               # Interactive TUI mode
moldx detect ./services/auth
moldx list
moldx run docker build ./services/auth
moldx new module docker ./services/new-service
```

## Commands

| Command | Description |
|---------|-------------|
| `moldx` or `moldx ui` | Launch interactive terminal UI (default) |
| `moldx detect <path>` | Show strategies available for a module |
| `moldx list` | List all discovered modules in the workspace |
| `moldx run <strategy> <command> <path>` | Run a strategy command on a module |
| `moldx new <type> [options]` | Scaffold a new strategy, template, module, or command |
| `moldx init` | Initialize .moldx directory in current project |

## Concepts

| Term | Meaning |
|------|---------|
| **Module** | A directory matching at least one strategy template (has discoverable commands) |
| **Strategy** | A named directory under `.moldx/strategies/` (e.g., `docker`, `node`, `rust`) |
| **Template** | A file pattern used for strategy matching and new module scaffolding |
| **Command** | A shell script exposed by a strategy, usually `<strategy>/bin/<command>.sh` |
| **Agnostic Strategy** | A strategy with no templates; its commands are available to all modules |

## Configuration

### Project Layout

```text
<project-root>/
  .moldx/
    strategies/
      docker/
        bin/
          build.sh
          deploy.sh
          logs.sh
        template/
          Dockerfile
      node/
        bin/
          build.sh
          test.sh
        template/
          package.json
      default/
        bin/
          info.sh
          validate.sh
        template/
          .gitkeep   # Empty template makes strategy agnostic
  services/
    auth/
      Dockerfile
      src/
    api/
      package.json
      src/
```

### Template Layouts

Strategies can use either layout:

- Single template: `.moldx/strategies/<strategy>/template/`
- Multiple templates: `.moldx/strategies/<strategy>/templates/<template-name>/`

### Environment Variables

- `MOLDX_MOLDX_DIR` — Override `.moldx` directory location (default: `.moldx`)
- `MOLDX_STRATEGIES_DIR_NAME` — Strategies subdirectory name (default: `strategies`)
- `MOLDX_BIN_DIR_NAME` — Commands subdirectory name (default: `bin`)
- `MOLDX_TEMPLATE_DIR_NAME` — Single template directory name (default: `template`)
- `MOLDX_TEMPLATES_DIR_NAME` — Multiple templates directory name (default: `templates`)

## Interactive TUI

The default mode launches an interactive terminal UI for:

- **Left Panel** — Browse all modules in the workspace
- **Middle Panel** — View available commands for selected module
- **Right Panel** — Monitor running processes and their output

Navigate with arrow keys:
- `↓` / `↑` — Move between items
- `→` / `←` — Switch panels
- `Enter` — Execute selected command
- `k` — Kill running process
- `q` / `Ctrl+C` — Exit

## Architecture

moldx v2 is built on a clean, modular architecture:

- **Executor** — Shared execution runtime managing process lifecycle and I/O streaming
- **Client** — Orchestrates strategy/module discovery and command resolution
- **TUI** — Terminal UI leveraging the shared executor for real-time feedback
- **CLI** — Command-line interface for scripting and automation

All components share a single `Executor` instance (managed via `Arc`), ensuring consistent process management and state tracking without unnecessary cloning.

## Project Structure

```
.
├── src/              # v2 implementation (main)
│   ├── main.rs       # CLI entrypoint
│   ├── client.rs     # Core orchestration
│   ├── executor.rs   # Shared execution runtime
│   ├── tui.rs        # Terminal UI
│   ├── cli/          # CLI commands
│   └── ...           # Supporting modules
├── v1/               # Legacy v1 implementation (archived)
├── playground/       # Example strategies and modules
└── tests/            # Integration tests
```

## Migrating from v1

The v1 codebase is preserved in the [`v1/`](./v1/) directory. The v2 refactor focuses on:

- ✅ Unified execution model (shared `Executor` instead of duplicated state)
- ✅ Cleaner separation of concerns (CLI, TUI, discovery, execution)
- ✅ Better error handling and diagnostics
- ✅ Real-time output streaming in TUI
- ✅ Improved module and template matching logic

## Development

```bash
# Build
cargo build --release

# Test
cargo test

# Check
cargo check

# Run in dev mode
cargo run -- detect ./services/auth

# With playground
MOLDX_MOLDX_DIR=$PWD/playground/.moldx cargo run -- detect playground/modules/auth-service
```

## Playground

The `playground/` directory contains working examples:

```bash
MOLDX_DIR=$PWD/playground/.moldx cargo run -- detect playground/modules/auth-service
MOLDX_DIR=$PWD/playground/.moldx cargo run -- list
MOLDX_DIR=$PWD/playground/.moldx cargo run -- ui
```

## License

MIT

## Contributing

Contributions welcome! Please open an issue or submit a PR.
