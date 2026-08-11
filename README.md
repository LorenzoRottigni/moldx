# moldx

Technology-agnostic orchestration for monorepos, powered by shell scripts.

moldx now treats strategies as isolated plug-and-play directories under
`.moldx/strategies/`. Each strategy can expose command scripts, one or more
matching templates, or both.

## Install

```bash
curl -fsSL https://raw.githubusercontent.com/LorenzoRottigni/moldx/main/install.sh | bash
```

## Quick Start

```bash
# 1. Create the strategy tree
mkdir -p .moldx/strategies/docker/template
mkdir -p .moldx/strategies/default/template
mkdir -p .moldx/strategies/docker/bin
mkdir -p .moldx/strategies/default/bin

# 2. Add a matching template
cat > .moldx/strategies/docker/template/Dockerfile <<'EOF'
FROM scratch
EOF

# 3. Add commands for the strategy
cat > .moldx/strategies/docker/bin/build.sh <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
echo "[moldx] docker/build -> $1"
EOF

# 4. Add an agnostic command
cat > .moldx/strategies/default/bin/diff.sh <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
echo "[moldx] agnostic/diff -> $1"
EOF

# 5. Run moldx
moldx detect ./services/auth
moldx build ./services/auth
moldx diff ./services/auth
moldx new module docker ./services/new-auth
```

## Concepts

| Term | Meaning |
|------|---------|
| Module | A directory that matches at least one strategy template and has at least one runnable command |
| Strategy | A named directory under `.moldx/strategies/` such as `docker`, `node`, or `rust` |
| Template | A file tree used both for matching and for scaffolding new modules |
| Command | A shell script exposed by a strategy, usually `.<strategy>/bin/<command>.sh` |
| Agnostic strategy | Any strategy with no non-empty templates; its commands are available to every path |

Template matching is filename-based. A template matches when all of its
filenames exist directly in the target path. Empty template directories do not
match anything and make the strategy agnostic.

## Configuration

Recommended project layout:

```text
<project-root>/
  .moldx/
    strategies/
      docker/
        bin/
          build.sh
          deploy.sh
          logs.sh
          start.sh
          stop.sh
        template/
          Dockerfile
      node/
        bin/
          build.sh
          install.sh
          start.sh
          test.sh
        template/
          package.json
      default/
        bin/
          diff.sh
        template/
          .gitkeep   # optional placeholder; hidden files are ignored for matching
```

Supported template layouts:

- `.moldx/strategies/<strategy>/template/`
- `.moldx/strategies/<strategy>/templates/<template-name>/`

Supported command layouts:

- `.moldx/strategies/<strategy>/bin/<command>.sh`

If a strategy has no non-empty templates, it is treated as agnostic and its
commands are offered to every path.

## CLI Reference

### Global options

| Flag | Env var | Notes |
|------|---------|-------|
| `--moldx-dir <dir>` | `MOLDX_DIR` | Override the `.moldx/` root |
| `--strategies-dir <dir>` | `MOLDX_STRATEGIES_DIR` | Override `.moldx/strategies/` |
| `--bin-dir <dir>` | `MOLDX_BIN_DIR` | Compatibility alias for `--strategies-dir` |

### Commands

#### `moldx [strategy] <command> <path>`

Run a command against a module path.

Resolution order:

1. If a strategy is provided explicitly, moldx uses that strategy.
2. Otherwise, moldx finds strategies whose templates match the target path.
3. If no strategy-specific command matches, moldx falls back to agnostic commands.

Examples:

```bash
moldx docker build ./services/auth
moldx build ./services/auth
moldx diff ./services/auth
```

#### `moldx detect <path>`

Print the strategies whose templates match the path.

```bash
moldx detect ./services/auth
```

#### `moldx list [path] [--depth <n>]`

Walk a tree and list all discovered modules, their strategies, and the commands
available for each one.

```bash
moldx list ./services
moldx list --depth 5
```

#### `moldx new module <strategy> [template] <path>`

Scaffold a new module from a strategy template.

- If the strategy exposes exactly one non-empty template, `template` can be omitted.
- If the strategy exposes multiple templates, pick one explicitly.
- Existing non-empty target directories are rejected to avoid accidental overwrite.

```bash
moldx new module docker ./services/new-auth
moldx new module docker api ./services/api
```

#### `moldx`

Launch the interactive terminal UI.

## How It Works

### Detection

moldx resolves `.moldx/` starting from the current path or the configured
override, then loads `.moldx/strategies/`.

For a target path, moldx collects the direct filenames present in that path and
checks them against each strategy template. A strategy is considered matched if
any of its non-empty templates is a filename subset of the target path.

### Command execution

For `moldx [strategy] <command> <path>`:

1. Validate the command and strategy names.
2. Resolve the strategies directory.
3. Match the target path against strategy templates.
4. Choose the requested strategy, the best matching strategy, or an agnostic command.
5. Execute the resulting shell script with inherited stdio.

### Scaffolding

`moldx new module ...` copies the selected template directory into the target
path, preserving nested files.

## User Interface

`moldx` opens a three-panel TUI:

- Modules
- Commands for the selected module
- Running processes and their output

Keys:

- `Tab` / `Shift+Tab` cycle focus
- `Up` / `Down` move the selection
- `Enter` selects a module or runs a command
- `k` kills a running process
- `r` refreshes the module scan
- `q` or `Ctrl+C` quits

## Playground

The `playground/` directory contains a working example of the new layout:

- `playground/.moldx/strategies/docker/`
- `playground/.moldx/strategies/node/`
- `playground/.moldx/strategies/rust/`
- `playground/.moldx/strategies/default/`

Run the example with:

```bash
MOLDX_DIR=$PWD/playground/.moldx cargo run -- detect playground/modules/auth-service
MOLDX_DIR=$PWD/playground/.moldx cargo run -- build playground/modules/auth-service
MOLDX_DIR=$PWD/playground/.moldx cargo run -- new module docker playground/modules/scaffolded
```

## Development

```bash
cargo test
cargo clippy -- -D warnings
cargo fmt
```
