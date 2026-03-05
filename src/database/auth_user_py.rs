use crate::database::auth_user::AuthUserStruct;
use pyo3::types::PyType;
use pyo3::{Bound, PyAny, PyResult, Python, pymethods};
use pyo3_async_runtimes::tokio::future_into_py;
use serde_pyobject::to_pyobject;

#[pymethods]
impl AuthUserStruct {
    pub fn __repr__(&self) -> String {
        format!(
            "AuthUserStruct(id: {}, email: '{}', username: '{}', last_login: {:?}, last_update: '{}', date_joined: '{}')",
            self.id, self.email, self.username, self.last_login, self.last_update, self.date_joined
        )
    }

    pub fn to_dict<'p>(&self, py: Python<'p>) -> PyResult<Bound<'p, PyAny>> {
        Ok(to_pyobject(py, self).map_err(|e| {
            pyo3::exceptions::PyRuntimeError::new_err(format!(
                "Failed to convert object to dict: {}",
                e
            ))
        })?)
    }

    #[new]
    #[pyo3(signature=(email, username))]
    pub fn new(email: String, username: String) -> Self {
        AuthUserStruct {
            email,
            username,
            ..Default::default()
        }
    }

    #[classmethod]
    #[pyo3(signature=(user_id))]
    /// Get an AuthUser by id, python wrapper.
    /// # Arguments
    /// * `user_id` - The id of the user to get.
    /// Returns:
    /// * `AuthUserStruct` - The auth user with the given id.
    pub fn get_by_id<'p>(
        _cls: &Bound<'p, PyType>,
        py: Python<'p>,
        user_id: i32,
    ) -> PyResult<Bound<'p, PyAny>> {
        future_into_py(py, async move {
            let pool = crate::database::start_conn_pool().await.map_err(|e| {
                pyo3::exceptions::PyRuntimeError::new_err(format!(
                    "Failed to get database connection pool: {}",
                    e
                ))
            })?;
            match AuthUserStruct::get_by_unique(&pool, None, Some(user_id)).await {
                Ok(user) => Ok(user.unwrap()),
                Err(e) => Err(pyo3::exceptions::PyRuntimeError::new_err(format!(
                    "AuthUser with id {} not found, error: {}",
                    user_id, e
                ))),
            }
        })
    }

    /// Save the AuthUser to the database, python wrapper.
    /// If the email already exists, update the username.
    /// If not, insert a new record.
    /// Returns:
    /// * `AuthUserStruct` - The saved auth user.
    pub fn save<'p>(&mut self, py: Python<'p>) -> PyResult<Bound<'p, PyAny>> {
        let mut this = self.clone();
        future_into_py(py, async move {
            let pool = crate::database::start_conn_pool().await.map_err(|e| {
                pyo3::exceptions::PyRuntimeError::new_err(format!(
                    "Failed to get database connection pool: {}",
                    e
                ))
            })?;

            this.upsert(&pool).await.map_err(|e| {
                pyo3::exceptions::PyRuntimeError::new_err(format!("Failed to save AuthUser: {}", e))
            })?;
            Ok(this)
        })
    }
}
