use crate::database::auth_user::AuthUserStruct;
use pyo3::prelude::PyAnyMethods;
use pyo3::types::{PyDict, PyType};
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
    #[pyo3(signature=(email, username, **kwargs))]
    pub fn new(
        email: String,
        username: String,
        kwargs: Option<&Bound<'_, PyDict>>,
    ) -> PyResult<Self> {
        let mut user = AuthUserStruct {
            email,
            username,
            ..Default::default()
        };
        if let Some(kw) = kwargs {
            for (key, value) in kw {
                let key_str = key.extract::<String>().unwrap();
                match key_str.as_str() {
                    "id" => {
                        user.id = value.extract::<i32>().unwrap();
                    }
                    "last_login" => {
                        return Err(pyo3::exceptions::PyValueError::new_err(
                            "last_login is not allowed to be set".to_string(),
                        ));
                    }
                    "last_update" => {
                        return Err(pyo3::exceptions::PyValueError::new_err(
                            "last_update is not allowed to be set".to_string(),
                        ));
                    }
                    "date_joined" => {
                        return Err(pyo3::exceptions::PyValueError::new_err(
                            "date_joined is not allowed to be set".to_string(),
                        ));
                    }
                    _ => {
                        return Err(pyo3::exceptions::PyValueError::new_err(format!(
                            "Invalid keyword argument: {}",
                            key_str
                        )));
                    }
                }
            }
        }
        Ok(user)
    }

    #[classmethod]
    #[pyo3(signature=(email=None, id=None))]
    /// Get an AuthUser by id or email, python wrapper.
    /// # Arguments
    /// * `email` - The email of the user to get.
    /// * `id` - The id of the user to get.
    /// Returns:
    /// * `AuthUserStruct` - The auth user with the given id or email.
    pub fn get_by_unique_py_async<'p>(
        _cls: &Bound<'p, PyType>,
        py: Python<'p>,
        email: Option<String>,
        id: Option<i32>,
    ) -> PyResult<Bound<'p, PyAny>> {
        if email.is_none() && id.is_none() {
            return Err(pyo3::exceptions::PyValueError::new_err(
                "Either email or id must be provided".to_string(),
            ));
        }
        future_into_py(py, async move {
            let pool = crate::database::start_conn_pool().await.map_err(|e| {
                pyo3::exceptions::PyRuntimeError::new_err(format!(
                    "Failed to get database connection pool: {}",
                    e
                ))
            })?;
            match AuthUserStruct::get_by_unique(&pool, email, id).await {
                Ok(user) => Ok(user),
                Err(e) => Err(pyo3::exceptions::PyRuntimeError::new_err(format!(
                    "AuthUser not found, error: {}",
                    e
                ))),
            }
        })
    }

    #[classmethod]
    #[pyo3(signature=(email=None, id=None))]
    pub fn get_by_unique_py<'p>(
        _cls: &Bound<'p, PyType>,
        py: Python<'p>,
        email: Option<String>,
        id: Option<i32>,
    ) -> PyResult<Option<Self>> {
        py.detach(|| {
            let rt = tokio::runtime::Runtime::new().map_err(|e| {
                pyo3::exceptions::PyRuntimeError::new_err(format!(
                    "Failed to create Tokio runtime: {}",
                    e
                ))
            })?;

            rt.block_on(async move {
                let pool = crate::database::start_conn_pool().await.map_err(|e| {
                    pyo3::exceptions::PyRuntimeError::new_err(format!(
                        "Failed to get database connection pool: {}",
                        e
                    ))
                })?;
                AuthUserStruct::get_by_unique(&pool, email, id)
                    .await
                    .map_err(|e| {
                        pyo3::exceptions::PyRuntimeError::new_err(format!(
                            "Failed to get AuthUser: {}",
                            e
                        ))
                    })
            })
        })
    }

    /// Save the AuthUser to the database, python wrapper.
    /// If the email already exists, update the username.
    /// If not, insert a new record.
    /// Returns:
    /// * `AuthUserStruct` - The saved auth user.
    pub fn save_async<'p>(&mut self, py: Python<'p>) -> PyResult<Bound<'p, PyAny>> {
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

    pub fn save<'p>(&mut self, py: Python<'p>) -> PyResult<Self> {
        py.detach(|| {
            let rt = tokio::runtime::Runtime::new().map_err(|e| {
                pyo3::exceptions::PyRuntimeError::new_err(format!(
                    "Failed to create Tokio runtime: {}",
                    e
                ))
            })?;

            rt.block_on(async move {
                let pool = crate::database::start_conn_pool().await.map_err(|e| {
                    pyo3::exceptions::PyRuntimeError::new_err(format!(
                        "Failed to get database connection pool: {}",
                        e
                    ))
                })?;
                self.upsert(&pool).await.map_err(|e| {
                    pyo3::exceptions::PyRuntimeError::new_err(format!(
                        "Failed to save AuthUser: {}",
                        e
                    ))
                })?;
                Ok(self.clone())
            })
        })
    }
}
