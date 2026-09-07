pub mod repositories;

use time::OffsetDateTime;
use turso::{Builder, Connection, Database as TursoDatabase};

use crate::error::ApiError;

const INIT_SQL: &str = include_str!("../../migrations/0001_init.sql");
const AUTHZ_SQL: &str = include_str!("../../migrations/0002_authz.sql");
const EDGE_NODES_SQL: &str = include_str!("../../migrations/0003_edge_nodes.sql");
const DEVICE_POLICIES_SQL: &str = include_str!("../../migrations/0004_device_policies.sql");
const AUDIT_EVENTS_SQL: &str = include_str!("../../migrations/0005_audit_events.sql");
const NATIVE_AUTHORIZATION_SQL: &str =
    include_str!("../../migrations/0006_native_authorization.sql");
const TENANT_RESOURCES_SQL: &str = include_str!("../../migrations/0007_tenant_resources.sql");
const TELEMETRY_PERMISSIONS_SQL: &str =
    include_str!("../../migrations/0008_telemetry_permissions.sql");
const DNS_STAT_BUCKETS_SQL: &str = include_str!("../../migrations/0009_dns_stat_buckets.sql");
const EDGE_CREDENTIALS_SQL: &str = include_str!("../../migrations/0010_edge_credentials.sql");

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

        let audit_migrated = self.migration_applied("0005_audit_events").await?;
        if !audit_migrated {
            self.conn
                .execute_batch(AUDIT_EVENTS_SQL)
                .await
                .map_err(|error| {
                    tracing::error!(?error, "audit event database migration failed");
                    ApiError::Database
                })?;
            self.conn
                .execute(
                    "INSERT INTO schema_migrations(version, applied_at) VALUES (?1, ?2)",
                    ("0005_audit_events", now()),
                )
                .await
                .map_err(|error| {
                    tracing::error!(?error, "audit event schema migration insert failed");
                    ApiError::Database
                })?;
        }

        if !self.migration_applied("0006_native_authorization").await? {
            self.conn
                .execute_batch(NATIVE_AUTHORIZATION_SQL)
                .await
                .map_err(|error| {
                    tracing::error!(?error, "native authorization database migration failed");
                    ApiError::Database
                })?;
            self.conn
                .execute(
                    "INSERT INTO schema_migrations(version, applied_at) VALUES (?1, ?2)",
                    ("0006_native_authorization", now()),
                )
                .await
                .map_err(|error| {
                    tracing::error!(?error, "native authorization migration insert failed");
                    ApiError::Database
                })?;
        }

        if !self.migration_applied("0007_tenant_resources").await? {
            self.conn
                .execute_batch(TENANT_RESOURCES_SQL)
                .await
                .map_err(|error| {
                    tracing::error!(?error, "tenant resources database migration failed");
                    ApiError::Database
                })?;
            self.conn
                .execute(
                    "INSERT INTO schema_migrations(version, applied_at) VALUES (?1, ?2)",
                    ("0007_tenant_resources", now()),
                )
                .await
                .map_err(|error| {
                    tracing::error!(?error, "tenant resources migration insert failed");
                    ApiError::Database
                })?;
        }

        if !self.migration_applied("0008_telemetry_permissions").await? {
            self.conn
                .execute_batch(TELEMETRY_PERMISSIONS_SQL)
                .await
                .map_err(|error| {
                    tracing::error!(?error, "telemetry permissions database migration failed");
                    ApiError::Database
                })?;
            self.conn
                .execute(
                    "INSERT INTO schema_migrations(version, applied_at) VALUES (?1, ?2)",
                    ("0008_telemetry_permissions", now()),
                )
                .await
                .map_err(|error| {
                    tracing::error!(?error, "telemetry permissions migration insert failed");
                    ApiError::Database
                })?;
        }

        if !self.migration_applied("0009_dns_stat_buckets").await? {
            self.conn
                .execute_batch(DNS_STAT_BUCKETS_SQL)
                .await
                .map_err(|error| {
                    tracing::error!(?error, "DNS statistic buckets database migration failed");
                    ApiError::Database
                })?;
            self.conn
                .execute(
                    "INSERT INTO schema_migrations(version, applied_at) VALUES (?1, ?2)",
                    ("0009_dns_stat_buckets", now()),
                )
                .await
                .map_err(|error| {
                    tracing::error!(?error, "DNS statistic buckets migration insert failed");
                    ApiError::Database
                })?;
        }

        if !self.migration_applied("0010_edge_credentials").await? {
            self.conn
                .execute_batch(EDGE_CREDENTIALS_SQL)
                .await
                .map_err(|error| {
                    tracing::error!(?error, "edge credentials database migration failed");
                    ApiError::Database
                })?;
            self.conn
                .execute(
                    "INSERT INTO schema_migrations(version, applied_at) VALUES (?1, ?2)",
                    ("0010_edge_credentials", now()),
                )
                .await
                .map_err(|error| {
                    tracing::error!(?error, "edge credentials migration insert failed");
                    ApiError::Database
                })?;
        }

        Ok(())
    }

    async fn migration_applied(&self, version: &str) -> Result<bool, ApiError> {
        let mut rows = self
            .conn
            .query(
                "SELECT 1 FROM schema_migrations WHERE version = ?1 LIMIT 1",
                (version,),
            )
            .await
            .map_err(|error| {
                tracing::error!(?error, version, "schema migration lookup failed");
                ApiError::Database
            })?;
        Ok(rows
            .next()
            .await
            .map_err(|error| {
                tracing::error!(?error, version, "schema migration lookup row failed");
                ApiError::Database
            })?
            .is_some())
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
                 VALUES (?1, ?2, ?3, ?4, ?5, 'baseline', ?6, ?7)
                 ON CONFLICT(provider, subject) DO UPDATE SET
                   email = excluded.email,
                   display_name = excluded.display_name,
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
        self.conn.execute(
            "INSERT OR IGNORE INTO subject_global_roles(subject_id, role_id, created_at)
             SELECT id, 'baseline', ?3 FROM oidc_subjects WHERE provider = ?1 AND subject = ?2",
            (subject.provider.as_str(), subject.subject.as_str(), now.as_str()),
        ).await.map_err(|error| {
            tracing::error!(?error, provider = %subject.provider, subject = %subject.subject, "baseline role grant failed");
            ApiError::Database
        })?;
        Ok(())
    }

    pub async fn authorization_for_subject(
        &self,
        provider: &str,
        subject: &str,
    ) -> Result<repositories::AuthorizationRecord, ApiError> {
        let mut global_rows = self
            .conn
            .query(
                "SELECT gr.name, grp.permission_id
             FROM oidc_subjects s
             JOIN subject_global_roles sgr ON sgr.subject_id = s.id
             JOIN global_roles gr ON gr.id = sgr.role_id
             LEFT JOIN global_role_permissions grp ON grp.role_id = gr.id
             WHERE s.provider = ?1 AND s.subject = ?2
             ORDER BY gr.name, grp.permission_id",
                (provider, subject),
            )
            .await
            .map_err(|error| {
                tracing::error!(
                    ?error,
                    provider,
                    subject,
                    "global authorization query failed"
                );
                ApiError::Database
            })?;
        let mut global_roles = Vec::new();
        let mut global_permissions = Vec::new();
        while let Some(row) = global_rows.next().await.map_err(|_| ApiError::Database)? {
            global_roles.push(row.get(0).map_err(|_| ApiError::Database)?);
            if let Some(permission) = row
                .get::<Option<String>>(1)
                .map_err(|_| ApiError::Database)?
            {
                global_permissions.push(permission);
            }
        }
        global_roles.sort();
        global_roles.dedup();
        global_permissions.sort();
        global_permissions.dedup();

        let mut tenant_rows = self
            .conn
            .query(
                "SELECT t.id, t.slug, tr.name, trp.permission_id
             FROM oidc_subjects s
             JOIN tenant_memberships tm ON tm.subject_id = s.id
             JOIN tenants t ON t.id = tm.tenant_id
             JOIN tenant_roles tr ON tr.id = tm.role_id
             LEFT JOIN tenant_role_permissions trp ON trp.role_id = tr.id
             WHERE s.provider = ?1 AND s.subject = ?2
             ORDER BY t.id, tr.name, trp.permission_id",
                (provider, subject),
            )
            .await
            .map_err(|error| {
                tracing::error!(
                    ?error,
                    provider,
                    subject,
                    "tenant authorization query failed"
                );
                ApiError::Database
            })?;
        let mut tenants: Vec<repositories::TenantAuthorization> = Vec::new();
        while let Some(row) = tenant_rows.next().await.map_err(|_| ApiError::Database)? {
            let tenant_id: String = row.get(0).map_err(|_| ApiError::Database)?;
            let index = match tenants
                .iter()
                .position(|tenant| tenant.tenant_id == tenant_id)
            {
                Some(index) => index,
                None => {
                    tenants.push(repositories::TenantAuthorization {
                        tenant_id,
                        tenant_slug: row.get(1).map_err(|_| ApiError::Database)?,
                        roles: Vec::new(),
                        permissions: Vec::new(),
                    });
                    tenants.len() - 1
                }
            };
            tenants[index]
                .roles
                .push(row.get(2).map_err(|_| ApiError::Database)?);
            if let Some(permission) = row
                .get::<Option<String>>(3)
                .map_err(|_| ApiError::Database)?
            {
                tenants[index].permissions.push(permission);
            }
        }
        for tenant in &mut tenants {
            tenant.roles.sort();
            tenant.roles.dedup();
            tenant.permissions.sort();
            tenant.permissions.dedup();
        }
        Ok(repositories::AuthorizationRecord {
            global_roles,
            global_permissions,
            tenants,
        })
    }

    pub async fn grant_global_role(
        &self,
        provider: &str,
        subject: &str,
        role: &str,
    ) -> Result<(), ApiError> {
        let affected = self
            .conn
            .execute(
                "INSERT OR IGNORE INTO subject_global_roles(subject_id, role_id, created_at)
             SELECT id, ?3, ?4 FROM oidc_subjects WHERE provider = ?1 AND subject = ?2",
                (provider, subject, role, now()),
            )
            .await
            .map_err(|error| {
                tracing::error!(?error, provider, subject, role, "global role grant failed");
                ApiError::Database
            })?;
        if affected == 0 {
            let auth = self.authorization_for_subject(provider, subject).await?;
            if !auth.global_roles.iter().any(|candidate| candidate == role) {
                return Err(ApiError::NotFound);
            }
        }
        Ok(())
    }

    pub async fn create_tenant(
        &self,
        id: &str,
        slug: &str,
        name: &str,
    ) -> Result<repositories::TenantRecord, ApiError> {
        let timestamp = now();
        self.conn.execute(
            "INSERT INTO tenants(id, slug, name, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5)",
            (id, slug, name, timestamp.as_str(), timestamp.as_str()),
        ).await.map_err(|error| {
            tracing::error!(?error, id, slug, "tenant insert failed");
            ApiError::BadRequest
        })?;
        for role in ["manager", "user", "viewer"] {
            let role_id = format!("{id}:{role}");
            self.conn
                .execute(
                    "INSERT INTO tenant_roles(id, tenant_id, name, description, created_at)
                 SELECT ?1, ?2, name, description, ?4 FROM tenant_role_templates WHERE name = ?3",
                    (role_id.as_str(), id, role, timestamp.as_str()),
                )
                .await
                .map_err(|error| {
                    tracing::error!(?error, id, role, "tenant role creation failed");
                    ApiError::Database
                })?;
            self.conn.execute(
                "INSERT INTO tenant_role_permissions(role_id, permission_id, created_at)
                 SELECT ?1, permission_id, ?3 FROM tenant_role_template_permissions WHERE role_name = ?2",
                (role_id.as_str(), role, timestamp.as_str()),
            ).await.map_err(|error| {
                tracing::error!(?error, id, role, "tenant role permission creation failed");
                ApiError::Database
            })?;
        }
        self.tenant(id).await
    }

    pub async fn tenant(&self, id: &str) -> Result<repositories::TenantRecord, ApiError> {
        let mut rows = self
            .conn
            .query(
                "SELECT id, slug, name, created_at, updated_at FROM tenants WHERE id = ?1",
                (id,),
            )
            .await
            .map_err(|_| ApiError::Database)?;
        let row = rows
            .next()
            .await
            .map_err(|_| ApiError::Database)?
            .ok_or(ApiError::NotFound)?;
        tenant_from_row(row)
    }

    pub async fn list_tenants(&self) -> Result<Vec<repositories::TenantRecord>, ApiError> {
        let mut rows = self
            .conn
            .query(
                "SELECT id, slug, name, created_at, updated_at FROM tenants ORDER BY slug",
                (),
            )
            .await
            .map_err(|_| ApiError::Database)?;
        let mut tenants = Vec::new();
        while let Some(row) = rows.next().await.map_err(|_| ApiError::Database)? {
            tenants.push(tenant_from_row(row)?);
        }
        Ok(tenants)
    }

    pub async fn grant_tenant_role(
        &self,
        tenant_id: &str,
        provider: &str,
        subject: &str,
        role: &str,
        granted_by_subject_id: Option<&str>,
    ) -> Result<(), ApiError> {
        let role_id = format!("{tenant_id}:{role}");
        let affected = self.conn.execute(
            "INSERT OR IGNORE INTO tenant_memberships(tenant_id, subject_id, role_id, granted_by_subject_id, created_at)
             SELECT ?1, id, ?4, ?5, ?6 FROM oidc_subjects WHERE provider = ?2 AND subject = ?3",
            (tenant_id, provider, subject, role_id.as_str(), granted_by_subject_id, now()),
        ).await.map_err(|error| {
            tracing::error!(?error, tenant_id, provider, subject, role, "tenant role grant failed");
            ApiError::BadRequest
        })?;
        if affected == 0 {
            return Err(ApiError::NotFound);
        }
        Ok(())
    }

    pub async fn append_audit_event(
        &self,
        event: &repositories::AuditEventInput,
    ) -> Result<repositories::AuditEventRecord, ApiError> {
        let id = uuid::Uuid::new_v4().to_string();
        self.conn.execute(
            "INSERT INTO audit_events(id, actor_provider, actor_subject, action, target_type, target_id, metadata_json, created_at, tenant_id, outcome, reason, request_id)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            (
                id.as_str(), event.actor_provider.as_deref(), event.actor_subject.as_deref(),
                event.action.as_str(), event.target_type.as_str(), event.target_id.as_deref(),
                event.metadata_json.as_deref(), now(), event.tenant_id.as_deref(),
                event.outcome.as_str(), event.reason.as_deref(), event.request_id.as_deref(),
            ),
        ).await.map_err(|error| {
            tracing::error!(?error, action = %event.action, "audit event insert failed");
            ApiError::Database
        })?;
        self.audit_event(&id).await
    }

    pub async fn audit_event(&self, id: &str) -> Result<repositories::AuditEventRecord, ApiError> {
        let mut rows = self.conn.query(
            "SELECT id, actor_provider, actor_subject, action, target_type, target_id, tenant_id, outcome, reason, request_id, metadata_json, created_at FROM audit_events WHERE id = ?1",
            (id,),
        ).await.map_err(|error| {
            tracing::error!(?error, id, "audit event query failed");
            ApiError::Database
        })?;
        let row = rows
            .next()
            .await
            .map_err(|_| ApiError::Database)?
            .ok_or(ApiError::NotFound)?;
        audit_event_from_row(row)
    }

    pub async fn list_audit_events(
        &self,
        filter: &repositories::AuditEventFilter,
    ) -> Result<Vec<repositories::AuditEventRecord>, ApiError> {
        let limit = i64::from(filter.limit.clamp(1, 500));
        let mut rows = self.conn.query(
            "SELECT id, actor_provider, actor_subject, action, target_type, target_id, tenant_id, outcome, reason, request_id, metadata_json, created_at
             FROM audit_events
             WHERE (?1 IS NULL OR actor_subject = ?1)
               AND (?2 IS NULL OR action = ?2)
               AND (?3 IS NULL OR tenant_id = ?3)
             ORDER BY created_at DESC LIMIT ?4",
            (filter.actor_subject.as_deref(), filter.action.as_deref(), filter.tenant_id.as_deref(), limit),
        ).await.map_err(|error| {
            tracing::error!(?error, "audit event list query failed");
            ApiError::Database
        })?;
        let mut events = Vec::new();
        while let Some(row) = rows.next().await.map_err(|_| ApiError::Database)? {
            events.push(audit_event_from_row(row)?);
        }
        Ok(events)
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

    pub async fn enroll_edge_node(
        &self,
        node: &repositories::EdgeNodeInput,
        token_hash: &str,
    ) -> Result<repositories::EdgeNodeRecord, ApiError> {
        let mut conn = self.conn.clone();
        let transaction = conn.transaction().await.map_err(|error| {
            tracing::error!(?error, node_id = %node.id, "edge enrollment transaction failed");
            ApiError::Database
        })?;
        let timestamp = now();
        transaction.execute(
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
                node.id.as_str(), node.name.as_str(), node.hostname.as_str(),
                node.architecture.as_str(), node.os.as_str(), node.kernel.as_str(),
                node.speiche_version.as_str(), node.health_status.as_str(),
                node.inventory_json.as_deref(), timestamp.as_str(), timestamp.as_str(),
                timestamp.as_str(),
            ),
        ).await.map_err(|error| {
            tracing::error!(?error, node_id = %node.id, "edge enrollment node upsert failed");
            ApiError::Database
        })?;
        let credential_insert = transaction
            .execute(
                "INSERT INTO edge_node_credentials(edge_node_id, token_hash, created_at)
             VALUES (?1, ?2, ?3)",
                (node.id.as_str(), token_hash, timestamp.as_str()),
            )
            .await;
        if let Err(error) = credential_insert {
            tracing::warn!(?error, node_id = %node.id, "edge node already enrolled");
            transaction.rollback().await.map_err(|rollback_error| {
                tracing::error!(?rollback_error, node_id = %node.id, "edge enrollment rollback failed");
                ApiError::Database
            })?;
            return Err(ApiError::Forbidden);
        }
        transaction.commit().await.map_err(|error| {
            tracing::error!(?error, node_id = %node.id, "edge enrollment commit failed");
            ApiError::Database
        })?;
        self.edge_node(&node.id).await
    }

    pub async fn authenticate_edge_node(
        &self,
        node_id: &str,
        token_hash: &str,
    ) -> Result<(), ApiError> {
        let affected = self
            .conn
            .execute(
                "UPDATE edge_node_credentials SET last_used_at = ?3
             WHERE edge_node_id = ?1 AND token_hash = ?2 AND revoked_at IS NULL",
                (node_id, token_hash, now()),
            )
            .await
            .map_err(|error| {
                tracing::error!(?error, node_id, "edge credential verification failed");
                ApiError::Database
            })?;
        if affected == 1 {
            Ok(())
        } else {
            Err(ApiError::Forbidden)
        }
    }

    pub async fn revoke_edge_credential(&self, node_id: &str) -> Result<(), ApiError> {
        let affected = self
            .conn
            .execute(
                "DELETE FROM edge_node_credentials WHERE edge_node_id = ?1",
                (node_id,),
            )
            .await
            .map_err(|error| {
                tracing::error!(?error, node_id, "edge credential revocation failed");
                ApiError::Database
            })?;
        if affected == 1 {
            Ok(())
        } else {
            Err(ApiError::NotFound)
        }
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
                "INSERT INTO device_clients(id, label, owner_scope, client_id, enabled, created_at, updated_at, owner_provider, owner_subject, tenant_id)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                (
                    client.id.as_str(),
                    client.label.as_str(),
                    client.owner_scope.as_str(),
                    client.client_id.as_str(),
                    client.enabled as i64,
                    now.as_str(),
                    now.as_str(),
                    client.owner_provider.as_deref(),
                    client.owner_subject.as_deref(),
                    client.tenant_id.as_deref(),
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
                "UPDATE device_clients SET label = ?2, owner_scope = ?3, client_id = ?4, enabled = ?5, updated_at = ?6, owner_provider = ?7, owner_subject = ?8, tenant_id = ?9
                 WHERE id = ?1",
                (
                    client.id.as_str(),
                    client.label.as_str(),
                    client.owner_scope.as_str(),
                    client.client_id.as_str(),
                    client.enabled as i64,
                    now(),
                    client.owner_provider.as_deref(),
                    client.owner_subject.as_deref(),
                    client.tenant_id.as_deref(),
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
                "SELECT id, label, owner_scope, client_id, enabled, created_at, updated_at, owner_provider, owner_subject, tenant_id
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
                "SELECT id, label, owner_scope, client_id, enabled, created_at, updated_at, owner_provider, owner_subject, tenant_id
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
                "SELECT id, label, owner_scope, client_id, enabled, created_at, updated_at, owner_provider, owner_subject, tenant_id
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

    pub async fn ingest_dns_stat_buckets(
        &self,
        batch_id: &str,
        edge_node_id: &str,
        collected_through: &str,
        buckets: &[repositories::DnsStatBucketInput],
        retention_cutoff: &str,
    ) -> Result<bool, ApiError> {
        let mut conn = self.conn.clone();
        let transaction = conn.transaction().await.map_err(|error| {
            tracing::error!(?error, batch_id, "telemetry transaction start failed");
            ApiError::Database
        })?;
        let inserted = transaction.execute(
            "INSERT OR IGNORE INTO telemetry_ingestion_batches(id, edge_node_id, collected_through, created_at)
             VALUES (?1, ?2, ?3, ?4)",
            (batch_id, edge_node_id, collected_through, now()),
        ).await.map_err(|error| {
            tracing::error!(?error, batch_id, edge_node_id, "telemetry batch insert failed");
            ApiError::BadRequest
        })?;
        if inserted == 0 {
            transaction.commit().await.map_err(|_| ApiError::Database)?;
            return Ok(false);
        }
        for bucket in buckets {
            transaction.execute(
                "INSERT INTO dns_stat_buckets(edge_node_id, device_client_id, bucket_start, bucket_seconds, queries, blocked, cached, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
                 ON CONFLICT(edge_node_id, device_client_id, bucket_start, bucket_seconds) DO UPDATE SET
                   queries = dns_stat_buckets.queries + excluded.queries,
                   blocked = dns_stat_buckets.blocked + excluded.blocked,
                   cached = dns_stat_buckets.cached + excluded.cached,
                   updated_at = excluded.updated_at",
                (
                    edge_node_id,
                    bucket.device_client_id.as_str(),
                    bucket.bucket_start.as_str(),
                    bucket.bucket_seconds,
                    bucket.queries,
                    bucket.blocked,
                    bucket.cached,
                    now(),
                ),
            ).await.map_err(|error| {
                tracing::error!(?error, batch_id, edge_node_id, "DNS statistic bucket upsert failed");
                ApiError::BadRequest
            })?;
        }
        transaction
            .execute(
                "DELETE FROM dns_stat_buckets WHERE bucket_start < ?1",
                (retention_cutoff,),
            )
            .await
            .map_err(|error| {
                tracing::error!(?error, "DNS statistic retention cleanup failed");
                ApiError::Database
            })?;
        transaction
            .execute(
                "DELETE FROM telemetry_ingestion_batches WHERE created_at < ?1",
                (retention_cutoff,),
            )
            .await
            .map_err(|error| {
                tracing::error!(?error, "telemetry batch retention cleanup failed");
                ApiError::Database
            })?;
        transaction.commit().await.map_err(|error| {
            tracing::error!(?error, batch_id, "telemetry transaction commit failed");
            ApiError::Database
        })?;
        Ok(true)
    }

    pub async fn list_dns_stat_buckets(
        &self,
        from: &str,
        to: &str,
    ) -> Result<Vec<repositories::DnsStatBucketRecord>, ApiError> {
        let mut rows = self.conn.query(
            "SELECT edge_node_id, device_client_id, bucket_start, bucket_seconds, queries, blocked, cached
             FROM dns_stat_buckets WHERE bucket_start >= ?1 AND bucket_start < ?2
             ORDER BY bucket_start ASC, device_client_id ASC, edge_node_id ASC",
            (from, to),
        ).await.map_err(|error| {
            tracing::error!(?error, from, to, "DNS statistic bucket query failed");
            ApiError::Database
        })?;
        let mut buckets = Vec::new();
        while let Some(row) = rows.next().await.map_err(|_| ApiError::Database)? {
            buckets.push(repositories::DnsStatBucketRecord {
                edge_node_id: row.get(0).map_err(|_| ApiError::Database)?,
                device_client_id: row.get(1).map_err(|_| ApiError::Database)?,
                bucket_start: row.get(2).map_err(|_| ApiError::Database)?,
                bucket_seconds: row.get(3).map_err(|_| ApiError::Database)?,
                queries: row.get(4).map_err(|_| ApiError::Database)?,
                blocked: row.get(5).map_err(|_| ApiError::Database)?,
                cached: row.get(6).map_err(|_| ApiError::Database)?,
            });
        }
        Ok(buckets)
    }

    pub async fn create_dns_policy(
        &self,
        policy: &repositories::DnsPolicyInput,
    ) -> Result<repositories::DnsPolicyRecord, ApiError> {
        let now = now();
        self.conn
            .execute(
                "INSERT INTO dns_policies(id, name, blocked_domains_json, enabled, created_at, updated_at, tenant_id)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                (
                    policy.id.as_str(),
                    policy.name.as_str(),
                    policy.blocked_domains_json.as_str(),
                    policy.enabled as i64,
                    now.as_str(),
                    now.as_str(),
                    policy.tenant_id.as_deref(),
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
                "UPDATE dns_policies SET name = ?2, blocked_domains_json = ?3, enabled = ?4, updated_at = ?5, tenant_id = ?6
                 WHERE id = ?1",
                (
                    policy.id.as_str(),
                    policy.name.as_str(),
                    policy.blocked_domains_json.as_str(),
                    policy.enabled as i64,
                    now(),
                    policy.tenant_id.as_deref(),
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
                "SELECT id, name, blocked_domains_json, enabled, last_applied_at, created_at, updated_at, tenant_id
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
                "SELECT id, name, blocked_domains_json, enabled, last_applied_at, created_at, updated_at, tenant_id
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
        owner_provider: row.get(7).map_err(|_| ApiError::Database)?,
        owner_subject: row.get(8).map_err(|_| ApiError::Database)?,
        tenant_id: row.get(9).map_err(|_| ApiError::Database)?,
    })
}

fn audit_event_from_row(row: turso::Row) -> Result<repositories::AuditEventRecord, ApiError> {
    Ok(repositories::AuditEventRecord {
        id: row.get(0).map_err(|_| ApiError::Database)?,
        actor_provider: row.get(1).map_err(|_| ApiError::Database)?,
        actor_subject: row.get(2).map_err(|_| ApiError::Database)?,
        action: row.get(3).map_err(|_| ApiError::Database)?,
        target_type: row.get(4).map_err(|_| ApiError::Database)?,
        target_id: row.get(5).map_err(|_| ApiError::Database)?,
        tenant_id: row.get(6).map_err(|_| ApiError::Database)?,
        outcome: row.get(7).map_err(|_| ApiError::Database)?,
        reason: row.get(8).map_err(|_| ApiError::Database)?,
        request_id: row.get(9).map_err(|_| ApiError::Database)?,
        metadata_json: row.get(10).map_err(|_| ApiError::Database)?,
        created_at: row.get(11).map_err(|_| ApiError::Database)?,
    })
}

fn tenant_from_row(row: turso::Row) -> Result<repositories::TenantRecord, ApiError> {
    Ok(repositories::TenantRecord {
        id: row.get(0).map_err(|_| ApiError::Database)?,
        slug: row.get(1).map_err(|_| ApiError::Database)?,
        name: row.get(2).map_err(|_| ApiError::Database)?,
        created_at: row.get(3).map_err(|_| ApiError::Database)?,
        updated_at: row.get(4).map_err(|_| ApiError::Database)?,
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
        tenant_id: row.get(7).map_err(|_| ApiError::Database)?,
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
    use crate::db::repositories::{
        AuditEventFilter, AuditEventInput, DeviceClientInput, DnsPolicyInput, DnsStatBucketInput,
        EdgeNodeInput, SubjectInput,
    };
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
            owner_provider: Some("issuer.example".to_string()),
            owner_subject: Some("alice".to_string()),
            tenant_id: None,
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
            tenant_id: None,
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

    #[tokio::test]
    async fn audit_events_are_durable_and_filterable() {
        let db = test_db().await;
        for (subject, action, tenant) in [
            ("alice", "policy.update", Some("gray")),
            ("bob", "edge.read", None),
            ("alice", "querylog.read", Some("gray")),
        ] {
            db.append_audit_event(&AuditEventInput {
                actor_provider: Some("oidc".to_string()),
                actor_subject: Some(subject.to_string()),
                action: action.to_string(),
                target_type: "test".to_string(),
                target_id: None,
                tenant_id: tenant.map(str::to_string),
                outcome: "allowed".to_string(),
                reason: None,
                request_id: None,
                metadata_json: None,
            })
            .await
            .expect("append");
        }

        let alice = db
            .list_audit_events(&AuditEventFilter {
                actor_subject: Some("alice".to_string()),
                limit: 100,
                ..Default::default()
            })
            .await
            .expect("filter actor");
        assert_eq!(alice.len(), 2);

        let tenant = db
            .list_audit_events(&AuditEventFilter {
                tenant_id: Some("gray".to_string()),
                limit: 1,
                ..Default::default()
            })
            .await
            .expect("filter tenant");
        assert_eq!(tenant.len(), 1);
        assert_eq!(tenant[0].tenant_id.as_deref(), Some("gray"));

        db.migrate().await.expect("migration is idempotent");
    }

    #[tokio::test]
    async fn native_authorization_resolves_global_and_tenant_grants() {
        let db = test_db().await;
        db.upsert_subject(&SubjectInput {
            id: "subject-1".to_string(),
            provider: "issuer.example".to_string(),
            subject: "alice".to_string(),
            email: None,
            display_name: None,
        })
        .await
        .expect("subject");

        let baseline = db
            .authorization_for_subject("issuer.example", "alice")
            .await
            .expect("baseline authorization");
        assert_eq!(baseline.global_roles, vec!["baseline"]);
        assert!(!baseline
            .global_permissions
            .iter()
            .any(|permission| permission == "edge.read"));

        db.grant_global_role("issuer.example", "alice", "admin")
            .await
            .expect("admin grant");
        db.create_tenant("tenant-gray", "gray", "Gray")
            .await
            .expect("tenant");
        db.grant_tenant_role("tenant-gray", "issuer.example", "alice", "manager", None)
            .await
            .expect("tenant grant");

        let authorized = db
            .authorization_for_subject("issuer.example", "alice")
            .await
            .expect("authorization");
        assert!(authorized
            .global_permissions
            .iter()
            .any(|permission| permission == "platform.admin"));
        assert_eq!(authorized.tenants.len(), 1);
        assert!(authorized.tenants[0]
            .permissions
            .iter()
            .any(|permission| permission == "tenant.manage"));
    }

    #[tokio::test]
    async fn telemetry_batches_are_atomic_idempotent_and_additive() {
        let db = test_db().await;
        db.upsert_edge_node(&EdgeNodeInput {
            id: "edge-1".to_string(),
            name: "edge".to_string(),
            hostname: "edge.local".to_string(),
            architecture: "aarch64".to_string(),
            os: "Linux".to_string(),
            kernel: "test".to_string(),
            speiche_version: "test".to_string(),
            health_status: "healthy".to_string(),
            inventory_json: None,
        })
        .await
        .expect("edge");
        db.create_device_client(&client_input("dc-1", "phone", true))
            .await
            .expect("client");
        let bucket = DnsStatBucketInput {
            device_client_id: "dc-1".to_string(),
            bucket_start: "2026-07-12T12:00:00Z".to_string(),
            bucket_seconds: 300,
            queries: 5,
            blocked: 2,
            cached: 1,
        };
        assert!(db
            .ingest_dns_stat_buckets(
                "batch-1",
                "edge-1",
                "2026-07-12T12:05:00Z",
                std::slice::from_ref(&bucket),
                "2026-06-12T00:00:00Z",
            )
            .await
            .expect("first batch"));
        assert!(!db
            .ingest_dns_stat_buckets(
                "batch-1",
                "edge-1",
                "2026-07-12T12:05:00Z",
                std::slice::from_ref(&bucket),
                "2026-06-12T00:00:00Z",
            )
            .await
            .expect("duplicate batch"));
        let mut increment = bucket.clone();
        increment.queries = 2;
        increment.blocked = 1;
        increment.cached = 0;
        assert!(db
            .ingest_dns_stat_buckets(
                "batch-2",
                "edge-1",
                "2026-07-12T12:05:30Z",
                &[increment],
                "2026-06-12T00:00:00Z",
            )
            .await
            .expect("second batch"));
        let buckets = db
            .list_dns_stat_buckets("2026-07-12T00:00:00Z", "2026-07-13T00:00:00Z")
            .await
            .expect("history");
        assert_eq!(buckets.len(), 1);
        assert_eq!(buckets[0].queries, 7);
        assert_eq!(buckets[0].blocked, 3);
        assert_eq!(buckets[0].cached, 1);
    }

    #[tokio::test]
    async fn edge_credentials_are_node_bound_and_reenrollment_is_explicit() {
        let db = test_db().await;
        let node = EdgeNodeInput {
            id: "edge-secure".to_string(),
            name: "edge".to_string(),
            hostname: "edge.local".to_string(),
            architecture: "aarch64".to_string(),
            os: "Linux".to_string(),
            kernel: "test".to_string(),
            speiche_version: "test".to_string(),
            health_status: "healthy".to_string(),
            inventory_json: None,
        };
        db.enroll_edge_node(&node, "hash-one")
            .await
            .expect("initial enrollment");
        db.authenticate_edge_node("edge-secure", "hash-one")
            .await
            .expect("credential");
        assert!(matches!(
            db.authenticate_edge_node("edge-secure", "wrong").await,
            Err(ApiError::Forbidden)
        ));
        assert!(matches!(
            db.enroll_edge_node(&node, "hash-two").await,
            Err(ApiError::Forbidden)
        ));
        db.revoke_edge_credential("edge-secure")
            .await
            .expect("revoke");
        assert!(matches!(
            db.authenticate_edge_node("edge-secure", "hash-one").await,
            Err(ApiError::Forbidden)
        ));
        db.enroll_edge_node(&node, "hash-two")
            .await
            .expect("explicit reenrollment");
    }
}
