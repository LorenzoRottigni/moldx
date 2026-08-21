# MoldX

Technology-agnostic CLI tool that brings convention and automation to project workflows — detecting what kind of code lives where and letting you run and scaffold tasks for it, with shell scripting as the default execution contract.

MoldX was created to solve a real-world problem encountered in a monorepo containing multiple modules written in different languages and technologies. The project was managed through a Makefile and a growing collection of Bash scripts, with custom logic needed to determine which commands and workflows applied to each module.

## Gettng Started

### Installation

```bash
curl -fsSL https://raw.githubusercontent.com/LorenzoRottigni/moldx/main/install.sh | bash
```

In the future, installation will also be available through package managers for common Linux distros.

## Glossary

### Strategy

Defines a suite of commands that work for specific targets, defined by its template(s). e.g. a Docker strategy would run docker-related commands on the target a Python likely provides commands to work exclusively with python modules.

### Command

A shell command managed by MoldX that might belong or not to a strategy.

### Template

A directory containing a bunch of files representing a target module, the idea is that if the target modules match the template files then the strategy's command is eligible to run.

### module

A project directory which is eligible to be targetted by MoldX commands.

### Setup

```bash
# create .moldx in the project
moldx init
# create a new strategy
moldx new <strategy>
# create a new template for the strategy
moldx new template <strategy> [...template-files]
# add a command to the strategy
modx new command [strategy] <command>
# run command for a module
moldx [strategy] <command> <target>
```

### Example

#### Init

Scaffold .moldx dir within the project:

```bash
moldx init
```

#### Strategy

Create a new strategy:

```bash
moldx new strategy docker
```

#### Strategy Template

Create a new template for docker strategy:

```bash
moldx new template docker .docker/Dockerfile .docker/compose.yml
```

Any module including .docker/Dockerfile and .docker/compose.yml will be associated to the docker strategy via template.

#### Strategy Command

Create a new command for building a docker module:

```bash
moldx new command docker build
```

Scaffolds a shell script at .moldx/strategies/docker/bin/build.sh

In the future, an --edit flag will be available to open the consumer editor configured at $EDITOR.

If strategy is not provided then command will belong to the default strategy target-agnostic

#### Run Strategy Command

Run the build for a Docker module:

```bash
moldx docker build packages/docker-module
```

If build command is defined within 1 strategy associated to that module then this is also allowed:

```bash
moldx build packages/docker-module
```

## Commands

### Moldx [strategy] [command] <target>

### Moldx init

### Moldx new <entity> [strategy] <target>

### Moldx list

### Moldx ui

## Architecture

What is expected where and what is allowed.

## Configuration

MoldX is designed to run configless, everything is inferred by the .moldx dir structure.

There are a few customization available for global ergonomics which can be provided as command flags or environment variables:

- env.MOLDX_DIR or --moldx-dir: specify where the .moldx dir is expected (default ./.moldx).
- env.MOLDX_STRATEGIES_DIR_NAME or --bin-dir-name: specify the name of the strategies dir contained by .moldx dir (default .moldx/strategies).
- env.MOLDX_BIN_DIR_NAME or --bin-dir-name: specify the name of the bin dir contained by strategy dir (default .moldx/strategies/<strategy>/bin).
- env.MOLDX_TEMPLATES_DIR_NAME or --templates-dir-name: specify the name of the templates dir contained by strategy dir (default .moldx/strategies/<strategy>/templates).
- env.MOLDX_TEMPLATE_DIR_NAME or --template-dir-name: specify the name of the template dir contained by strategy dir (default .moldx/strategies/<strategy>/template).
- env.MOLDX_MAX_RESOLUTION_DEPTH or --max-resolution-depth: max depth for .moldx and modules resolution recursion.

## TUI

MoldX provides a TUI able to wrap all the moldx CLI capabilities.

## Roadmap

Coming next

## Contributing

contributions are welcome

### Development

### Playground

### Release

### Versioning

## Licence

MIT
