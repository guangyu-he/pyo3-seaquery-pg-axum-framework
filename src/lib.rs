use pyo3::pymodule;

mod database;

#[pymodule]
mod gt_rust_py_pg_axum_framework {
    #[pymodule_export]
    use crate::database::auth_user::AuthUserStruct;
    #[pymodule_export]
    use crate::database::test_db_connection_py;
}
