# MoldX Playground

The playground is a **real-world microservices monorepo** used to exercise every
MoldX feature described in the root `README.md`. Every module is a real,
runnable application — not a placeholder — and every shell command delegates to
the actual technology toolchain when it is installed.

```text
playground/
├── .moldx/                  # MoldX project structure (the "convention")
│   ├── bin/                 # profile-agnostic commands (moldx <command>)
│   │   ├── diff.sh
│   │   └── info.sh
│   └── profiles/
│       ├── docker/
│       │   ├── template/Dockerfile
│       │   ├── bin/{build.sh,start.sh,stop.sh,logs.sh,deploy.sh}
│       │   └── profiles/postgres/
│       │       ├── template/{Dockerfile,init.sql}
│       │       └── bin/{seed.sh,status.sh}
│       ├── node/
│       │   ├── template/package.json
│       │   ├── bin/{install.sh,build.sh,test.sh,start.sh}
│       │   └── profiles/
│       │       ├── nuxt/   # template: package.json + nuxt.config.ts
│       │       └── vue/   # template: package.json + index.html
│       ├── python/
│       │   ├── template/__init__.py
│       │   ├── bin/{lint.sh,test.sh}
│       │   └── profiles/
│       │       ├── pip/    # template: __init__.py + requirements.txt
│       │       └── uv/     # template: __init__.py + pyproject.toml
│       └── rust/
│           ├── template/Cargo.toml
│           └── bin/{build.sh,test.sh,run.sh}
├── compose.yml              # orchestrate the services with Docker Compose
└── modules/
    ├── api-server/          # Node.js + Express REST API
    ├── frontend-nuxt/       # Nuxt 3 frontend
    ├── frontend-vue/      # Vue 3 + Vite frontend
    ├── backend-api/         # Python FastAPI service (managed with uv)
    ├── worker-py/           # Python background worker (managed with pip)
    ├── worker-rs/           # Rust worker
    ├── database/            # PostgreSQL (Docker + .env.example + admin.py)
    └── gateway/             # multi-profile: Node + Docker + Rust
```

## How modules are matched

MoldX detects a module by checking whether a directory contains **all** the
files of a profile's `template/`:

| module             | files it contains                                       | matching profiles                          |
| ------------------ | ------------------------------------------------------- | ------------------------------------------ |
| `api-server`       | `package.json`                                          | `node`                                     |
| `frontend-nuxt`    | `package.json`, `nuxt.config.ts`, `Dockerfile`          | `node`, `node > nuxt`, `docker`            |
| `frontend-vue`     | `package.json`, `index.html`, `Dockerfile`              | `node`, `node > vue`, `docker`             |
| `backend-api`      | `__init__.py`, `pyproject.toml`, `Dockerfile`           | `python`, `python > uv`, `docker`          |
| `worker-py`        | `__init__.py`, `requirements.txt`, `Dockerfile`         | `python`, `python > pip`, `docker`         |
| `worker-rs`        | `Cargo.toml`, `Dockerfile`                              | `rust`, `docker`                           |
| `database`         | `Dockerfile`, `init.sql`, `__init__.py`                 | `docker`, `docker > postgres`, `python`    |
| `gateway`          | `package.json`, `Cargo.toml`, `Dockerfile`              | `node`, `docker`, `rust`                   |

`gateway` is the canonical example of the README's "a module may match multiple
profiles": it exposes `node`, `docker`, AND `rust` workflows.

## Features covered

- **Module detection** — `moldx detect <path>` prints every profile a module matches.
- **List** — `moldx list` prints profiles (including nested), commands, and modules.
- **Profile-qualified commands** — `moldx docker build modules/database`.
- **Nested profile commands** — `moldx node nuxt dev modules/frontend-nuxt`,
  `moldx python uv run modules/backend-api`, `moldx docker postgres seed modules/database`.
- **Unqualified commands** — `moldx test modules/api-server` resolves `node/test`
  automatically from the module's matching profiles.
- **Conflict resolution** — `moldx build modules/gateway` finds `docker/build`,
  `node/build`, and `rust/build`; it prompts on a TTY and otherwise asks for
  `--skip-conflicts`:
  ```bash
  moldx --skip-conflicts build modules/gateway
  ```
- **Glob targets** — single-level `*` and recursive `**`:
  ```bash
  moldx test 'modules/*'    # immediate children
  moldx test 'modules/**'   # recursive
  ```
- **Command options** — everything after `--` is forwarded to the script:
  ```bash
  moldx docker build modules/database -- --platform linux/amd64
  ```
- **Profile-agnostic commands** — `.moldx/bin` defines commands usable on any
  module (the root profile is a catch-all):
  ```bash
  moldx info
  moldx diff modules/gateway
  ```

## Try it

From this directory:

```bash
# Inspect the project
moldx list
moldx detect modules/gateway

# Node.js workflows
moldx node install modules/api-server
moldx node test  modules/api-server
moldx build       modules/api-server        # auto-resolved to node/build

# Python workflows
moldx python uv run     modules/backend-api
moldx python pip run    modules/worker-py
moldx python uv build   modules/backend-api

# Rust workflows
moldx rust build modules/worker-rs
moldx rust test  modules/worker-rs

# Docker workflows (requires a running Docker daemon)
moldx docker build modules/database
moldx docker start modules/database
moldx docker postgres seed modules/database
moldx docker logs modules/database
moldx docker stop modules/database

# Or orchestrate everything with Docker Compose
docker compose up --build
```

The commands are deliberately **simple but real**: they check for the required
toolchain (`docker`, `npm`, `cargo`, `python3`, `uv`, `ruff`...) and run the real
tool when available. When a tool is missing they print a clear hint and stay
out of the way, which keeps the automated test-suite environment-friendly while
the same scripts execute real builds and tests on a development machine.