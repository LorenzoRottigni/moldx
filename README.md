# MoldX

Technology-agnostic CLI tool that brings convention and automation to project workflows.

MoldX detects what kind of code lives where, associates project modules with **strategies**, and lets you run or scaffold commands for those modules. Shell scripts are the default execution contract, keeping MoldX simple, portable, and easy to extend.

MoldX was created to solve a real-world problem encountered in a monorepo containing multiple modules written in different languages and technologies. The project was managed through a Makefile and a growing collection of Bash scripts, with custom logic required to determine which commands and workflows applied to each module.

MoldX aims to turn that implicit project knowledge into a conventional, discoverable, and automatable project structure.

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

This creates the `.moldx` directory used by MoldX to store strategies, templates, and commands.

### Create a strategy

```bash
moldx new strategy docker
```

### Create a strategy template

```bash
moldx new template docker .docker/Dockerfile .docker/compose.yml
```

Any module containing the files represented by this template can then be associated with the `docker` strategy.

### Create a command

```bash
moldx new command docker build
```

This scaffolds:

```text
.moldx/
└── strategies/
    └── docker/
        └── bin/
            └── build.sh
```

The generated shell script becomes the implementation of the `build` command for the `docker` strategy.

### Run a command

```bash
moldx docker build packages/docker-module
```

If exactly one strategy associated with the target provides the requested command, the strategy can be omitted:

```bash
moldx build packages/docker-module
```

## Concepts

### Strategy

A **strategy** is a collection of commands applicable to a particular kind of target.

A strategy is identified through one or more templates. For example, a `docker` strategy might use Docker-related files to determine whether a module is a Docker module.

A strategy can provide commands such as:

```text
build
test
run
publish
```

The same command name can exist in multiple strategies while having different implementations.

### Command

A **command** is an executable workflow managed by MoldX.

Commands are normally implemented as shell scripts and may belong to a specific strategy or be strategy-agnostic.

For example:

```text
.moldx/strategies/docker/bin/build.sh
```

defines the `build` command for the `docker` strategy.

### Template

A **template** describes the files that identify a module as belonging to a strategy.

For example:

```text
.docker/
├── Dockerfile
└── compose.yml
```

A Docker template containing those files can be used to identify Docker modules.

Templates are not necessarily scaffolding templates in the traditional sense. They primarily act as a convention used for module detection.

### Module

A **module** is a project directory that can be targeted by MoldX commands.

A module becomes associated with one or more strategies when it matches their templates.

A module may match multiple strategies.

For example:

```text
packages/my-service/
├── package.json
├── Dockerfile
└── ...
```

could potentially match both a `node` strategy and a `docker` strategy.

### Target

A **target** is the path supplied to a MoldX command representing the module on which the command should operate.

For example:

```bash
moldx docker build packages/my-service
```

Here:

- `docker` is the strategy
- `build` is the command
- `packages/my-service` is the target

### Executor

An **executor** defines how a MoldX command is ultimately executed.

Shell is the default execution mechanism. Additional executors are planned for future versions.

## Project Structure

A typical MoldX project looks like this:

```text
project/
├── .moldx/
│   └── strategies/
│       ├── docker/
│       │   ├── bin/
│       │   │   ├── build.sh
│       │   │   └── run.sh
│       │   └── templates/
│       │       └── docker/
│       │           ├── Dockerfile
│       │           └── compose.yml
│       │
│       └── node/
│           ├── bin/
│           │   ├── build.sh
│           │   └── test.sh
│           └── templates/
│               └── node/
│                   └── package.json
│
└── packages/
    ├── api/
    └── web/
```

The exact directory layout is configurable through MoldX options and environment variables.

## Setup

### Initialize MoldX

Create the `.moldx` directory in the project:

```bash
moldx init
```

### Create a strategy

```bash
moldx new strategy docker
```

### Create a template

```bash
moldx new template docker .docker/Dockerfile .docker/compose.yml
```

A module containing the relevant template files can then be associated with the `docker` strategy.

### Create a strategy command

```bash
moldx new command docker build
```

MoldX scaffolds:

```text
.moldx/strategies/docker/bin/build.sh
```

An `--edit` option is planned to open the generated command using the editor configured through `$EDITOR`.

### Create a strategy-agnostic command

If no strategy is supplied, the command belongs to the default strategy-agnostic command set:

```bash
moldx new command clean
```

### Run a strategy command

```bash
moldx docker build packages/docker-module
```

### Run an unambiguous command

If only one strategy associated with the target provides the requested command, the strategy can be omitted:

```bash
moldx build packages/docker-module
```

If multiple strategies provide the same command, MoldX can require the strategy to be specified explicitly.

## CLI

### `moldx [strategy] <command> <target>`

Run a command against a target.

Examples:

```bash
moldx docker build packages/docker-module
moldx node test packages/api
```

If the command is unambiguous for the target:

```bash
moldx build packages/docker-module
```

### `moldx init`

Initialize MoldX in the current project.

```bash
moldx init
```

### `moldx new`

Create MoldX entities.

```bash
moldx new strategy <strategy>
moldx new template <strategy> [...template-files]
moldx new command [strategy] <command>
```

### `moldx list`

List available strategies, commands, templates, and/or resolved modules.

The exact output is subject to the CLI implementation.

### `moldx ui`

Start the MoldX terminal user interface.

```bash
moldx ui
```

## Guidelines

### `.moldx` Directory

MoldX requires a `.moldx` directory to be configured in order to operate.

MoldX is designed to work inside a Git repository, and the `.moldx` directory should normally be committed to version control.

The `.moldx` path can also be supplied explicitly through a command-line argument or environment variable, allowing MoldX to operate outside a Git repository.

By convention, MoldX expects `.moldx` at the root of the repository.

If it is not found there, MoldX can search the Git workspace within the configured maximum resolution depth.

Once the `.moldx` directory has been resolved, MoldX commands can be invoked from anywhere within the Git workspace.

### Module Resolution

MoldX resolves modules relative to the parent directory of the resolved `.moldx` directory.

Module discovery is recursive and limited by the configured maximum resolution depth.

This prevents MoldX from unnecessarily traversing the entire filesystem while still supporting common monorepo layouts.

### Multiple Strategies

A module may match more than one strategy.

For example, a module could simultaneously be:

- a Node.js module
- a Docker module
- a Terraform module

This allows strategies to represent independent capabilities rather than forcing every module into a single technology classification.

When a command exists in multiple matching strategies, MoldX should require the strategy to be specified explicitly or provide an interactive selection mechanism.

## Configuration

MoldX is designed to be **configuration-light**.

Project behavior is primarily inferred from the `.moldx` directory structure rather than from a central configuration file.

Global path-resolution and naming behavior can be customized through command-line arguments or environment variables.

| Environment variable | CLI option | Default | Description |
|---|---|---|---|
| `MOLDX_DIR` | `--moldx-dir` | `./.moldx` | Path to the MoldX directory |
| `MOLDX_STRATEGIES_DIR_NAME` | `--strategies-dir-name` | `strategies` | Strategies directory name |
| `MOLDX_BIN_DIR_NAME` | `--bin-dir-name` | `bin` | Commands directory name inside a strategy |
| `MOLDX_TEMPLATES_DIR_NAME` | `--templates-dir-name` | `templates` | Templates directory name inside a strategy |
| `MOLDX_TEMPLATE_DIR_NAME` | `--template-dir-name` | `template` | Template directory naming convention |
| `MOLDX_MAX_RESOLUTION_DEPTH` | `--max-resolution-depth` | implementation-defined | Maximum recursion depth for `.moldx` and module resolution |

## Shell Command Contract

Shell is the default execution contract for MoldX commands.

A generated command is a shell script that receives the target and command arguments according to the MoldX command contract.

For example:

```bash
#!/usr/bin/env bash

set -e

target="$1"

# command implementation
```

The exact argument contract should be considered part of the public MoldX CLI specification and documented as the executor model evolves.

## TUI

MoldX provides a terminal user interface capable of exposing MoldX's CLI functionality interactively.

The TUI is intended to make common operations easier to discover, particularly when:

- multiple strategies match a module;
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

A formal argument contract will be needed so that commands implemented using different executors receive parameters consistently.

### Interactive Command Selection

When a target matches multiple strategies that expose the same command, MoldX should be able to interactively ask the user which strategy to execute.

For example:

```bash
moldx build ./multi-strategy-target
```

If the target matches both `docker` and `node` strategies, MoldX could display:

```text
Which build strategy do you want to run?

> docker
  node
```

### Daemon

Add an optional MoldX daemon for long-running project interaction.

The proposed model is:

```text
             ┌─────────────┐
CLI ────────►│             │
             │ MoldX daemon│
TUI ────────►│             │
             │             │
VS Code ────►│             │
             └─────────────┘
```

When a daemon is running, CLI commands and the TUI can connect to it.

When no daemon is running, CLI commands should continue to work directly, while the TUI can optionally start one.

The daemon should primarily exist to support stateful integrations and long-lived clients rather than being required for normal CLI execution.

### VS Code Integration

Create a VS Code extension capable of connecting to the MoldX daemon.

Potential functionality includes:

- browsing modules;
- browsing strategies;
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

could suggest or scaffold a Node.js strategy with common commands such as:

```text
test
dev
build
start
```

Similarly, other project manifests could be used to detect additional technologies.

This should remain opt-in or reviewable so that `init` does not unexpectedly modify an existing project.

### Git Submodules

Explore using Git submodules to distribute reusable MoldX strategies outside individual monorepos.

This could allow teams to maintain shared strategy collections independently from the projects consuming them.

### Dev Containers

Explore integration with development containers.

Potential integrations include:

- detecting Dev Container configuration;
- providing a Dev Container strategy;
- running container-specific workflows;
- exposing MoldX commands inside the development environment.

## Contributing

Contributions are welcome.

MoldX is still evolving, and contributions around CLI design, module resolution, strategy conventions, executors, the TUI, and integrations are especially useful.

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

MoldX automatically resolves the playground's `.moldx` directory, making it possible to test strategies, templates, commands, and module resolution without modifying the development environment itself.

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
