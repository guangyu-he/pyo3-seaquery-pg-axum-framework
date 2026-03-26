use pyo3::pymodule;

mod database;
pub mod middleware;

pub mod endpoints;

#[pymodule]
/// Python module definition
mod pyo3_seaquery_pg_axum_framework {
    #[pymodule_export]
    use crate::database::auth_user::AuthUserStruct;
    #[pymodule_export]
    use crate::database::test_db_connection_py;
}
