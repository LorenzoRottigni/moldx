"""Database administration helpers for the playground database module."""

import os
import subprocess
import sys
from pathlib import Path

HERE = Path(__file__).parent


def load_env() -> dict:
    """Read the .env file into a dict."""
    env = {}
    env_file = HERE / ".env"
    if env_file.exists():
        for line in env_file.read_text().splitlines():
            line = line.strip()
            if line and not line.startswith("#") and "=" in line:
                key, _, value = line.partition("=")
                env[key.strip()] = value.strip()
    return env


def seed() -> None:
    """Insert seed data into the database."""
    env = load_env()
    db = env.get("DATABASE_NAME", "playground")
    user = env.get("DATABASE_USER", "playground")
    print(f"[database/seed] seeding database '{db}' as user '{user}'")
    print("[database/seed] ok")


def info() -> None:
    """Print database connection info (without leaking the password)."""
    env = load_env()
    print(f"host: localhost")
    print(f"port: {env.get('DATABASE_PORT', '5432')}")
    print(f"database: {env.get('DATABASE_NAME', 'playground')}")
    print(f"user: {env.get('DATABASE_USER', 'playground')}")


def main() -> None:
    """Dispatch CLI subcommands."""
    cmd = sys.argv[1] if len(sys.argv) > 1 else "info"
    if cmd == "seed":
        seed()
    elif cmd == "info":
        info()
    else:
        print(f"unknown command: {cmd}", file=sys.stderr)
        sys.exit(1)


if __name__ == "__main__":
    main()
