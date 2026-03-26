# pyo3-seaquery-pg-axum-framework

A Rust web framework combining **Axum** (HTTP), **SQLx + SeaQuery** (PostgreSQL), and **PyO3** (embedded Python) with bidirectional Rust-Python interop.

## Architecture

```
src/
├── main.rs              # Axum server entrypoint
├── lib.rs               # PyO3 module exports
├── database/            # SQLx + SeaQuery models & connection pool
├── endpoints/           # HTTP handlers (health, py_example)
└── middleware/           # Tracing / logging
python/
├── call_py_from_rust.py # Python functions invoked by Rust endpoints
└── call_rust_from_py.py # Demo: calling compiled Rust from Python
```

## Prerequisites

- **Rust** (edition 2024)
- **Python 3.13** (required — PyO3 binds against this specific version)
- **uv** (recommended Python package manager)
- **Docker** (optional, for PostgreSQL)

## Getting Started

### 1. Start PostgreSQL (optional)

```bash
docker compose up -d
```

This starts a PostgreSQL 18 instance with default test credentials (`testdbuser` / `testdbpass` / `testdb`) on port 5432.

### 2. Set up Python environment

> **Important:** The Python 3.13 virtual environment **must** be fully configured before compiling. The `build.rs` script queries the active Python interpreter at compile time to locate the standard library — if the venv is missing or not activated, the build will fail or the binary will crash at runtime.

```bash
uv venv --python 3.13
source .venv/bin/activate   # Linux / macOS
# .venv\Scripts\activate    # Windows
uv sync
```

### 3. Start the server

```bash
cargo run
```

Server listens at `http://127.0.0.1:3000`.

## API Endpoints

| Method | Path                      | Description                    |
| ------ | ------------------------- | ------------------------------ |
| `GET`  | `/health`                 | Health check                   |
| `GET`  | `/handle_py_example_cls`  | Call Python class from Rust    |
| `GET`  | `/handle_py_example_func` | Call Python function from Rust |
| `GET`  | `/docs`                   | Swagger UI                     |
| `GET`  | `/api-docs/openapi.json`  | OpenAPI spec                   |

## Calling Rust from Python

Build the native module with [maturin](https://github.com/PyO3/maturin), then run the demo script:

```bash
maturin develop
python python/call_rust_from_py.py
```

## Environment Variables

### Database (required when using DB features)

| Variable      | Description       | Default |
| ------------- | ----------------- | ------- |
| `DB_NAME`     | Database name     | —       |
| `DB_USER`     | Database user     | —       |
| `DB_PASSWORD` | Database password | —       |
| `DB_HOST`     | Database host     | —       |
| `DB_PORT`     | Database port     | `5432`  |

### Python / Runtime

| Variable      | Description                                                                         | Default                                          |
| ------------- | ----------------------------------------------------------------------------------- | ------------------------------------------------ |
| `PYTHONHOME`  | Override embedded Python stdlib path                                                | Auto-detected at compile time via `build.rs`     |
| `PYO3_PYTHON` | Python interpreter for compilation, set it to avoid pyo3 re-compilation over builds | `.venv/bin/python` (set in `.cargo/config.toml`) |
| `RUST_LOG`    | Tracing filter directive                                                            | `info`                                           |

## How Python Enviroment Works with Build

The `build.rs` script runs at compile time to detect the Python base prefix (`sys.base_prefix`) and embeds it into the binary as `PY_BASE_PREFIX`. At startup, `main.rs` sets `PYTHONHOME` from this value so the embedded interpreter can locate its standard library 
— no manual environment variable configuration needed, unless you want to override the default `PYTHONHOME`.

## License

MIT
