# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Rust web framework with bidirectional Python interop via PyO3. Axum serves HTTP endpoints that can call Python code, and Rust structs/functions are exposed as a Python module via maturin.

## Build & Run Commands

```bash
# Prerequisites: Rust (edition 2024), Python 3.13 (exact), uv, Docker

# Start PostgreSQL
docker compose up -d

# Create venv and install Python deps
uv venv --python 3.13
source .venv/bin/activate
uv sync

# Build and run the server (listens on 127.0.0.1:3000)
cargo run

# Run Rust tests
cargo test

# Build Python native module (for calling Rust from Python)
maturin develop
python python/call_rust_from_py.py
```

## Environment Variables

- `DB_NAME`, `DB_USER`, `DB_PASSWORD`, `DB_HOST`, `DB_PORT` — PostgreSQL connection (docker-compose defaults: testdb/testdbuser/testdbpass/localhost/5432)
- `PYO3_PYTHON` — Python interpreter path (set in `.cargo/config.toml` to `.venv/bin/python`)
- `RUST_LOG` — Tracing filter level (default: "info")
- `PYTHONHOME` — Auto-set at startup from compile-time `PY_BASE_PREFIX` detected by `build.rs`

## Architecture

**Bidirectional Rust-Python interop:**
- **Rust → Python:** Axum endpoints use `Python::attach()` to call Python classes/functions in `python/`
- **Python → Rust:** PyO3 `#[pyclass]`/`#[pyfunction]` exports compiled via maturin as a native module

**Key layers:**
- `src/main.rs` — Axum router setup, PYTHONHOME init, Swagger UI mount at `/docs`
- `src/lib.rs` — PyO3 module definition exporting `AuthUserStruct` and `test_db_connection_py`
- `src/database/mod.rs` — Singleton `PgPool` via `OnceCell`, lazy-initialized from env vars
- `src/database/auth_user.rs` — `AuthUserStruct` model with SeaQuery-based `create_table`, `upsert`, `get_by_unique`
- `src/database/auth_user_py.rs` — Python bindings for AuthUser (sync + async variants, Pydantic-compatible)
- `src/endpoints/` — Axum handlers; `py_example.rs` demonstrates calling Python from Rust
- `build.rs` — Detects Python `sys.base_prefix` at compile time, embeds as `PY_BASE_PREFIX` env var

**Database pattern:** SeaQuery builds SQL, SQLx executes it against PostgreSQL. Pool has 5 max / 2 min connections.

**Python path management:** Endpoints dynamically add venv site-packages and `python/` dir to `sys.path` at runtime.

## API Endpoints

- `GET /health` — Health check
- `GET /handle_py_example_cls` — Calls Python `User` class from Rust
- `GET /handle_py_example_func` — Calls Python `hello()` function from Rust
- `GET /docs` — Swagger UI
- `GET /api-docs/openapi.json` — OpenAPI spec

## Key Dependencies

- **PyO3 0.28** with `auto-initialize` — Python interop
- **Axum 0.8** — HTTP framework
- **SQLx 0.8** (runtime-tokio-rustls) — Async PostgreSQL
- **SeaQuery 1.0.0-rc.29** — SQL query builder
- **Utoipa 5.4** — OpenAPI doc generation from handler annotations
- **Maturin** — Builds the crate as both a Rust binary and a Python native extension (`cdylib` + `rlib`)
