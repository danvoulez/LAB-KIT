use crate::{
    hash::{hash_bytes, hash_value},
    model::LogLineAct,
};
use anyhow::{anyhow, bail, Context, Result};
use chrono::{DateTime, Utc};
use serde_json::{json, Value};
use std::{fs, path::PathBuf};
use tokio_postgres::{Client, NoTls, Row};
use uuid::Uuid;

pub async fn connect_required(database_url: Option<&str>) -> Result<Client> {
    let url = database_url.ok_or_else(|| {
        anyhow!("SUPABASE_DB_URL or --database-url is required for semantic spine operations")
    })?;
    connect(url).await
}

pub async fn connect(url: &str) -> Result<Client> {
    let (client, connection) = tokio_postgres::connect(url, NoTls).await?;
    tokio::spawn(async move {
        if let Err(err) = connection.await {
            eprintln!("postgres connection error: {err}");
        }
    });
    Ok(client)
}

pub async fn run_migrations(client: &Client) -> Result<()> {
    client
        .batch_execute(
            "create schema if not exists ops;
             create table if not exists ops.schema_migrations (
               version text primary key,
               checksum text not null,
               applied_at timestamptz not null default now()
             );",
        )
        .await?;

    let mut migrations = Vec::<PathBuf>::new();
    for entry in fs::read_dir("supabase/migrations")? {
        let path = entry?.path();
        if path.extension().and_then(|ext| ext.to_str()) == Some("sql") {
            migrations.push(path);
        }
    }
    migrations.sort();

    for path in migrations {
        let version = path
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| anyhow!("invalid migration filename: {}", path.display()))?
            .to_owned();
        let sql = fs::read_to_string(&path)?;
        let checksum = hash_bytes(sql.as_bytes());
        if let Some(row) = client
            .query_opt(
                "select checksum from ops.schema_migrations where version = $1",
                &[&version],
            )
            .await?
        {
            let applied_checksum: String = row.get(0);
            if applied_checksum != checksum {
                bail!(
                    "migration checksum mismatch for {version}: db has {applied_checksum}, file has {checksum}"
                );
            }
            continue;
        }
        client
            .batch_execute(&sql)
            .await
            .with_context(|| format!("migration {}", path.display()))?;
        client
            .execute(
                "insert into ops.schema_migrations (version, checksum) values ($1, $2)",
                &[&version, &checksum],
            )
            .await?;
    }
    Ok(())
}

pub async fn run_projector_refresh(client: &Client) -> Result<()> {
    client
        .batch_execute(include_str!(
            "../supabase/migrations/0009_functions_projectors.sql"
        ))
        .await?;
    Ok(())
}

pub async fn insert_act(client: &Client, act: &LogLineAct) -> Result<Uuid> {
    act.validate()?;
    let tuple_hash = hash_value(&json!([
        act.who,
        act.did,
        act.this_,
        act.when.unwrap_or_else(Utc::now).to_rfc3339(),
        act.status
    ]));
    let content_hash = hash_value(&serde_json::to_value(act)?);
    let when = act.when.unwrap_or_else(Utc::now);
    let evidence_state = act
        .evidence_state
        .clone()
        .unwrap_or_else(|| "declared".into());
    let promotion_state = act
        .promotion_state
        .clone()
        .unwrap_or_else(|| "candidate".into());
    let row = client.query_one("insert into ops.logline_acts (who,did,this,\"when\",confirmed_by,if_ok,if_doubt,if_not,status,runtime_envelope,tuple_hash,content_hash,previous_act_refs,evidence_state,promotion_state) values ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15) returning id", &[&act.who,&act.did,&act.this_,&when,&act.confirmed_by,&act.if_ok,&act.if_doubt,&act.if_not,&act.status,&act.runtime_envelope,&tuple_hash,&content_hash,&act.previous_act_refs,&evidence_state,&promotion_state]).await?;
    Ok(row.get(0))
}

pub async fn relation_exists(client: &Client, relation: &str) -> Result<bool> {
    validate_relation_name(relation)?;
    let row = client
        .query_one("select to_regclass($1) is not null", &[&relation])
        .await?;
    Ok(row.get(0))
}

pub async fn count_relation(client: &Client, relation: &str) -> Result<i64> {
    validate_relation_name(relation)?;
    let row = client
        .query_one(&format!("select count(*)::bigint from {relation}"), &[])
        .await?;
    Ok(row.get(0))
}

pub async fn list_relation(client: &Client, relation: &str, limit: i64) -> Result<Vec<Value>> {
    validate_relation_name(relation)?;
    let limit = limit.clamp(1, 500);
    let sql = format!("select row_to_json(t) from (select * from {relation} limit {limit}) t");
    let rows = client.query(&sql, &[]).await?;
    Ok(rows.into_iter().map(|row| row.get(0)).collect())
}

pub fn row_to_json(row: &Row) -> Value {
    json!({
        "id": row.get::<_, Uuid>(0),
        "who": row.get::<_, String>(1),
        "did": row.get::<_, String>(2),
        "this": row.get::<_, Value>(3),
        "when": row.get::<_, DateTime<Utc>>(4),
        "confirmed_by": row.get::<_, Value>(5),
        "if_ok": row.get::<_, Value>(6),
        "if_doubt": row.get::<_, Value>(7),
        "if_not": row.get::<_, Value>(8),
        "status": row.get::<_, String>(9),
        "runtime_envelope": row.get::<_, Value>(10),
        "tuple_hash": row.get::<_, Option<String>>(11),
        "content_hash": row.get::<_, Option<String>>(12),
        "previous_act_refs": row.get::<_, Vec<String>>(13),
        "evidence_state": row.get::<_, String>(14),
        "promotion_state": row.get::<_, String>(15),
        "created_at": row.get::<_, DateTime<Utc>>(16)
    })
}

fn validate_relation_name(relation: &str) -> Result<()> {
    let parts: Vec<_> = relation.split('.').collect();
    if parts.len() != 2 {
        bail!("relation must be schema.name");
    }
    for part in parts {
        if part.is_empty()
            || !part
                .chars()
                .all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
        {
            bail!("unsafe relation name: {relation}");
        }
    }
    Ok(())
}
