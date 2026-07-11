pub mod repositories;

use time::OffsetDateTime;
use turso::{Builder, Connection, Database as TursoDatabase};

use crate::error::ApiError;

const INIT_SQL: &str = include_str!("../../migrations/0001_init.sql");
const AUTHZ_SQL: &str = include_str!("../../migrations/0002_authz.sql");
const EDGE_NODES_SQL: &str = include_str!("../../migrations/0003_edge_nodes.sql");
const DEVICE_POLICIES_SQL: &str = include_str!("../../migrations/0004_device_policies.sql");

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

        self.conn
            .execute_batch(DEVICE_POLICIES_SQL)
            .await
            .map_err(|error| {
                tracing::error!(?error, "device/policy database migration failed");
                ApiError::Database
            })?;

        self.conn
            .execute(
                "INSERT OR IGNORE INTO schema_migrations(version, applied_at) VALUES (?1, ?2)",
                ("0004_device_policies", now()),
            )
            .await
            .map_err(|error| {
                tracing::error!(?error, "device/policy schema migration insert failed");
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

    pub async fn create_device_client(
        &self,
        client: &repositories::DeviceClientInput,
    ) -> Result<repositories::DeviceClientRecord, ApiError> {
        let now = now();
        self.conn
            .execute(
                "INSERT INTO device_clients(id, label, owner_scope, client_id, enabled, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                (
                    client.id.as_str(),
                    client.label.as_str(),
                    client.owner_scope.as_str(),
                    client.client_id.as_str(),
                    client.enabled as i64,
                    now.as_str(),
                    now.as_str(),
                ),
            )
            .await
            .map_err(|error| {
                tracing::error!(?error, client_id = %client.client_id, "device client insert failed");
                ApiError::BadRequest
            })?;
        self.device_client(&client.id).await
    }

    pub async fn update_device_client(
        &self,
        client: &repositories::DeviceClientInput,
    ) -> Result<repositories::DeviceClientRecord, ApiError> {
        let affected = self
            .conn
            .execute(
                "UPDATE device_clients SET label = ?2, owner_scope = ?3, client_id = ?4, enabled = ?5, updated_at = ?6
                 WHERE id = ?1",
                (
                    client.id.as_str(),
                    client.label.as_str(),
                    client.owner_scope.as_str(),
                    client.client_id.as_str(),
                    client.enabled as i64,
                    now(),
                ),
            )
            .await
            .map_err(|error| {
                tracing::error!(?error, id = %client.id, "device client update failed");
                ApiError::BadRequest
            })?;
        if affected == 0 {
            return Err(ApiError::NotFound);
        }
        self.device_client(&client.id).await
    }

    pub async fn device_client(
        &self,
        id: &str,
    ) -> Result<repositories::DeviceClientRecord, ApiError> {
        let mut rows = self
            .conn
            .query(
                "SELECT id, label, owner_scope, client_id, enabled, created_at, updated_at
                 FROM device_clients WHERE id = ?1",
                (id,),
            )
            .await
            .map_err(|error| {
                tracing::error!(?error, id = %id, "device client query failed");
                ApiError::Database
            })?;
        let row = rows.next().await.map_err(|error| {
            tracing::error!(?error, id = %id, "device client row read failed");
            ApiError::Database
        })?;
        let row = row.ok_or(ApiError::NotFound)?;
        device_client_from_row(row)
    }

    pub async fn device_client_by_client_id(
        &self,
        client_id: &str,
    ) -> Result<Option<repositories::DeviceClientRecord>, ApiError> {
        let mut rows = self
            .conn
            .query(
                "SELECT id, label, owner_scope, client_id, enabled, created_at, updated_at
                 FROM device_clients WHERE client_id = ?1",
                (client_id,),
            )
            .await
            .map_err(|error| {
                tracing::error!(?error, "device client lookup failed");
                ApiError::Database
            })?;
        let row = rows.next().await.map_err(|error| {
            tracing::error!(?error, "device client lookup row read failed");
            ApiError::Database
        })?;
        row.map(device_client_from_row).transpose()
    }

    pub async fn list_device_clients(
        &self,
    ) -> Result<Vec<repositories::DeviceClientRecord>, ApiError> {
        let mut rows = self
            .conn
            .query(
                "SELECT id, label, owner_scope, client_id, enabled, created_at, updated_at
                 FROM device_clients ORDER BY created_at ASC",
                (),
            )
            .await
            .map_err(|error| {
                tracing::error!(?error, "device client list query failed");
                ApiError::Database
            })?;
        let mut clients = Vec::new();
        while let Some(row) = rows.next().await.map_err(|error| {
            tracing::error!(?error, "device client list row read failed");
            ApiError::Database
        })? {
            clients.push(device_client_from_row(row)?);
        }
        Ok(clients)
    }

    pub async fn create_dns_policy(
        &self,
        policy: &repositories::DnsPolicyInput,
    ) -> Result<repositories::DnsPolicyRecord, ApiError> {
        let now = now();
        self.conn
            .execute(
                "INSERT INTO dns_policies(id, name, blocked_domains_json, enabled, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                (
                    policy.id.as_str(),
                    policy.name.as_str(),
                    policy.blocked_domains_json.as_str(),
                    policy.enabled as i64,
                    now.as_str(),
                    now.as_str(),
                ),
            )
            .await
            .map_err(|error| {
                tracing::error!(?error, id = %policy.id, "dns policy insert failed");
                ApiError::BadRequest
            })?;
        self.dns_policy(&policy.id).await
    }

    pub async fn update_dns_policy(
        &self,
        policy: &repositories::DnsPolicyInput,
    ) -> Result<repositories::DnsPolicyRecord, ApiError> {
        let affected = self
            .conn
            .execute(
                "UPDATE dns_policies SET name = ?2, blocked_domains_json = ?3, enabled = ?4, updated_at = ?5
                 WHERE id = ?1",
                (
                    policy.id.as_str(),
                    policy.name.as_str(),
                    policy.blocked_domains_json.as_str(),
                    policy.enabled as i64,
                    now(),
                ),
            )
            .await
            .map_err(|error| {
                tracing::error!(?error, id = %policy.id, "dns policy update failed");
                ApiError::BadRequest
            })?;
        if affected == 0 {
            return Err(ApiError::NotFound);
        }
        self.dns_policy(&policy.id).await
    }

    pub async fn dns_policy(&self, id: &str) -> Result<repositories::DnsPolicyRecord, ApiError> {
        let mut rows = self
            .conn
            .query(
                "SELECT id, name, blocked_domains_json, enabled, last_applied_at, created_at, updated_at
                 FROM dns_policies WHERE id = ?1",
                (id,),
            )
            .await
            .map_err(|error| {
                tracing::error!(?error, id = %id, "dns policy query failed");
                ApiError::Database
            })?;
        let row = rows.next().await.map_err(|error| {
            tracing::error!(?error, id = %id, "dns policy row read failed");
            ApiError::Database
        })?;
        let row = row.ok_or(ApiError::NotFound)?;
        dns_policy_from_row(row)
    }

    pub async fn list_dns_policies(&self) -> Result<Vec<repositories::DnsPolicyRecord>, ApiError> {
        let mut rows = self
            .conn
            .query(
                "SELECT id, name, blocked_domains_json, enabled, last_applied_at, created_at, updated_at
                 FROM dns_policies ORDER BY created_at ASC",
                (),
            )
            .await
            .map_err(|error| {
                tracing::error!(?error, "dns policy list query failed");
                ApiError::Database
            })?;
        let mut policies = Vec::new();
        while let Some(row) = rows.next().await.map_err(|error| {
            tracing::error!(?error, "dns policy list row read failed");
            ApiError::Database
        })? {
            policies.push(dns_policy_from_row(row)?);
        }
        Ok(policies)
    }

    pub async fn mark_dns_policies_applied(&self, ids: &[String]) -> Result<(), ApiError> {
        let now = now();
        for id in ids {
            self.conn
                .execute(
                    "UPDATE dns_policies SET last_applied_at = ?2 WHERE id = ?1",
                    (id.as_str(), now.as_str()),
                )
                .await
                .map_err(|error| {
                    tracing::error!(?error, id = %id, "dns policy apply timestamp update failed");
                    ApiError::Database
                })?;
        }
        Ok(())
    }
}

fn device_client_from_row(row: turso::Row) -> Result<repositories::DeviceClientRecord, ApiError> {
    Ok(repositories::DeviceClientRecord {
        id: row.get(0).map_err(|_| ApiError::Database)?,
        label: row.get(1).map_err(|_| ApiError::Database)?,
        owner_scope: row.get(2).map_err(|_| ApiError::Database)?,
        client_id: row.get(3).map_err(|_| ApiError::Database)?,
        enabled: row.get::<i64>(4).map_err(|_| ApiError::Database)? != 0,
        created_at: row.get(5).map_err(|_| ApiError::Database)?,
        updated_at: row.get(6).map_err(|_| ApiError::Database)?,
    })
}

fn dns_policy_from_row(row: turso::Row) -> Result<repositories::DnsPolicyRecord, ApiError> {
    Ok(repositories::DnsPolicyRecord {
        id: row.get(0).map_err(|_| ApiError::Database)?,
        name: row.get(1).map_err(|_| ApiError::Database)?,
        blocked_domains_json: row.get(2).map_err(|_| ApiError::Database)?,
        enabled: row.get::<i64>(3).map_err(|_| ApiError::Database)? != 0,
        last_applied_at: row.get(4).map_err(|_| ApiError::Database)?,
        created_at: row.get(5).map_err(|_| ApiError::Database)?,
        updated_at: row.get(6).map_err(|_| ApiError::Database)?,
    })
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

#[cfg(test)]
mod tests {
    use super::Database;
    use crate::db::repositories::{DeviceClientInput, DnsPolicyInput};
    use crate::error::ApiError;

    async fn test_db() -> Database {
        let path = std::env::temp_dir().join(format!("nabe-test-{}.db", uuid::Uuid::new_v4()));
        let db = Database::open(path.to_str().expect("path"))
            .await
            .expect("open");
        db.migrate().await.expect("migrate");
        db
    }

    fn client_input(id: &str, client_id: &str, enabled: bool) -> DeviceClientInput {
        DeviceClientInput {
            id: id.to_string(),
            label: format!("client {id}"),
            owner_scope: "default".to_string(),
            client_id: client_id.to_string(),
            enabled,
        }
    }

    #[tokio::test]
    async fn device_client_crud_roundtrip() {
        let db = test_db().await;
        let created = db
            .create_device_client(&client_input("dc-1", "pixel-8", true))
            .await
            .expect("create");
        assert!(created.enabled);
        assert_eq!(created.client_id, "pixel-8");

        let mut update = client_input("dc-1", "pixel-8", false);
        update.label = "renamed".to_string();
        let updated = db.update_device_client(&update).await.expect("update");
        assert!(!updated.enabled);
        assert_eq!(updated.label, "renamed");

        let listed = db.list_device_clients().await.expect("list");
        assert_eq!(listed.len(), 1);

        let by_client_id = db
            .device_client_by_client_id("pixel-8")
            .await
            .expect("lookup");
        assert!(by_client_id.is_some());
        assert!(db
            .device_client_by_client_id("missing")
            .await
            .expect("lookup")
            .is_none());
    }

    #[tokio::test]
    async fn device_client_client_id_must_be_unique() {
        let db = test_db().await;
        db.create_device_client(&client_input("dc-1", "pixel-8", true))
            .await
            .expect("create");
        let duplicate = db
            .create_device_client(&client_input("dc-2", "pixel-8", true))
            .await;
        assert!(duplicate.is_err());
    }

    #[tokio::test]
    async fn update_missing_device_client_returns_not_found() {
        let db = test_db().await;
        let result = db
            .update_device_client(&client_input("missing", "pixel-8", true))
            .await;
        assert!(matches!(result, Err(ApiError::NotFound)));
    }

    #[tokio::test]
    async fn dns_policy_crud_and_apply_timestamp() {
        let db = test_db().await;
        let input = DnsPolicyInput {
            id: "pol-1".to_string(),
            name: "test policy".to_string(),
            blocked_domains_json: r#"["blocked.nabe.test"]"#.to_string(),
            enabled: true,
        };
        let created = db.create_dns_policy(&input).await.expect("create");
        assert!(created.last_applied_at.is_none());

        let mut update = input.clone();
        update.enabled = false;
        let updated = db.update_dns_policy(&update).await.expect("update");
        assert!(!updated.enabled);

        db.mark_dns_policies_applied(&["pol-1".to_string()])
            .await
            .expect("mark applied");
        let applied = db.dns_policy("pol-1").await.expect("get");
        assert!(applied.last_applied_at.is_some());

        assert_eq!(db.list_dns_policies().await.expect("list").len(), 1);
    }
}
