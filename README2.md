# MoldX

Technology-agnostic CLI tool that brings convention, structure, and automation to project workflows.

MoldX detects the types of code present in a project, associates project modules with **profiles**, and allows commands to be executed or scaffolded for those modules. Shell scripts serve as the default execution contract, keeping MoldX simple, portable, and easy to extend.

MoldX was created to solve a real-world problem encountered in a monorepo containing multiple modules written in different languages and technologies. The project was managed through a Makefile and an increasingly large collection of Bash scripts, with custom logic required to determine which commands and workflows applied to each module.

MoldX aims to transform this implicit project knowledge into a conventional, discoverable, and automatable project structure.

## Getting Started

### Installation

```bash
curl -fsSL https://raw.githubusercontent.com/LorenzoRottigni/moldx/main/install.sh | bash
```

> **Note:** Installation through package managers for common Linux distributions is planned.

### Initialize a project

From the root of a Git repository:

```bash
moldx init
```

This creates the `.moldx` directory used by MoldX to store profiles, templates, and commands.

### Example Project Structure

```text
project/
├── .moldx/
│   ├── bin/ # moldx <command> <target>
│   │   ├── diff.sh
│   │   └── version.sh
│   │
│   └── profiles/
│       ├── docker/
│       │   ├── template/ # matches modules containing a Dockerfile
│       │   │   └── Dockerfile
│       │   └── bin/ # moldx docker <command> <target>
│       │       ├── build.sh
│       │       ├── run.sh
│       │       ├── push.sh
│       │       ├── tag.sh
│       │       └── deploy.sh
│       │
│       ├── rust/
│       │   ├── template/ # matches modules containing a Cargo.toml
│       │   │   └── Cargo.toml
│       │   └── bin/ # moldx rust <command> <target>
│       │       ├── build.sh
│       │       ├── test.sh
│       │       └── run.sh
│       │
│       ├── node/
│       │   ├── template/ # matches modules containing a package.json
│       │   │   └── package.json
│       │   ├── bin/ # moldx node <command> <target>
│       │   │   ├── build.sh
│       │   │   └── test.sh
│       │   └── profiles/
│       │       ├── nuxt/
│       │       │   ├── template/ # matches modules containing package.json and nuxt.config.ts
│       │       │   │   ├── package.json
│       │       │   │   └── nuxt.config.ts
│       │       │   └── bin/ # moldx node nuxt <command> <target>
│       │       │       ├── dev.sh
│       │       │       └── start.sh
│       │       │
│       │       └── next/
│       │           ├── template/ # matches modules containing package.json and next.config.ts
│       │           │   ├── package.json
│       │           │   └── next.config.ts
│       │           └── bin/ # moldx node next <command> <target>
│       │               ├── dev.sh
│       │               └── start.sh
│       │
│       └── python/
│           ├── template/ # matches any module
│           ├── bin/ # moldx python <command> <target>
│           │   ├── lint.sh
│           │   └── test.sh
│           │
│           └── profiles/
│               ├── pip/
│               │   ├── template/ # matches modules containing requirements.txt
│               │   │   └── requirements.txt
│               │   └── bin/ # moldx python pip <command> <target>
│               │       ├── install.sh
│               │       ├── build.sh
│               │       └── run.sh
│               │
│               └── uv/
│                   ├── template/ # matches modules containing pyproject.toml
│                   │   └── pyproject.toml
│                   └── bin/ # moldx python uv <command> <target>
│                       ├── install.sh
│                       ├── build.sh
│                       └── run.sh
│
└── packages/
```

## Glossary

### Profile

A **profile** is a collection of commands related to a specific **technology** that can be applied to a particular type of **module**.

Profiles can be nested to provide more specific implementations for sub-technologies (for example, `node > nuxt`, `node > next`, or `python > pip`, `python > uv`).

Profiles are associated with modules through **templates**. A parent profile's template must be compatible with the templates of its child profiles.

### Command

A **command** is an executable workflow managed by MoldX.

Commands are typically implemented as shell scripts and may belong to a specific profile or be profile-agnostic.

For example:

```text
.moldx/profiles/docker/bin/build.sh
```

defines the `build` command for the `docker` profile.

### Template

A **template** describes the files that identify a module as belonging to a profile.

For example:

```text
docker/
├── Dockerfile
└── compose.yml
```

A Docker template containing these files can be used to identify Docker modules.

Templates are not necessarily scaffolding templates in the traditional sense. Their primary purpose is to define conventions used for module detection.

### Module

A **module** is a project directory that can be targeted by MoldX commands.

A module becomes associated with one or more profiles when it matches their templates.

A module may match multiple profiles.

For example:

```text
packages/my-service/
├── package.json
├── Dockerfile
└── ...
```

could potentially match both the `node` and `docker` profiles.

### Executor

An **executor** defines how a MoldX command is ultimately executed.

Shell is the default execution mechanism. Additional executors are planned for future versions.

## CLI

### `moldx [...profile] <command> <module>`

Runs a command against a module.

Examples:

```bash
moldx docker build packages/server

moldx python uv build packages/worker
```

Profiles can be omitted. If multiple matching profiles provide a command with the requested name, MoldX prompts the user to select which command to execute.

```bash
moldx build packages/server

# STDIN
# -> python/uv/build
# -> python/pip/build
# -> docker/build
```

It is also possible to target multiple modules:

```bash
moldx install packages/*

# Recursively
moldx install packages/**
```

For each profile matching a module, MoldX attempts to resolve the requested command. If multiple commands are available for a module, the user is prompted recursively to resolve the conflict.

Command conflicts can be skipped using the `--skip-conflicts` flag, avoiding manual input.

### moldx init

- `moldx init` => creates `.moldx/README.md`, `.moldx/bin/.keep`, and `.moldx/profiles/.keep`
- `moldx init profile <...profile>` => creates `.moldx/profiles/<profile>/bin` and `.moldx/profiles/<profile>/template` (supports nested profiles)
- `moldx init command [...profile] <command>` => creates `.moldx/profiles/<profile>/bin/<command>` (supports nested profiles)
- `moldx init template [...profile] [...file_names]` => creates `.moldx/profiles/<profile>/template/<...file_names>` (supports nested profiles)

Creating MoldX entities through the `init` command allows MoldX to validate input, enforce constraints, and prevent undefined or invalid configurations.

### moldx status

Prints the state of the MoldX client after initialization, including available profiles, commands, templates, and resolved modules.

### moldx

Runs the MoldX TUI in the current working directory.

## Guidelines

### `.moldx` Directory

MoldX requires a `.moldx` directory to be configured in order to operate.

MoldX is designed to work inside a Git repository, and the `.moldx` directory should normally be committed to version control.

The `.moldx` path can also be supplied explicitly through a command-line argument or environment variable, allowing MoldX to operate outside a Git repository.

By convention, MoldX expects `.moldx` to be located at the root of the repository.

If it is not found there, MoldX can search the Git workspace within the configured maximum resolution depth.

Once the `.moldx` directory has been resolved, MoldX commands can be invoked from anywhere within the Git workspace.

### Module Resolution

MoldX resolves modules relative to the parent directory of the resolved `.moldx` directory.

Module discovery is recursive and limited by the configured maximum resolution depth.

This prevents MoldX from unnecessarily traversing the entire filesystem while still supporting common monorepo layouts.

## Configuration

MoldX is designed to be **configuration-light**.

Project behavior is primarily inferred from the `.moldx` directory structure rather than from a central configuration file.

Global path-resolution and naming behavior can be customized through command-line arguments or environment variables.

| Environment variable | CLI option | Default | Description |
|---|---|---|---|
| `MOLDX_DIR` | `--moldx-dir` | `./.moldx` | Path to the MoldX directory |
| `MOLDX_PROFILES_DIR_NAME` | `--profiles-dir-name` | `profiles` | Profiles directory name |
| `MOLDX_BIN_DIR_NAME` | `--bin-dir-name` | `bin` | Commands directory name inside a profile |
| `MOLDX_TEMPLATE_DIR_NAME` | `--template-dir-name` | `template` | Template directory naming convention |
| `MOLDX_MAX_RESOLUTION_DEPTH` | `--max-resolution-depth` | implementation-defined | Maximum recursion depth for `.moldx` and module resolution |

## TUI

MoldX provides a terminal user interface capable of exposing MoldX's CLI functionality interactively.

The TUI is intended to make common operations easier to discover, particularly when:

- multiple profiles match a module;
- multiple commands are available;
- users want to browse available modules;
- users do not remember the exact CLI syntax.

Future versions may allow the TUI to connect to the MoldX daemon.

## Roadmap

### Executor Support

Introduce executors for execution targets other than the default shell executor.

Potential examples:

```text
executors/
├── shell.sh
└── python.sh
```

An executor could wrap another runtime while preserving a common command interface.

A formal argument contract will be required to ensure that commands implemented using different executors receive parameters consistently.

### Daemon

Add an optional MoldX daemon for long-running project interaction.

The proposed model is:

```text
              ┌─────────────┐
CLI ─────────►│             │
              │ MoldX daemon│
TUI ─────────►│             │
              │             │
VS Code ─────►│             │
              └─────────────┘
```

When a daemon is running, CLI commands and the TUI can connect to it.

When no daemon is running, CLI commands should continue to work directly, while the TUI may optionally start one.

The daemon should primarily support stateful integrations and long-lived clients rather than being required for normal CLI execution.

### VS Code Integration

Create a VS Code extension capable of connecting to the MoldX daemon.

Potential functionality includes:

- browsing modules;
- browsing profiles;
- browsing available commands;
- running commands;
- viewing command output;
- interacting with MoldX state.

### Improved `init`

Make `moldx init` capable of detecting existing project conventions and scaffolding useful defaults.

For example, detecting:

```text
package.json
```

could suggest or scaffold a Node.js profile with common commands such as:

```text
test
dev
build
start
```

Similarly, other project manifests could be used to detect additional technologies.

This should remain opt-in or reviewable so that `init` does not unexpectedly modify an existing project.

### Git Submodules

Explore using Git submodules to distribute reusable MoldX profiles outside individual monorepos.

This could allow teams to maintain shared profile collections independently from the projects consuming them.

## Contributing

Contributions are welcome.

MoldX is still evolving, and contributions around CLI design, module resolution, profile conventions, executors, the TUI, and integrations are especially valuable.

### Development

MoldX is written in Rust.

Typical development commands:

```bash
cargo build

cargo test

cargo run -- --help
```

See the repository's development documentation for the complete development workflow.

### Playground

MoldX includes a playground for local experimentation.

When running MoldX locally:

```bash
cargo run -- list
```

MoldX automatically resolves the playground's `.moldx` directory, making it possible to test profiles, templates, commands, and module resolution without modifying the development environment itself.

The playground should be treated as an integration-testing environment for MoldX itself.

### Testing

### Release

At minimum, the release process should cover:

1. updating the version;
2. running the test suite;
3. building release binaries;
4. generating release artifacts;
5. publishing the GitHub release;
6. updating installation artifacts.

### Versioning

MoldX should follow [Semantic Versioning](https://semver.org/).

In general:

- **MAJOR** versions may contain breaking changes;
- **MINOR** versions add backwards-compatible functionality;
- **PATCH** versions contain backwards-compatible fixes.

## License

MoldX is licensed under the MIT License.

See the `LICENSE` file for the complete license text.