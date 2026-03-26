use pyo3::pymodule;

pub mod database;
pub mod endpoints;
pub mod middleware;

#[pymodule]
/// Python module definition
mod pyo3_seaquery_pg_axum_framework {
    #[pymodule_export]
    use crate::database::auth_user::AuthUserStruct;
    #[pymodule_export]
    use crate::database::utils_py::test_db_connection_py;
}
