# MoldX

MoldX is a convention-based CLI for discovering project modules and running technology-specific workflows across them.

It detects the technologies and conventions present in a project, associates modules with **profiles**, and resolves the commands available for each module. Shell scripts serve as the default execution contract, keeping MoldX simple, portable, and easy to extend.

MoldX was created to solve a common problem in heterogeneous projects and monorepos: as modules adopt different languages, frameworks, and deployment strategies, project automation often grows into an increasingly complex collection of Makefiles, Bash scripts, and custom logic for determining which workflows apply to which modules.

MoldX turns this implicit project knowledge into a conventional, discoverable, and automatable structure.

## Why MoldX?

Modern projects are rarely built around a single technology. A repository may contain Node.js applications, Rust services, Python workers, Docker configurations, and framework-specific projects side by side.

Each module may require different workflows:

- a Rust service can be built and tested with Cargo;
- a Node.js application can expose build and test commands;
- a Nuxt application can additionally provide development and production commands;
- a Docker-enabled module can be built, tagged, and deployed.

Traditional automation tools typically centralize this knowledge in configuration files, Makefiles, or custom scripts. As projects grow, these abstractions often become increasingly difficult to maintain and require explicit logic to determine which commands apply to each module.

MoldX takes a different approach: **the structure of the project becomes the configuration**.

Modules are discovered from the filesystem and associated with profiles based on conventions. Commands are then resolved dynamically from the profiles matched by each module.

A module is not limited to a single profile. For example, a service containing both a `package.json` and a `Dockerfile` can simultaneously expose Node.js and Docker workflows:

```text
packages/api/
├── package.json
├── Dockerfile
└── ...
```

```bash
moldx test packages/api
moldx docker build packages/api
moldx docker deploy packages/api
```

This allows project automation to remain decentralized and technology-specific while still providing a consistent interface across the entire repository.

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

### `moldx [OPTIONS...] [PROFILE...] <COMMAND> [MODULE] [-- <COMMAND_OPTIONS>...]`

Runs a command against one or more modules.
For convenience, the module parameter is optional, allowing MoldX to support commands that are not tied to a specific module.

#### Profiles

Profiles can be specified to explicitly select the profile from which the command
should be resolved.

```bash
moldx docker build packages/server
moldx python uv build packages/worker
```

Profiles can be omitted. When a command is not qualified by a profile, MoldX
resolves it from the profiles matching the target module.

If multiple matching profiles provide a command with the requested name, MoldX
prompts the user to resolve the conflict.

```bash
moldx build packages/server

# STDIN
# -> python/uv/build
# -> python/pip/build
# -> docker/build
```

Command conflicts can be skipped using the `--skip-conflicts` option, avoiding
manual input.

#### Multiple Modules

Commands can target multiple modules using glob patterns.

```bash
moldx install packages/*
```

The `*` pattern is expanded by the shell and allows commands to target multiple modules at the same directory level.

MoldX also supports the `**` pattern as a recursive module glob. Unlike standard shell glob expansion, MoldX interprets `**` itself, allowing recursive module matching independently of the shell's globstar configuration.

```bash
# Match modules directly under packages/
moldx install packages/*

# Recursively match modules under packages/
moldx install packages/**
```

For each matching module, MoldX resolves the requested command independently from the profiles associated with that module.

#### Command Options

Arguments following `--` are forwarded unchanged to the resolved command.

Positional command arguments are only supported after `--`.

```bash
moldx docker build packages/server -- --platform linux/amd64 --push
```

In this example, `--platform linux/amd64 --push` are passed directly to the
resolved `docker/build` command.

### `moldx [-- <OPTIONS>...] init <ENTITY> <PROFILE...> [ARGS...]`

- `moldx init` => creates `.moldx/README.md`, `.moldx/bin/.keep`, and `.moldx/profiles/.keep`
- `moldx init profile <...profile>` => creates `.moldx/profiles/<profile>/bin` and `.moldx/profiles/<profile>/template` (supports nested profiles)
- `moldx init command [...profile] <command>` => creates `.moldx/profiles/<profile>/bin/<command>` (supports nested profiles)
- `moldx init template [...profile] [...file_names]` => creates `.moldx/profiles/<profile>/template/<...file_names>` (supports nested profiles)

Creating MoldX entities through the `init` command allows MoldX to validate input, enforce constraints, and prevent undefined or invalid configurations.

### `moldx status`

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
| --- | --- | --- | --- |
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
cargo run -- status
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