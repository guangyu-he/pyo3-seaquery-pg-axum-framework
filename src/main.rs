use pyo3::prelude::*;
use pyo3::types::PyDict;

fn main() -> PyResult<()> {
    // Example of initializing Python interpreter and calling a Python function
    Python::initialize();
    Python::attach(|py| {
        let sys = py.import("sys")?;
        let path = sys.getattr("path")?;
        path.call_method1("append", (".venv/lib/python3.13/site-packages",))?; // append my venv path
        path.call_method1("append", ("./python",))?; // append my custom module path

        let module = py.import("main")?;
        let user_cls = module.getattr("User")?;

        // 构造参数（dict / kwargs）
        let kwargs = PyDict::new(py);
        kwargs.set_item("id", 1)?;
        kwargs.set_item("name", "Bob")?;
        kwargs.set_item("email", "test@example.de")?;

        let user = user_cls.call((), Some(&kwargs))?;

        // 调用方法
        let greet: String = user.getattr("greet")?.call0()?.extract()?;
        println!("{}", greet);

        Ok(())
    })
}
