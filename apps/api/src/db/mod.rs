pub mod repositories;

use time::OffsetDateTime;
use turso::{Builder, Connection, Database as TursoDatabase};

use crate::error::ApiError;

const INIT_SQL: &str = include_str!("../../migrations/0001_init.sql");
const AUTHZ_SQL: &str = include_str!("../../migrations/0002_authz.sql");
const EDGE_NODES_SQL: &str = include_str!("../../migrations/0003_edge_nodes.sql");

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

        self.conn
            .execute_batch(EDGE_NODES_SQL)
            .await
            .map_err(|error| {
                tracing::error!(?error, "edge node database migration failed");
                ApiError::Database
            })?;

        self.conn
            .execute(
                "INSERT OR IGNORE INTO schema_migrations(version, applied_at) VALUES (?1, ?2)",
                ("0003_edge_nodes", now()),
            )
            .await
            .map_err(|error| {
                tracing::error!(?error, "edge node schema migration insert failed");
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

    pub async fn upsert_edge_node(
        &self,
        node: &repositories::EdgeNodeInput,
    ) -> Result<repositories::EdgeNodeRecord, ApiError> {
        let now = now();
        self.conn
            .execute(
                "INSERT INTO edge_nodes(id, name, hostname, architecture, os, kernel, speiche_version, health_status, inventory_json, enrolled_at, last_seen_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
                 ON CONFLICT(id) DO UPDATE SET
                   name = excluded.name,
                   hostname = excluded.hostname,
                   architecture = excluded.architecture,
                   os = excluded.os,
                   kernel = excluded.kernel,
                   speiche_version = excluded.speiche_version,
                   health_status = excluded.health_status,
                   inventory_json = excluded.inventory_json,
                   last_seen_at = excluded.last_seen_at,
                   updated_at = excluded.updated_at",
                (
                    node.id.as_str(),
                    node.name.as_str(),
                    node.hostname.as_str(),
                    node.architecture.as_str(),
                    node.os.as_str(),
                    node.kernel.as_str(),
                    node.speiche_version.as_str(),
                    node.health_status.as_str(),
                    node.inventory_json.as_deref(),
                    now.as_str(),
                    now.as_str(),
                    now.as_str(),
                ),
            )
            .await
            .map_err(|error| {
                tracing::error!(?error, node_id = %node.id, "edge node upsert failed");
                ApiError::Database
            })?;

        self.edge_node(&node.id).await
    }

    pub async fn edge_node(&self, id: &str) -> Result<repositories::EdgeNodeRecord, ApiError> {
        let mut rows = self
            .conn
            .query(
                "SELECT id, name, hostname, architecture, os, kernel, speiche_version, health_status, inventory_json, enrolled_at, last_seen_at
                 FROM edge_nodes WHERE id = ?1",
                (id,),
            )
            .await
            .map_err(|error| {
                tracing::error!(?error, node_id = %id, "edge node query failed");
                ApiError::Database
            })?;
        let row = rows.next().await.map_err(|error| {
            tracing::error!(?error, node_id = %id, "edge node row read failed");
            ApiError::Database
        })?;
        let row = row.ok_or(ApiError::NotFound)?;
        edge_node_from_row(row)
    }

    pub async fn list_edge_nodes(&self) -> Result<Vec<repositories::EdgeNodeRecord>, ApiError> {
        let mut rows = self
            .conn
            .query(
                "SELECT id, name, hostname, architecture, os, kernel, speiche_version, health_status, inventory_json, enrolled_at, last_seen_at
                 FROM edge_nodes ORDER BY last_seen_at DESC",
                (),
            )
            .await
            .map_err(|error| {
                tracing::error!(?error, "edge node list query failed");
                ApiError::Database
            })?;

        let mut nodes = Vec::new();
        while let Some(row) = rows.next().await.map_err(|error| {
            tracing::error!(?error, "edge node list row read failed");
            ApiError::Database
        })? {
            nodes.push(edge_node_from_row(row)?);
        }
        Ok(nodes)
    }
}

fn edge_node_from_row(row: turso::Row) -> Result<repositories::EdgeNodeRecord, ApiError> {
    Ok(repositories::EdgeNodeRecord {
        id: row.get(0).map_err(|_| ApiError::Database)?,
        name: row.get(1).map_err(|_| ApiError::Database)?,
        hostname: row.get(2).map_err(|_| ApiError::Database)?,
        architecture: row.get(3).map_err(|_| ApiError::Database)?,
        os: row.get(4).map_err(|_| ApiError::Database)?,
        kernel: row.get(5).map_err(|_| ApiError::Database)?,
        speiche_version: row.get(6).map_err(|_| ApiError::Database)?,
        health_status: row.get(7).map_err(|_| ApiError::Database)?,
        inventory_json: row.get(8).map_err(|_| ApiError::Database)?,
        enrolled_at: row.get(9).map_err(|_| ApiError::Database)?,
        last_seen_at: row.get(10).map_err(|_| ApiError::Database)?,
    })
}

fn now() -> String {
    OffsetDateTime::now_utc()
        .format(&time::format_description::well_known::Rfc3339)
        .unwrap_or_else(|_| "1970-01-01T00:00:00Z".to_string())
}
