use chrono::{DateTime, Utc};
use pyo3::types::PyType;
use pyo3::{Bound, PyAny, PyResult, Python, pyclass, pymethods};
use pyo3_async_runtimes::tokio::future_into_py;
use sea_query::{ColumnDef, Expr, ExprTrait, Iden, PostgresQueryBuilder, Query, Table};
use sea_query_sqlx::SqlxBinder;
use serde::Serialize;
use serde_pyobject::to_pyobject;
use sqlx::PgPool;

#[derive(Iden)]
enum AuthUser {
    Table,
    Id,
    Email,
    Username,
    DateJoined,
}

#[pyclass(dict, subclass, get_all, set_all)]
#[derive(Clone, Debug, Serialize, sqlx::FromRow)]
pub struct AuthUserStruct {
    id: i32,
    email: String,    // unique, as identifier
    username: String, // unique
    date_joined: DateTime<Utc>,
}

impl Default for AuthUserStruct {
    fn default() -> Self {
        AuthUserStruct {
            id: 0,
            email: String::new(),
            username: String::new(),
            date_joined: Utc::now(),
        }
    }
}

#[pymethods]
impl AuthUserStruct {
    pub fn __repr__(&self) -> String {
        format!(
            "AuthUserStruct(id: {}, email: '{}', username: '{}', date_joined: '{}')",
            self.id, self.email, self.username, self.date_joined
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
            let pool = crate::database::start_conn_pool().await;
            match get_auth_user_by_id(&pool, user_id).await {
                Ok(user) => Ok(user),
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
    pub fn save<'p>(&self, py: Python<'p>) -> PyResult<Bound<'p, PyAny>> {
        let user = self.clone();
        future_into_py(py, async move {
            let pool = crate::database::start_conn_pool().await;
            match upsert_auth_user(&pool, user.email, user.username, user.date_joined).await {
                Ok(user) => Ok(user),
                Err(e) => {
                    return Err(pyo3::exceptions::PyRuntimeError::new_err(format!(
                        "Failed to save AuthUser: {}",
                        e
                    )));
                }
            }
        })
    }
}

#[allow(dead_code)]
/// Create the auth_user table if it does not exist.
/// # Arguments
/// * `pool` - The database connection pool.
async fn create_table(pool: &PgPool) -> Result<(), sqlx::Error> {
    let mut connection = pool.acquire().await?;

    let sql = Table::create()
        .table(AuthUser::Table)
        .if_not_exists()
        .col(
            ColumnDef::new(AuthUser::Id)
                .integer()
                .not_null()
                .auto_increment()
                .primary_key(),
        )
        .col(
            ColumnDef::new(AuthUser::Email)
                .string()
                .not_null()
                .unique_key(),
        )
        .col(
            ColumnDef::new(AuthUser::Username)
                .string()
                .not_null()
                .unique_key(),
        )
        .col(
            ColumnDef::new(AuthUser::DateJoined)
                .timestamp_with_time_zone()
                .not_null(),
        )
        .build(PostgresQueryBuilder);

    let result = sqlx::query(&sql).execute(&mut *connection).await;
    match result {
        Ok(r) => Ok(println!(
            "AuthUser table created or already exists: {} rows affected",
            r.rows_affected()
        )),
        Err(e) => Err(e),
    }
}

/// Upsert an auth user. If the email already exists, update the username.
/// If not, insert a new record.
/// # Arguments
/// * `pool` - The database connection pool.
/// * `email` - The email of the user.
/// * `username` - The username of the user.
/// * `date_joined` - The date the user joined.
/// Returns:
/// * `AuthUserStruct` - The upserted auth user.
pub async fn upsert_auth_user(
    pool: &PgPool,
    email: impl AsRef<str>,
    username: impl AsRef<str>,
    date_joined: DateTime<Utc>,
) -> Result<AuthUserStruct, sqlx::Error> {
    let mut conn = pool.acquire().await?;

    let (sql, values) = Query::insert()
        .into_table(AuthUser::Table)
        .columns([AuthUser::Email, AuthUser::Username, AuthUser::DateJoined])
        .values_panic([
            email.as_ref().into(),
            username.as_ref().into(),
            date_joined.into(),
        ])
        .on_conflict(
            sea_query::OnConflict::column(AuthUser::Email)
                .update_columns([AuthUser::Username]) // 仅更新 username
                .to_owned(),
        )
        .returning_all()
        .build_sqlx(PostgresQueryBuilder);

    let row_structured = sqlx::query_as_with::<_, AuthUserStruct, _>(&sql, values)
        .fetch_one(&mut *conn)
        .await?;

    Ok(row_structured)
}

/// Get an auth user by id.
/// # Arguments
/// * `pool` - The database connection pool.
/// * `user_id` - The id of the user to get.
/// Returns:
/// * `Some(AuthUserStruct)` - The auth user with the given id.
/// * `None` - If the user does not exist.
async fn get_auth_user_by_id(pool: &PgPool, user_id: i32) -> Result<AuthUserStruct, sqlx::Error> {
    let mut connection = pool.acquire().await?;
    let (sql, values) = Query::select()
        .columns([
            AuthUser::Id,
            AuthUser::Email,
            AuthUser::Username,
            AuthUser::DateJoined,
        ])
        .from(AuthUser::Table)
        .and_where(Expr::col(AuthUser::Id).eq(user_id))
        .build_sqlx(PostgresQueryBuilder);
    let row_structured = sqlx::query_as_with::<_, AuthUserStruct, _>(&sql, values)
        .fetch_one(&mut *connection)
        .await?;
    Ok(row_structured)
}

mod tests {
    #[allow(unused_imports)]
    use super::*;
    #[allow(unused_imports)]
    use crate::database;

    #[tokio::test]
    async fn test_create_table() {
        let pool = database::start_conn_pool().await;
        create_table(&pool)
            .await
            .expect("Failed to create auth_user table");
    }

    #[tokio::test]
    async fn test_upsert_auth_user() {
        let pool = database::start_conn_pool().await;
        upsert_auth_user(&pool, "test@test.de", "testuser", Utc::now())
            .await
            .expect("Failed to upsert auth user");
    }
}
