pub mod repositories;

use time::OffsetDateTime;
use turso::{Builder, Connection, Database as TursoDatabase};

use crate::error::ApiError;

const INIT_SQL: &str = include_str!("../../migrations/0001_init.sql");
const AUTHZ_SQL: &str = include_str!("../../migrations/0002_authz.sql");

pub struct Database {
    _db: TursoDatabase,
    conn: Connection,
}

impl Database {
    pub async fn open(path: &str) -> anyhow::Result<Self> {
        let db = Builder::new_local(path).build().await?;
        let conn = db.connect()?;
        Ok(Self { _db: db, conn })
    }

    pub async fn migrate(&self) -> Result<(), ApiError> {
        self.conn.execute_batch(INIT_SQL).await.map_err(|error| {
            tracing::error!(?error, "database migration failed");
            ApiError::Database
        })?;

        self.conn
            .execute(
                "INSERT OR IGNORE INTO schema_migrations(version, applied_at) VALUES (?1, ?2)",
                ("0001_init", now()),
            )
            .await
            .map_err(|error| {
                tracing::error!(?error, "schema migration insert failed");
                ApiError::Database
            })?;

        self.conn.execute_batch(AUTHZ_SQL).await.map_err(|error| {
            tracing::error!(?error, "authz database migration failed");
            ApiError::Database
        })?;

        self.conn
            .execute(
                "INSERT OR IGNORE INTO schema_migrations(version, applied_at) VALUES (?1, ?2)",
                ("0002_authz", now()),
            )
            .await
            .map_err(|error| {
                tracing::error!(?error, "authz schema migration insert failed");
                ApiError::Database
            })?;

        Ok(())
    }

    pub async fn ping(&self) -> Result<(), ApiError> {
        self.conn.query("SELECT 1", ()).await.map_err(|error| {
            tracing::error!(?error, "database ping failed");
            ApiError::Database
        })?;
        Ok(())
    }

    pub async fn ensure_engine_instance(
        &self,
        name: &str,
        kind: &str,
        base_url: &str,
    ) -> Result<(), ApiError> {
        let now = now();
        self.conn
            .execute(
                "INSERT OR IGNORE INTO dns_engine_instances(id, kind, name, base_url, status, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, 'unknown', ?5, ?6)",
                ("cloud-dns", kind, name, base_url, now.as_str(), now.as_str()),
            )
            .await
            .map_err(|error| {
                tracing::error!(?error, "engine instance seed failed");
                ApiError::Database
            })?;
        Ok(())
    }

    pub async fn upsert_subject(
        &self,
        subject: &repositories::SubjectInput,
    ) -> Result<(), ApiError> {
        let now = now();
        self.conn
            .execute(
                "INSERT INTO oidc_subjects(id, provider, subject, email, display_name, role, first_seen_at, last_seen_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, 'admin', ?6, ?7)
                 ON CONFLICT(provider, subject) DO UPDATE SET
                   email = excluded.email,
                   display_name = excluded.display_name,
                   role = 'admin',
                   last_seen_at = excluded.last_seen_at",
                (
                    subject.id.as_str(),
                    subject.provider.as_str(),
                    subject.subject.as_str(),
                    subject.email.as_deref(),
                    subject.display_name.as_deref(),
                    now.as_str(),
                    now.as_str(),
                ),
            )
            .await
            .map_err(|error| {
                tracing::error!(?error, "subject upsert failed");
                ApiError::Database
            })?;
        Ok(())
    }
}

fn now() -> String {
    OffsetDateTime::now_utc()
        .format(&time::format_description::well_known::Rfc3339)
        .unwrap_or_else(|_| "1970-01-01T00:00:00Z".to_string())
}
