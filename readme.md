set necessary python env

```
# use this line to read LIBDIR in your system
# python -c "import sysconfig; print('LIBDIR=', sysconfig.get_config_var('LIBDIR')); print('LDLIBRARY=', sysconfig.get_config_var('LDLIBRARY'))"
export PYO3_PYTHON="$(pwd)/.venv/bin/python"
export LD_LIBRARY_PATH="/root/.local/share/uv/python/cpython-3.13.3-linux-x86_64-gnu/lib:$LD_LIBRARY_PATH"
```