use crate::{
    db, hooks,
    io::{read_act, read_json_or_yaml, read_manifest},
    model::{initial_ghost_act, initial_lab_act, LogLineAct},
    packets,
    schema::validate_json_schema,
};
use anyhow::{bail, Context, Result};
use chrono::Utc;
use serde_json::{json, Value};
use std::path::Path;
use tokio_postgres::Client;
use uuid::Uuid;

pub enum ReportKind {
    DailyExpedition,
    WeeklyReview,
}

pub async fn init(database_url: Option<&str>, manifest_path: &Path, no_seed: bool) -> Result<()> {
    let manifest = read_manifest(manifest_path)?;
    manifest.ensure_spine()?;
    println!("manifest ok: {} ({})", manifest.lab_id, manifest.profile);
    let Some(url) = database_url else {
        println!("no SUPABASE_DB_URL supplied; wrote no semantic state. Run with --database-url or SUPABASE_DB_URL to migrate and seed Supabase.");
        return Ok(());
    };
    let client = db::connect(url).await?;
    db::run_migrations(&client).await?;
    if !no_seed {
        db::insert_act(&client, &initial_lab_act(&manifest)).await?;
        db::insert_act(&client, &initial_ghost_act(&manifest.lab_id)).await?;
    }
    println!("init complete: Supabase act spine ready");
    Ok(())
}

pub async fn doctor(database_url: Option<&str>, json_output: bool) -> Result<()> {
    let mut checks = Vec::new();
    checks.push(json!({"check":"manifest examples", "ok": Path::new("manifests/lab.manifest.example.yaml").exists()}));
    checks.push(json!({"check":"migrations", "ok": Path::new("supabase/migrations/0001_ops_logline_acts.sql").exists()}));
    checks.push(
        json!({"check":"schemas", "ok": Path::new("schemas/logline-act.schema.json").exists()}),
    );
    if let Some(url) = database_url {
        match db::connect(url).await {
            Ok(client) => {
                checks.push(json!({"check":"Supabase/Postgres connection", "ok": true}));
                for relation in ["ops.logline_acts", "audit.v_mobile_today", "lab_observability.ghosts", "evidence.records", "receipts.candidates"] {
                    checks.push(json!({"check": format!("relation {relation}"), "ok": db::relation_exists(&client, relation).await.unwrap_or(false)}));
                }
            }
            Err(err) => checks.push(json!({"check":"Supabase/Postgres connection", "ok": false, "error": err.to_string()})),
        }
    } else {
        checks.push(json!({"check":"SUPABASE_DB_URL", "ok": false, "warning":"semantic commands need Supabase/Postgres"}));
    }
    if json_output {
        println!("{}", serde_json::to_string_pretty(&checks)?);
    } else {
        for check in checks {
            println!("{}", serde_json::to_string(&check)?);
        }
    }
    Ok(())
}

pub async fn status(database_url: Option<&str>) -> Result<()> {
    let client = db::connect_required(database_url).await?;
    println!(
        "{}",
        serde_json::to_string_pretty(&status_value(&client).await?)?
    );
    Ok(())
}

pub async fn status_value(client: &Client) -> Result<Value> {
    Ok(json!({
        "act_spine":"ops.logline_acts",
        "acts": db::count_relation(client, "ops.logline_acts").await.unwrap_or(0),
        "open_ghosts": db::count_relation(client, "lab_observability.ghosts").await.unwrap_or(0),
        "evidence_records": db::count_relation(client, "evidence.records").await.unwrap_or(0),
        "receipt_candidates": db::count_relation(client, "receipts.candidates").await.unwrap_or(0)
    }))
}

pub fn canon_status() -> Result<()> {
    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "foundation_refs":"canon/foundation.refs.yaml",
            "conformance_refs":"canon/conformance.refs.yaml",
            "profile":"profiles/logline-lab.practice.v0.yaml",
            "status":"loaded-by-reference"
        }))?
    );
    Ok(())
}

pub async fn emit_from_file(database_url: Option<&str>, path: &Path) -> Result<()> {
    let client = db::connect_required(database_url).await?;
    let act = read_act(path)?;
    let id = db::insert_act(&client, &act).await?;
    println!(
        "{}",
        serde_json::to_string_pretty(&json!({"inserted":id,"spine":"ops.logline_acts"}))?
    );
    Ok(())
}

pub async fn receipt_prepare(database_url: Option<&str>, path: &Path) -> Result<()> {
    let raw = read_json_or_yaml(path)?;
    validate_json_schema(Path::new("schemas/receipt-candidate.schema.json"), &raw)
        .context("receipt candidate schema validation failed")?;
    let act: LogLineAct = serde_json::from_value(raw)?;
    if act
        .this_
        .get("evidence_refs")
        .and_then(Value::as_array)
        .map(Vec::len)
        .unwrap_or(0)
        == 0
    {
        bail!("receipt candidate blocked: evidence_refs are required");
    }
    let client = db::connect_required(database_url).await?;
    let id = db::insert_act(&client, &act).await?;
    println!("{}", json!({"receipt_candidate": id, "status":"candidate"}));
    Ok(())
}

pub async fn act_get(database_url: Option<&str>, id: Uuid) -> Result<()> {
    let client = db::connect_required(database_url).await?;
    let row = client.query_one("select id, who, did, this, \"when\", confirmed_by, if_ok, if_doubt, if_not, status, runtime_envelope, tuple_hash, content_hash, previous_act_refs, evidence_state, promotion_state, created_at from ops.logline_acts where id=$1", &[&id]).await?;
    println!("{}", serde_json::to_string_pretty(&db::row_to_json(&row))?);
    Ok(())
}

pub async fn ghost_open(
    database_url: Option<&str>,
    key: String,
    what_missing: String,
    source: String,
    who: String,
) -> Result<()> {
    let act = LogLineAct {
        who,
        did: "open_ghost".into(),
        this_: json!({"ghost_key":key,"what_missing":what_missing}),
        when: Some(Utc::now()),
        confirmed_by: json!({"source":source}),
        if_ok: json!({"project":"lab_observability.ghosts"}),
        if_doubt: json!({"carry":true}),
        if_not: json!({"reject":"ghost_without_missing_condition"}),
        status: "open".into(),
        runtime_envelope: json!({"runtime_id":"logline-lab-cli"}),
        previous_act_refs: vec![],
        evidence_state: Some("declared".into()),
        promotion_state: Some("candidate".into()),
    };
    let client = db::connect_required(database_url).await?;
    let id = db::insert_act(&client, &act).await?;
    println!("{}", json!({"ghost_act": id, "spine":"ops.logline_acts"}));
    Ok(())
}

pub async fn list_projection(database_url: Option<&str>, relation: &str) -> Result<()> {
    let client = db::connect_required(database_url).await?;
    let values = db::list_relation(&client, relation, 100).await?;
    println!("{}", serde_json::to_string_pretty(&values)?);
    Ok(())
}

pub async fn report_generate(database_url: Option<&str>, kind: ReportKind) -> Result<()> {
    let client = db::connect_required(database_url).await?;
    let title = match kind {
        ReportKind::DailyExpedition => "Daily Lab Expedition",
        ReportKind::WeeklyReview => "Weekly Lab Review",
    };
    let status = status_value(&client).await?;
    println!("# {title}\n\n- generated_at: {}\n- act_spine: ops.logline_acts\n- acts: {}\n- open_ghosts: {}\n\nThis is an operational report, not a receipt.", Utc::now().to_rfc3339(), status["acts"], status["open_ghosts"]);
    Ok(())
}

pub async fn projector_run(database_url: Option<&str>, name: &str) -> Result<()> {
    let client = db::connect_required(database_url).await?;
    db::run_projector_refresh(&client).await?;
    println!("projectors refreshed: {name}");
    Ok(())
}

pub fn hook_run(hook: &str, payload: Option<&Path>) -> Result<()> {
    let result = hooks::run_hook(hook, payload)?;
    println!("{}", serde_json::to_string_pretty(&result)?);
    Ok(())
}

pub async fn clock_tick(database_url: Option<&str>, kind: &str) -> Result<()> {
    let client = db::connect_required(database_url).await?;
    let act = LogLineAct {
        who: "lab-clock".into(),
        did: "tick".into(),
        this_: json!({"tick_kind":kind,"checks":["open_ghosts","receipt_candidates","runtimes","evidence","hooks"]}),
        when: Some(Utc::now()),
        confirmed_by: json!({"runtime_id":"logline-lab-cli"}),
        if_ok: json!({"project":"audit.v_daily_lab_state"}),
        if_doubt: json!({"open_ghost":"clock-check-inconclusive"}),
        if_not: json!({"reject":"invalid_clock_tick"}),
        status: "observed".into(),
        runtime_envelope: json!({"runtime_id":"logline-lab-cli"}),
        previous_act_refs: vec![],
        evidence_state: Some("observed".into()),
        promotion_state: Some("candidate".into()),
    };
    let id = db::insert_act(&client, &act).await?;
    println!("{}", json!({"tick_act": id}));
    Ok(())
}

pub fn dispatch_prepare(process: &str) -> Result<()> {
    println!(
        "{}",
        serde_yaml::to_string(&packets::dispatch_packet(process))?
    );
    Ok(())
}

pub fn workorder_prepare(path: &Path) -> Result<()> {
    println!(
        "{}",
        serde_yaml::to_string(&packets::workorder_from_dispatch_file(path)?)?
    );
    Ok(())
}
