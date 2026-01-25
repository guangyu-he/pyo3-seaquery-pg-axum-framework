# pyo3-seaquery-pg-axum-framework

Rust + Python + PostgreSQL starter that combines Axum (HTTP server), SQLx + SeaQuery (DB), and PyO3 (Python bindings).
Includes a tiny example endpoint that calls into Python, plus a Python entrypoint that calls back into Rust.

## What’s inside

- **Rust server**: Axum app with `/health` and `/py_example`, plus Swagger UI at `/docs`.
- **Database layer**: SQLx + SeaQuery with a simple `AuthUserStruct` model and upsert/get-by-id helpers.
- **Python bridge**: PyO3 module exposing `AuthUserStruct` and `test_db_connection_py`.
- **Python example**: `python/main.py` shows calling Rust from Python and using the model.

## Quick start

### 1) Start Postgres

```bash
docker compose up -d
```

### 2) Create Python venv (Python 3.13)

```bash
uv venv
uv sync
source .venv/bin/activate
```

### 3) Run the Rust server

```bash
cargo run
```

Server runs at `http://127.0.0.1:3000`

## Endpoints

- `GET /health` → `OK`
- `GET /py_example` → calls `python/main.py`’s `User.greet()`
- `GET /docs` → Swagger UI (OpenAPI JSON at `/api-docs/openapi.json`)

## Python example (calling Rust)

```bash
python python/main.py
```

## Development notes

- DB connection settings are currently hard-coded in `src/database/mod.rs`.
- The sample Postgres config is in `docker-compose.yml`.

## Env hints

if you run into issues with loading the Python shared library, set these env vars before running the Rust server:

```
# use this line to read LIBDIR in your system, with the correct venv activated
# python -c "import sysconfig; print('LIBDIR=', sysconfig.get_config_var('LIBDIR')); print('LDLIBRARY=', sysconfig.get_config_var('LDLIBRARY'))"
export PYO3_PYTHON="$(pwd)/.venv/bin/python"
export LD_LIBRARY_PATH="/root/.local/share/uv/python/cpython-3.13.3-linux-x86_64-gnu/lib:$LD_LIBRARY_PATH"
```
