use crate::{
    error::{ApiError, ApiResult},
    handlers::auth::RequestAuthContext,
    middleware::TenantContext,
    state::AppState,
};
use serde::{de::DeserializeOwned, Serialize};
use serde_json::{json, Value};
use sqlx::{PgPool, Postgres, Row, Transaction};
use uuid::Uuid;

#[derive(Clone)]
pub struct Store {
    pub pool: PgPool,
    pub tenant: Uuid,
    pub workspace: Uuid,
    pub actor: Uuid,
}
pub fn bad(error: impl ToString) -> ApiError {
    ApiError::BadRequest(error.to_string())
}
pub fn database(_error: impl ToString) -> ApiError {
    tracing::error!("Policy storage operation failed; inspect sanitized database diagnostics");
    ApiError::Internal("Policy storage unavailable".into())
}
impl Store {
    pub async fn scoped(
        state: &AppState,
        ctx: &TenantContext,
        auth: &RequestAuthContext,
    ) -> ApiResult<Self> {
        if !state.auth.config.auth_enabled || state.auth.config.dev_mode {
            return Err(ApiError::forbidden_reason(
                "Policy records require real authenticated workspace membership",
            ));
        }
        crate::services::request_authorization::membership_role(
            state,
            &auth.user_id,
            ctx.tenant_id.as_deref(),
            ctx.workspace_id.as_deref(),
        )
        .await?;
        Ok(Self {
            pool: state
                .pg_pool
                .clone()
                .ok_or_else(|| ApiError::Internal("PostgreSQL required for Tacitus".into()))?,
            tenant: Uuid::parse_str(
                ctx.tenant_id
                    .as_deref()
                    .ok_or_else(|| bad("Tenant required"))?,
            )
            .map_err(bad)?,
            workspace: Uuid::parse_str(
                ctx.workspace_id
                    .as_deref()
                    .ok_or_else(|| bad("Workspace required"))?,
            )
            .map_err(bad)?,
            actor: Uuid::parse_str(&auth.user_id).map_err(bad)?,
        })
    }
    pub async fn transaction(&self) -> ApiResult<Transaction<'_, Postgres>> {
        let mut tx = self.pool.begin().await.map_err(database)?;
        sqlx::query(
            "SELECT set_config('app.tenant_id',$1,true),set_config('app.workspace_id',$2,true)",
        )
        .bind(self.tenant.to_string())
        .bind(self.workspace.to_string())
        .execute(&mut *tx)
        .await
        .map_err(database)?;
        Ok(tx)
    }
    pub async fn records<T: DeserializeOwned>(&self, kind: &str) -> ApiResult<Vec<T>> {
        let mut tx = self.transaction().await?;
        let rows=sqlx::query("SELECT payload FROM tacitus_records WHERE tenant_id=$1 AND workspace_id=$2 AND kind=$3 ORDER BY seq DESC LIMIT 2049")
            .bind(self.tenant).bind(self.workspace).bind(kind).fetch_all(&mut *tx).await.map_err(database)?;
        if rows.len() > 2048 {
            return Err(ApiError::Conflict("Policy MVP record bound reached; archive or migrate before continuing. Records were not silently omitted".into()));
        }
        rows.into_iter()
            .map(|r| serde_json::from_value(r.get::<Value, _>("payload")).map_err(database))
            .collect()
    }
    pub async fn record<T: DeserializeOwned>(
        &self,
        kind: &str,
        id: Uuid,
        version: Option<u32>,
    ) -> ApiResult<T> {
        let mut tx = self.transaction().await?;
        let row=sqlx::query("SELECT payload FROM tacitus_records WHERE tenant_id=$1 AND workspace_id=$2 AND kind=$3 AND id=$4 AND ($5::int IS NULL OR version=$5) ORDER BY version DESC LIMIT 1")
            .bind(self.tenant).bind(self.workspace).bind(kind).bind(id).bind(version.map(|v|v as i32)).fetch_optional(&mut *tx).await.map_err(database)?.ok_or_else(||ApiError::NotFound("Policy record unavailable".into()))?;
        serde_json::from_value(row.get::<Value, _>("payload")).map_err(database)
    }
    pub async fn complete(
        &self,
        id: Uuid,
        result: Value,
        assertions: &[tacitus_core::policy::PolicyAssertion],
    ) -> ApiResult<()> {
        let mut tx = self.transaction().await?;
        let status:Option<String>=sqlx::query_scalar("SELECT status FROM tacitus_analyses WHERE id=$1 AND tenant_id=$2 AND workspace_id=$3 FOR UPDATE")
            .bind(id).bind(self.tenant).bind(self.workspace).fetch_optional(&mut *tx).await.map_err(database)?;
        if status.as_deref() != Some("generating") {
            return Ok(());
        }
        for a in assertions {
            self.insert_in(&mut tx, "assertion", &a.id, a.version, a)
                .await?;
        }
        sqlx::query("UPDATE tacitus_analyses SET status='completed',result=$4,updated_at=now() WHERE id=$1 AND tenant_id=$2 AND workspace_id=$3")
            .bind(id).bind(self.tenant).bind(self.workspace).bind(result).execute(&mut *tx).await.map_err(database)?;
        tx.commit().await.map_err(database)
    }
    /// Serialize version-bound promotion with source revisions and document withdrawal.
    pub async fn ensure_evidence_in(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        spans: &[tacitus_core::Span],
    ) -> ApiResult<()> {
        let mut ids = std::collections::BTreeSet::new();
        for span in spans {
            let (id, _) = span
                .source
                .rsplit_once('@')
                .ok_or_else(|| bad("Invalid evidence identity"))?;
            ids.insert(id.to_string());
        }
        for id in ids {
            sqlx::query("SELECT pg_advisory_xact_lock(hashtextextended($1,0))")
                .bind(format!("{}:{}:{id}", self.tenant, self.workspace))
                .execute(&mut **tx)
                .await
                .map_err(database)?;
        }
        for span in spans {
            let (id, _) = span
                .source
                .rsplit_once('@')
                .ok_or_else(|| bad("Invalid evidence identity"))?;
            let value:Option<Value>=sqlx::query_scalar("SELECT payload FROM tacitus_records WHERE tenant_id=$1 AND workspace_id=$2 AND kind='source' AND id=$3 ORDER BY version DESC LIMIT 1").bind(self.tenant).bind(self.workspace).bind(Uuid::parse_str(id).map_err(bad)?).fetch_optional(&mut **tx).await.map_err(database)?;
            let source: tacitus_core::policy::SourceRecord =
                serde_json::from_value(value.ok_or_else(|| bad("Evidence unavailable"))?)
                    .map_err(database)?;
            if source.withdrawn || source.restricted || source.passage(span).is_err() {
                return Err(ApiError::Conflict(
                    "Evidence changed or authorization withdrawn; inspect and review again".into(),
                ));
            }
            if let Some(doc) = source.document_id {
                let exists:bool=sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM public.documents WHERE id=$1 AND workspace_id=$2 AND (tenant_id=$3 OR tenant_id IS NULL))").bind(Uuid::parse_str(&doc).map_err(bad)?).bind(self.workspace).bind(self.tenant).fetch_one(&mut **tx).await.map_err(database)?;
                if !exists {
                    return Err(ApiError::forbidden_reason("Evidence document unavailable"));
                }
            }
        }
        Ok(())
    }
    pub async fn insert<T: Serialize>(
        &self,
        kind: &str,
        id: &str,
        version: u32,
        payload: &T,
    ) -> ApiResult<()> {
        let mut tx = self.transaction().await?;
        self.insert_in(&mut tx, kind, id, version, payload).await?;
        tx.commit().await.map_err(database)
    }
    pub async fn insert_in<T: Serialize>(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        kind: &str,
        id: &str,
        version: u32,
        payload: &T,
    ) -> ApiResult<()> {
        sqlx::query("INSERT INTO tacitus_records(tenant_id,workspace_id,kind,id,version,payload) VALUES($1,$2,$3,$4,$5,$6)")
            .bind(self.tenant).bind(self.workspace).bind(kind).bind(Uuid::parse_str(id).map_err(bad)?).bind(version as i32).bind(serde_json::to_value(payload).map_err(database)?)
            .execute(&mut **tx).await.map_err(database)?;
        Ok(())
    }
    pub async fn analysis(&self, id: Uuid) -> ApiResult<Value> {
        let mut tx = self.transaction().await?;
        let row=sqlx::query("SELECT id,status,request,context,result,error,created_at,updated_at FROM tacitus_analyses WHERE id=$1 AND tenant_id=$2 AND workspace_id=$3")
            .bind(id).bind(self.tenant).bind(self.workspace).fetch_optional(&mut *tx).await.map_err(database)?.ok_or_else(||ApiError::NotFound("Analysis unavailable".into()))?;
        Ok(
            json!({"id":row.get::<Uuid,_>("id"),"status":row.get::<String,_>("status"),"request":row.get::<Value,_>("request"),"context":row.get::<Option<Value>,_>("context"),"result":row.get::<Option<Value>,_>("result"),"error":row.get::<Option<String>,_>("error"),"created_at":row.get::<chrono::DateTime<chrono::Utc>,_>("created_at"),"updated_at":row.get::<chrono::DateTime<chrono::Utc>,_>("updated_at")}),
        )
    }
    pub async fn transition(
        &self,
        id: Uuid,
        status: &str,
        context: Option<Value>,
        result: Option<Value>,
        error: Option<&str>,
    ) -> ApiResult<bool> {
        let mut tx = self.transaction().await?;
        let changed=sqlx::query("UPDATE tacitus_analyses SET status=$4,context=COALESCE($5,context),result=COALESCE($6,result),error=$7,updated_at=now() WHERE id=$1 AND tenant_id=$2 AND workspace_id=$3 AND status NOT IN ('cancelled','completed','failed','interrupted')")
            .bind(id).bind(self.tenant).bind(self.workspace).bind(status).bind(context).bind(result).bind(error).execute(&mut *tx).await.map_err(database)?.rows_affected()>0;
        tx.commit().await.map_err(database)?;
        Ok(changed)
    }
}
