use chrono::{NaiveDateTime, Utc};
use pyo3::pyclass;
use sea_query::{Asterisk, ColumnDef, Expr, ExprTrait, Iden, PostgresQueryBuilder, Query, Table};
use sea_query_sqlx::SqlxBinder;
use serde::Serialize;
use sqlx::PgPool;
use tracing::info;

#[derive(Iden)]
enum AuthUser {
    Table,
    Id,
    Email,
    Username,
    DateJoined,
    LastLogin,
    LastUpdate,
}

impl AuthUser {
    #[allow(dead_code)]
    pub async fn create_table(pool: &PgPool) -> anyhow::Result<()> {
        // create the auth_user table
        let sql = Table::create()
            .table(Self::Table)
            .if_not_exists()
            .col(
                ColumnDef::new(Self::Id)
                    .integer()
                    .not_null()
                    .auto_increment()
                    .primary_key(),
            )
            .col(ColumnDef::new(Self::Email).string().not_null().unique_key())
            .col(
                ColumnDef::new(Self::Username)
                    .string()
                    .not_null()
                    .unique_key(),
            )
            .col(ColumnDef::new(Self::DateJoined).timestamp().not_null())
            .col(ColumnDef::new(Self::LastLogin).timestamp())
            .col(ColumnDef::new(Self::LastUpdate).timestamp().not_null())
            .build(PostgresQueryBuilder);
        let result = sqlx::query(&sql).execute(pool).await?;

        // apply last_update pg function: properly set up to auto-update on row modifications
        let queries: [&str; 3] = [
            "CREATE OR REPLACE FUNCTION auth_user_set_last_update() RETURNS TRIGGER AS $$
                BEGIN
                    NEW.last_update = NOW();
                    RETURN NEW;
                END;
            $$ LANGUAGE plpgsql;",
            "DROP TRIGGER IF EXISTS trg_auth_user_set_last_update ON auth_user;",
            "CREATE TRIGGER trg_auth_user_set_last_update BEFORE UPDATE ON auth_user FOR EACH ROW EXECUTE PROCEDURE auth_user_set_last_update();",
        ];
        for query in queries {
            sqlx::query(query).execute(pool).await?;
        }
        info!(
            "AuthUser table created or already exists: {}",
            result.rows_affected()
        );
        Ok(())
    }
}

#[pyclass(from_py_object, dict, subclass, get_all, set_all)]
#[derive(Clone, Debug, Serialize, sqlx::FromRow)]
pub struct AuthUserStruct {
    pub id: i32,
    pub email: String,    // unique, as identifier
    pub username: String, // unique
    pub last_login: Option<NaiveDateTime>,
    pub last_update: NaiveDateTime,
    pub date_joined: NaiveDateTime,
}

impl Default for AuthUserStruct {
    fn default() -> Self {
        AuthUserStruct {
            id: 0,
            email: String::new(),
            username: String::new(),
            date_joined: Utc::now().naive_utc(),
            last_login: None,
            last_update: Utc::now().naive_utc(),
        }
    }
}

impl AuthUserStruct {
    pub async fn upsert(&mut self, pool: &PgPool) -> anyhow::Result<Self> {
        let (sql, values) = Query::insert()
            .into_table(AuthUser::Table)
            .columns([
                AuthUser::Email,
                AuthUser::Username,
                AuthUser::DateJoined,
                AuthUser::LastLogin,
            ])
            .values_panic([
                self.email.clone().into(),
                self.username.clone().into(),
                self.date_joined.into(),
                self.last_login.clone().into(),
            ])
            .on_conflict(
                sea_query::OnConflict::column(AuthUser::Email)
                    .update_columns([
                        AuthUser::Username,
                        AuthUser::DateJoined,
                        AuthUser::LastLogin,
                    ])
                    .to_owned(),
            )
            .returning_all()
            .build_sqlx(PostgresQueryBuilder);

        *self = sqlx::query_as_with::<_, Self, _>(&sql, values)
            .fetch_one(pool)
            .await?;

        Ok(self.clone())
    }

    pub async fn get_by_unique(
        pool: &PgPool,
        email: Option<String>,
        id: Option<i32>,
    ) -> anyhow::Result<Option<Self>> {
        let mut query = Query::select();
        query.column(Asterisk).from(AuthUser::Table);

        if let Some(e) = email {
            query.and_where(Expr::col(AuthUser::Email).eq(e));
        }
        if let Some(i) = id {
            query.and_where(Expr::col(AuthUser::Id).eq(i));
        }

        let (sql, values) = query.build_sqlx(PostgresQueryBuilder);

        let result = sqlx::query_as_with::<_, Self, _>(&sql, values)
            .fetch_optional(pool)
            .await?;
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database;

    #[tokio::test]
    async fn test_create_table() -> anyhow::Result<()> {
        let pool = database::start_conn_pool().await?;
        AuthUser::create_table(&pool)
            .await
            .expect("Failed to create auth_user table");
        Ok(())
    }

    #[tokio::test]
    async fn test_upsert_auth_user() -> anyhow::Result<()> {
        let pool = database::start_conn_pool().await?;
        let mut user = AuthUserStruct::default();
        user.upsert(&pool)
            .await
            .expect("Failed to upsert auth user");
        Ok(())
    }
}
