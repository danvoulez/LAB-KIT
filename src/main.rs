use anyhow::{anyhow, bail, Context, Result};
use axum::{
    extract::State,
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use chrono::{DateTime, Utc};
use clap::{Args, Parser, Subcommand, ValueEnum};
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};
use sha2::{Digest, Sha256};
use std::{
    fs,
    net::SocketAddr,
    path::{Path, PathBuf},
    sync::Arc,
};
use tokio_postgres::{Client, NoTls, Row};
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use uuid::Uuid;

const INIT_ACT_DID: &str = "instantiate_logline_lab";

#[derive(Parser, Debug)]
#[command(
    name = "logline-lab",
    version,
    about = "Rust-native LogLine Lab Kit CLI/labd"
)]
struct Cli {
    #[arg(long, env = "SUPABASE_DB_URL", global = true)]
    database_url: Option<String>,
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    Init(InitArgs),
    Doctor(DoctorArgs),
    Status,
    Canon {
        #[command(subcommand)]
        command: CanonCommand,
    },
    Act {
        #[command(subcommand)]
        command: ActCommand,
    },
    Ghost {
        #[command(subcommand)]
        command: GhostCommand,
    },
    Evidence {
        #[command(subcommand)]
        command: EvidenceCommand,
    },
    Receipt {
        #[command(subcommand)]
        command: ReceiptCommand,
    },
    Report {
        #[command(subcommand)]
        command: ReportCommand,
    },
    Projector {
        #[command(subcommand)]
        command: ProjectorCommand,
    },
    Hook {
        #[command(subcommand)]
        command: HookCommand,
    },
    Clock {
        #[command(subcommand)]
        command: ClockCommand,
    },
    Dispatch {
        #[command(subcommand)]
        command: DispatchCommand,
    },
    Workorder {
        #[command(subcommand)]
        command: WorkorderCommand,
    },
    Labd(LabdArgs),
}

#[derive(Args, Debug)]
struct InitArgs {
    #[arg(long, default_value = "manifests/lab.manifest.example.yaml")]
    manifest: PathBuf,
    #[arg(long)]
    no_seed: bool,
}

#[derive(Args, Debug)]
struct DoctorArgs {
    #[arg(long)]
    json: bool,
}

#[derive(Subcommand, Debug)]
enum CanonCommand {
    Status,
}

#[derive(Subcommand, Debug)]
enum ActCommand {
    Emit(FileArg),
    Get { id: Uuid },
}

#[derive(Subcommand, Debug)]
enum GhostCommand {
    List,
    Open(GhostOpenArgs),
}

#[derive(Args, Debug)]
struct GhostOpenArgs {
    #[arg(long)]
    key: String,
    #[arg(long)]
    what_missing: String,
    #[arg(long, default_value = "operator_observation")]
    source: String,
    #[arg(long, default_value = "operator")]
    who: String,
}

#[derive(Subcommand, Debug)]
enum EvidenceCommand {
    Add(FileArg),
    List,
}

#[derive(Subcommand, Debug)]
enum ReceiptCommand {
    Prepare(FileArg),
}

#[derive(Subcommand, Debug)]
enum ReportCommand {
    Generate { kind: ReportKind },
}

#[derive(Clone, Debug, ValueEnum)]
enum ReportKind {
    DailyExpedition,
    WeeklyReview,
}

#[derive(Subcommand, Debug)]
enum ProjectorCommand {
    Run {
        #[arg(long, default_value = "all")]
        name: String,
    },
}

#[derive(Subcommand, Debug)]
enum HookCommand {
    Run {
        #[arg(long)]
        hook: String,
        #[arg(long)]
        payload: Option<PathBuf>,
    },
}

#[derive(Subcommand, Debug)]
enum ClockCommand {
    Tick {
        #[arg(long, default_value = "daily")]
        kind: String,
    },
}

#[derive(Subcommand, Debug)]
enum DispatchCommand {
    Prepare {
        #[arg(long)]
        process: String,
    },
}

#[derive(Subcommand, Debug)]
enum WorkorderCommand {
    Prepare {
        #[arg(long)]
        dispatch_file: PathBuf,
    },
}

#[derive(Args, Debug)]
struct FileArg {
    #[arg(long)]
    file: PathBuf,
}

#[derive(Args, Debug)]
struct LabdArgs {
    #[arg(long, env = "LABD_BIND", default_value = "127.0.0.1:8787")]
    bind: SocketAddr,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct LabManifest {
    lab_id: String,
    profile: String,
    semantic_write_spine: String,
    projection_surface: String,
    #[serde(default)]
    canon_refs: Vec<String>,
    #[serde(default)]
    hooks: Vec<String>,
    #[serde(default)]
    benches: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct LogLineAct {
    who: String,
    did: String,
    #[serde(rename = "this")]
    this_: Value,
    #[serde(default)]
    when: Option<DateTime<Utc>>,
    #[serde(default)]
    confirmed_by: Value,
    #[serde(default)]
    if_ok: Value,
    #[serde(default)]
    if_doubt: Value,
    #[serde(default)]
    if_not: Value,
    status: String,
    #[serde(default)]
    runtime_envelope: Value,
    #[serde(default)]
    previous_act_refs: Vec<String>,
    #[serde(default)]
    evidence_state: Option<String>,
    #[serde(default)]
    promotion_state: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    let cli = Cli::parse();
    match cli.command {
        Command::Init(args) => init(cli.database_url.as_deref(), args).await,
        Command::Doctor(args) => doctor(cli.database_url.as_deref(), args).await,
        Command::Status => status(cli.database_url.as_deref()).await,
        Command::Canon {
            command: CanonCommand::Status,
        } => canon_status(),
        Command::Act { command } => match command {
            ActCommand::Emit(args) => emit_from_file(cli.database_url.as_deref(), &args.file).await,
            ActCommand::Get { id } => act_get(cli.database_url.as_deref(), id).await,
        },
        Command::Ghost { command } => match command {
            GhostCommand::List => {
                list_projection(cli.database_url.as_deref(), "audit.v_open_ghosts").await
            }
            GhostCommand::Open(args) => ghost_open(cli.database_url.as_deref(), args).await,
        },
        Command::Evidence { command } => match command {
            EvidenceCommand::Add(args) => {
                emit_from_file(cli.database_url.as_deref(), &args.file).await
            }
            EvidenceCommand::List => {
                list_projection(cli.database_url.as_deref(), "evidence.records").await
            }
        },
        Command::Receipt {
            command: ReceiptCommand::Prepare(args),
        } => receipt_prepare(cli.database_url.as_deref(), &args.file).await,
        Command::Report {
            command: ReportCommand::Generate { kind },
        } => report_generate(cli.database_url.as_deref(), kind).await,
        Command::Projector {
            command: ProjectorCommand::Run { name },
        } => projector_run(cli.database_url.as_deref(), &name).await,
        Command::Hook {
            command: HookCommand::Run { hook, payload },
        } => hook_run(&hook, payload.as_deref()),
        Command::Clock {
            command: ClockCommand::Tick { kind },
        } => clock_tick(cli.database_url.as_deref(), &kind).await,
        Command::Dispatch {
            command: DispatchCommand::Prepare { process },
        } => dispatch_prepare(&process),
        Command::Workorder {
            command: WorkorderCommand::Prepare { dispatch_file },
        } => workorder_prepare(&dispatch_file),
        Command::Labd(args) => labd(cli.database_url, args).await,
    }
}

async fn init(database_url: Option<&str>, args: InitArgs) -> Result<()> {
    let manifest = read_manifest(&args.manifest)?;
    ensure_spine(&manifest)?;
    println!("manifest ok: {} ({})", manifest.lab_id, manifest.profile);
    let Some(url) = database_url else {
        println!("no SUPABASE_DB_URL supplied; wrote no semantic state. Run with --database-url or SUPABASE_DB_URL to migrate and seed Supabase.");
        return Ok(());
    };
    let client = connect(url).await?;
    run_migrations(&client).await?;
    if !args.no_seed {
        let act = initial_lab_act(&manifest);
        insert_act(&client, &act).await?;
        let ghost = initial_ghost_act(&manifest.lab_id);
        insert_act(&client, &ghost).await?;
    }
    println!("init complete: Supabase act spine ready");
    Ok(())
}

async fn doctor(database_url: Option<&str>, args: DoctorArgs) -> Result<()> {
    let mut checks = Vec::new();
    checks.push(json!({"check":"manifest examples", "ok": Path::new("manifests/lab.manifest.example.yaml").exists()}));
    checks.push(json!({"check":"migrations", "ok": Path::new("supabase/migrations/0001_ops_logline_acts.sql").exists()}));
    checks.push(
        json!({"check":"schemas", "ok": Path::new("schemas/logline-act.schema.json").exists()}),
    );
    if let Some(url) = database_url {
        match connect(url).await {
            Ok(client) => {
                checks.push(json!({"check":"Supabase/Postgres connection", "ok": true}));
                for table in ["ops.logline_acts", "audit.v_mobile_today", "lab_observability.ghosts", "evidence.records", "receipts.candidates"] {
                    checks.push(json!({"check": format!("relation {table}"), "ok": relation_exists(&client, table).await.unwrap_or(false)}));
                }
            }
            Err(err) => checks.push(json!({"check":"Supabase/Postgres connection", "ok": false, "error": err.to_string()})),
        }
    } else {
        checks.push(json!({"check":"SUPABASE_DB_URL", "ok": false, "warning":"semantic commands need Supabase/Postgres"}));
    }
    if args.json {
        println!("{}", serde_json::to_string_pretty(&checks)?);
    } else {
        for c in checks {
            println!("{}", serde_json::to_string(&c)?);
        }
    }
    Ok(())
}

async fn status(database_url: Option<&str>) -> Result<()> {
    let client = connect_required(database_url).await?;
    let counts = client
        .query_one("select count(*)::bigint from ops.logline_acts", &[])
        .await?
        .get::<_, i64>(0);
    let ghosts = count_relation(&client, "lab_observability.ghosts")
        .await
        .unwrap_or(0);
    let evidence = count_relation(&client, "evidence.records")
        .await
        .unwrap_or(0);
    let receipts = count_relation(&client, "receipts.candidates")
        .await
        .unwrap_or(0);
    println!(
        "{}",
        serde_json::to_string_pretty(
            &json!({"act_spine":"ops.logline_acts","acts":counts,"open_ghosts":ghosts,"evidence_records":evidence,"receipt_candidates":receipts})
        )?
    );
    Ok(())
}

fn canon_status() -> Result<()> {
    println!(
        "{}",
        serde_json::to_string_pretty(
            &json!({"foundation_refs":"canon/foundation.refs.yaml","conformance_refs":"canon/conformance.refs.yaml","profile":"profiles/logline-lab.practice.v0.yaml","status":"loaded-by-reference"})
        )?
    );
    Ok(())
}

async fn emit_from_file(database_url: Option<&str>, path: &Path) -> Result<()> {
    let client = connect_required(database_url).await?;
    let act = read_act(path)?;
    validate_act(&act)?;
    let id = insert_act(&client, &act).await?;
    println!(
        "{}",
        serde_json::to_string_pretty(&json!({"inserted":id,"spine":"ops.logline_acts"}))?
    );
    Ok(())
}

async fn receipt_prepare(database_url: Option<&str>, path: &Path) -> Result<()> {
    let raw = read_json_or_yaml(path)?;
    validate_json_schema(Path::new("schemas/receipt-candidate.schema.json"), &raw)
        .context("receipt candidate schema validation failed")?;
    let act: LogLineAct = serde_json::from_value(raw)?;
    let refs = act
        .this_
        .get("evidence_refs")
        .and_then(Value::as_array)
        .map(Vec::len)
        .unwrap_or(0);
    if refs == 0 {
        bail!("receipt candidate blocked: evidence_refs are required");
    }
    let client = connect_required(database_url).await?;
    let id = insert_act(&client, &act).await?;
    println!("{}", json!({"receipt_candidate": id, "status":"candidate"}));
    Ok(())
}

async fn act_get(database_url: Option<&str>, id: Uuid) -> Result<()> {
    let client = connect_required(database_url).await?;
    let row = client.query_one("select id, who, did, this, \"when\", confirmed_by, if_ok, if_doubt, if_not, status, runtime_envelope, tuple_hash, content_hash, previous_act_refs, evidence_state, promotion_state, created_at from ops.logline_acts where id=$1", &[&id]).await?;
    println!("{}", serde_json::to_string_pretty(&row_to_json(&row))?);
    Ok(())
}

async fn ghost_open(database_url: Option<&str>, args: GhostOpenArgs) -> Result<()> {
    let act = LogLineAct {
        who: args.who,
        did: "open_ghost".into(),
        this_: json!({"ghost_key":args.key,"what_missing":args.what_missing}),
        when: Some(Utc::now()),
        confirmed_by: json!({"source":args.source}),
        if_ok: json!({"project":"lab_observability.ghosts"}),
        if_doubt: json!({"carry":true}),
        if_not: json!({"reject":"ghost_without_missing_condition"}),
        status: "open".into(),
        runtime_envelope: json!({"runtime_id":"logline-lab-cli"}),
        previous_act_refs: vec![],
        evidence_state: Some("declared".into()),
        promotion_state: Some("candidate".into()),
    };
    let client = connect_required(database_url).await?;
    let id = insert_act(&client, &act).await?;
    println!("{}", json!({"ghost_act": id, "spine":"ops.logline_acts"}));
    Ok(())
}

async fn list_projection(database_url: Option<&str>, relation: &str) -> Result<()> {
    let client = connect_required(database_url).await?;
    let sql = format!("select row_to_json(t) from (select * from {relation} limit 100) t");
    let rows = client.query(&sql, &[]).await?;
    let values: Vec<Value> = rows.into_iter().map(|r| r.get(0)).collect();
    println!("{}", serde_json::to_string_pretty(&values)?);
    Ok(())
}

async fn report_generate(database_url: Option<&str>, kind: ReportKind) -> Result<()> {
    let client = connect_required(database_url).await?;
    let acts = count_relation(&client, "ops.logline_acts")
        .await
        .unwrap_or(0);
    let ghosts = count_relation(&client, "lab_observability.ghosts")
        .await
        .unwrap_or(0);
    let title = match kind {
        ReportKind::DailyExpedition => "Daily Lab Expedition",
        ReportKind::WeeklyReview => "Weekly Lab Review",
    };
    println!("# {title}\n\n- generated_at: {}\n- act_spine: ops.logline_acts\n- acts: {acts}\n- open_ghosts: {ghosts}\n\nThis is an operational report, not a receipt.", Utc::now().to_rfc3339());
    Ok(())
}

async fn projector_run(database_url: Option<&str>, name: &str) -> Result<()> {
    let client = connect_required(database_url).await?;
    run_projector_refresh(&client).await?;
    println!("projectors refreshed: {name}");
    Ok(())
}

fn hook_run(hook: &str, payload: Option<&Path>) -> Result<()> {
    let path = PathBuf::from(format!("hooks/default/{hook}.yaml"));
    if !path.exists() {
        bail!("hook not found: {}", path.display());
    }
    let content = fs::read_to_string(&path)?;
    let parsed: Value = serde_yaml::from_str(&content)?;
    let payload_value = if let Some(payload) = payload {
        Some(read_json_or_yaml(payload)?)
    } else {
        None
    };
    let result = evaluate_hook(&parsed, payload_value.as_ref())?;
    let blocked = result.get("status").and_then(Value::as_str) == Some("blocked");
    println!("{}", serde_json::to_string_pretty(&result)?);
    if blocked {
        bail!("mandatory hook blocked");
    }
    Ok(())
}

fn evaluate_hook(hook: &Value, payload: Option<&Value>) -> Result<Value> {
    let checks = hook
        .get("checks")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let mut results = Vec::new();
    let mut passed = true;
    for check in checks {
        let Some(check_name) = check.as_str() else {
            continue;
        };
        let ok = match check_name {
            "evidence_refs_present" => payload
                .and_then(|value| {
                    value
                        .pointer("/this/evidence_refs")
                        .or_else(|| value.get("evidence_refs"))
                })
                .and_then(Value::as_array)
                .map(|refs| !refs.is_empty())
                .unwrap_or(false),
            "claim_scope_present" => payload
                .and_then(|value| {
                    value
                        .pointer("/this/scope_closed")
                        .or_else(|| value.get("scope_closed"))
                })
                .and_then(Value::as_str)
                .map(|scope| !scope.trim().is_empty())
                .unwrap_or(false),
            "ghosts_remaining_named" => payload
                .and_then(|value| {
                    value
                        .pointer("/this/ghosts_remaining")
                        .or_else(|| value.get("ghosts_remaining"))
                })
                .is_some(),
            "claim_refs_present" => payload
                .and_then(|value| value.get("claim_refs"))
                .and_then(Value::as_array)
                .map(|refs| !refs.is_empty())
                .unwrap_or(false),
            "receipt_refs_present" => payload
                .and_then(|value| value.get("receipt_refs"))
                .and_then(Value::as_array)
                .map(|refs| !refs.is_empty())
                .unwrap_or(false),
            "verification_plan_present" => payload
                .and_then(|value| value.get("verification_plan"))
                .is_some(),
            _ => bail!("unknown hook check: {check_name}"),
        };
        if !ok {
            passed = false;
        }
        results.push(json!({"check": check_name, "ok": ok}));
    }
    Ok(json!({
        "hook": hook.get("hook_id"),
        "mandatory": hook.get("mandatory"),
        "status": if passed { "passed" } else { "blocked" },
        "checks": results,
        "next": if passed { hook.get("on_pass") } else { hook.get("on_fail") }
    }))
}

async fn clock_tick(database_url: Option<&str>, kind: &str) -> Result<()> {
    let client = connect_required(database_url).await?;
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
    let id = insert_act(&client, &act).await?;
    println!("{}", json!({"tick_act": id}));
    Ok(())
}

fn dispatch_prepare(process: &str) -> Result<()> {
    let packet = json!({"dispatch_version":"logline.dispatch.v0","packet_id":format!("dispatch_{}", Uuid::new_v4()),"process_id":process,"authority_level":"read_only","objective":"prepared by logline-lab CLI","input_sources":[],"expected_evidence":[],"required_human_check":true,"hermes":{"workorder_allowed":true,"suggested_mode":"dry_run"},"allowed_actions":[],"forbidden_actions":["protected_apply_without_window"],"ghosts":[]});
    println!("{}", serde_yaml::to_string(&packet)?);
    Ok(())
}

fn workorder_prepare(path: &Path) -> Result<()> {
    let dispatch = read_json_or_yaml(path)?;
    let mode = dispatch
        .pointer("/hermes/suggested_mode")
        .and_then(Value::as_str)
        .unwrap_or("dry_run");
    let workorder = json!({"workorder_version":"hermes.workorder.v0","workorder_id":format!("wo_{}", Uuid::new_v4()),"source_dispatch_packet":dispatch.get("packet_id"),"authority_decision_ref":null,"execution_window_ref":null,"mode":mode,"target":{"lab_id":null,"runtime_id":null,"working_directory":null},"allowed_actions":[],"forbidden_actions":["secrets_in_logs","protected_without_window"],"commands":[],"expected_outputs":[],"evidence_required":[],"secret_policy":{"secret_values_allowed_in_logs":false,"redact_before_store":true},"projection_targets":[]});
    println!("{}", serde_yaml::to_string(&workorder)?);
    Ok(())
}

async fn labd(database_url: Option<String>, args: LabdArgs) -> Result<()> {
    let state = Arc::new(AppState { database_url });
    let app = Router::new()
        .route("/status", get(http_status))
        .route("/daily-state", get(http_status))
        .route("/ghosts", get(http_ghosts).post(http_post_act))
        .route("/evidence", get(http_evidence).post(http_post_act))
        .route("/receipts", get(http_receipts))
        .route("/receipts/prepare", post(http_post_act))
        .route("/canon", get(http_canon))
        .route("/runtimes", get(http_runtimes))
        .route("/acts", post(http_post_act))
        .route("/projectors/run", post(http_projectors))
        .route("/clock/tick", post(http_clock_tick))
        .route("/dispatch/prepare", post(http_dispatch))
        .route("/workorders/prepare", post(http_workorder))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(state);
    println!("labd listening on http://{}", args.bind);
    let listener = tokio::net::TcpListener::bind(args.bind).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

struct AppState {
    database_url: Option<String>,
}

async fn http_status(State(state): State<Arc<AppState>>) -> (StatusCode, Json<Value>) {
    db_summary(&state).await
}
async fn http_ghosts(State(state): State<Arc<AppState>>) -> (StatusCode, Json<Value>) {
    db_list(&state, "lab_observability.ghosts").await
}
async fn http_evidence(State(state): State<Arc<AppState>>) -> (StatusCode, Json<Value>) {
    db_list(&state, "evidence.records").await
}
async fn http_receipts(State(state): State<Arc<AppState>>) -> (StatusCode, Json<Value>) {
    db_list(&state, "receipts.candidates").await
}
async fn http_canon() -> Json<Value> {
    Json(json!({"status":"loaded-by-reference"}))
}
async fn http_runtimes(State(state): State<Arc<AppState>>) -> (StatusCode, Json<Value>) {
    db_list(&state, "registry.runtimes").await
}
async fn http_post_act(
    State(state): State<Arc<AppState>>,
    Json(act): Json<LogLineAct>,
) -> (StatusCode, Json<Value>) {
    match async {
        validate_act(&act)?;
        let client = connect_required(state.database_url.as_deref()).await?;
        let id = insert_act(&client, &act).await?;
        Ok::<_, anyhow::Error>(id)
    }
    .await
    {
        Ok(id) => (
            StatusCode::CREATED,
            Json(json!({"id": id, "spine":"ops.logline_acts"})),
        ),
        Err(err) => (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": err.to_string()})),
        ),
    }
}
async fn http_projectors(State(state): State<Arc<AppState>>) -> (StatusCode, Json<Value>) {
    match async {
        let c = connect_required(state.database_url.as_deref()).await?;
        run_projector_refresh(&c).await
    }
    .await
    {
        Ok(_) => (StatusCode::OK, Json(json!({"projectors":"refreshed"}))),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(json!({"error":e.to_string()})),
        ),
    }
}
async fn http_clock_tick(State(state): State<Arc<AppState>>) -> (StatusCode, Json<Value>) {
    match clock_tick(state.database_url.as_deref(), "api").await {
        Ok(_) => (StatusCode::CREATED, Json(json!({"tick":"emitted"}))),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(json!({"error":e.to_string()})),
        ),
    }
}
async fn http_dispatch(Json(payload): Json<Value>) -> Json<Value> {
    Json(
        json!({"dispatch_version":"logline.dispatch.v0","packet_id":format!("dispatch_{}", Uuid::new_v4()),"process_id":payload.get("process_id").cloned().unwrap_or(json!("unknown")),"authority_level":"read_only"}),
    )
}
async fn http_workorder(Json(payload): Json<Value>) -> Json<Value> {
    Json(
        json!({"workorder_version":"hermes.workorder.v0","workorder_id":format!("wo_{}", Uuid::new_v4()),"source_dispatch_packet":payload.get("packet_id"),"mode":"dry_run","secret_policy":{"secret_values_allowed_in_logs":false,"redact_before_store":true}}),
    )
}

async fn db_summary(state: &AppState) -> (StatusCode, Json<Value>) {
    match async { let c=connect_required(state.database_url.as_deref()).await?; Ok::<_, anyhow::Error>(json!({"acts":count_relation(&c,"ops.logline_acts").await.unwrap_or(0),"ghosts":count_relation(&c,"lab_observability.ghosts").await.unwrap_or(0),"spine":"ops.logline_acts"})) }.await { Ok(v)=>(StatusCode::OK,Json(v)), Err(e)=>(StatusCode::SERVICE_UNAVAILABLE,Json(json!({"error":e.to_string()}))) }
}
async fn db_list(state: &AppState, relation: &str) -> (StatusCode, Json<Value>) {
    match async { let c=connect_required(state.database_url.as_deref()).await?; let sql=format!("select coalesce(jsonb_agg(row_to_json(t)), '[]'::jsonb) from (select * from {relation} limit 100) t"); let row=c.query_one(&sql,&[]).await?; Ok::<_, anyhow::Error>(row.get::<_, Value>(0)) }.await { Ok(v)=>(StatusCode::OK,Json(v)), Err(e)=>(StatusCode::SERVICE_UNAVAILABLE,Json(json!({"error":e.to_string()}))) }
}

fn read_manifest(path: &Path) -> Result<LabManifest> {
    let value =
        read_json_or_yaml(path).with_context(|| format!("read manifest {}", path.display()))?;
    validate_json_schema(Path::new("schemas/lab-manifest.schema.json"), &value)
        .context("manifest schema validation failed")?;
    Ok(serde_json::from_value(value)?)
}
fn ensure_spine(m: &LabManifest) -> Result<()> {
    if m.semantic_write_spine != "ops.logline_acts" {
        bail!("manifest must declare semantic_write_spine: ops.logline_acts");
    }
    Ok(())
}
fn read_act(path: &Path) -> Result<LogLineAct> {
    let value = read_json_or_yaml(path)?;
    validate_json_schema(Path::new("schemas/logline-act.schema.json"), &value)
        .context("LogLine Act schema validation failed")?;
    Ok(serde_json::from_value(value)?)
}
fn read_json_or_yaml(path: &Path) -> Result<Value> {
    let s = fs::read_to_string(path)?;
    if path.extension().and_then(|s| s.to_str()) == Some("json") {
        Ok(serde_json::from_str(&s)?)
    } else {
        Ok(serde_yaml::from_str(&s)?)
    }
}
fn validate_act(act: &LogLineAct) -> Result<()> {
    if act.who.trim().is_empty() || act.did.trim().is_empty() || act.status.trim().is_empty() {
        bail!("who, did, and status are required");
    }
    if act.this_.is_null() {
        bail!("this must be present");
    }
    Ok(())
}

fn initial_lab_act(m: &LabManifest) -> LogLineAct {
    LogLineAct {
        who: "operator".into(),
        did: INIT_ACT_DID.into(),
        this_: json!({"lab_id":m.lab_id,"kit":"logline-lab-kit","profile":m.profile,"semantic_write_spine":m.semantic_write_spine,"projection_surface":m.projection_surface}),
        when: Some(Utc::now()),
        confirmed_by: json!({"tool":"logline-lab init"}),
        if_ok: json!({"project":["registry.entities","audit.v_mobile_today","lab_observability.current_state"]}),
        if_doubt: json!({"open_ghost":"lab-instantiation-incomplete"}),
        if_not: json!({"reject":"invalid_lab_instantiation"}),
        status: "declared".into(),
        runtime_envelope: json!({"runtime_id":"logline-lab-cli"}),
        previous_act_refs: vec![],
        evidence_state: Some("declared".into()),
        promotion_state: Some("candidate".into()),
    }
}
fn initial_ghost_act(lab_id: &str) -> LogLineAct {
    LogLineAct {
        who: lab_id.into(),
        did: "open_ghost".into(),
        this_: json!({"ghost_key":"initial-lab-review","what_missing":"Human review of first lab installation remains open.","next_act":"run logline-lab doctor and prepare receipt candidate with evidence refs"}),
        when: Some(Utc::now()),
        confirmed_by: json!({"source":"init_seed"}),
        if_ok: json!({"project":"lab_observability.ghosts"}),
        if_doubt: json!({"carry":true}),
        if_not: json!({"reject":"ghost_without_missing_condition"}),
        status: "open".into(),
        runtime_envelope: json!({"runtime_id":"logline-lab-cli"}),
        previous_act_refs: vec![],
        evidence_state: Some("declared".into()),
        promotion_state: Some("candidate".into()),
    }
}

async fn connect_required(database_url: Option<&str>) -> Result<Client> {
    let url = database_url.ok_or_else(|| {
        anyhow!("SUPABASE_DB_URL or --database-url is required for semantic spine operations")
    })?;
    connect(url).await
}
async fn connect(url: &str) -> Result<Client> {
    let (client, connection) = tokio_postgres::connect(url, NoTls).await?;
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("postgres connection error: {e}");
        }
    });
    Ok(client)
}
async fn run_migrations(client: &Client) -> Result<()> {
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

    let mut migrations = Vec::new();
    for entry in fs::read_dir("supabase/migrations")? {
        let path = entry?.path();
        if path.extension().and_then(|s| s.to_str()) == Some("sql") {
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
async fn run_projector_refresh(client: &Client) -> Result<()> {
    client
        .batch_execute(include_str!(
            "../supabase/migrations/0009_functions_projectors.sql"
        ))
        .await?;
    Ok(())
}
async fn insert_act(client: &Client, act: &LogLineAct) -> Result<Uuid> {
    validate_act(act)?;
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
fn hash_value(value: &Value) -> String {
    let canonical = canonicalize_value(value);
    let bytes = serde_json::to_vec(&canonical).expect("json serialization");
    hash_bytes(&bytes)
}

fn hash_bytes(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("sha256:{}", hex::encode(hasher.finalize()))
}

fn canonicalize_value(value: &Value) -> Value {
    match value {
        Value::Array(items) => Value::Array(items.iter().map(canonicalize_value).collect()),
        Value::Object(object) => {
            let mut keys: Vec<_> = object.keys().collect();
            keys.sort();
            let mut sorted = Map::new();
            for key in keys {
                sorted.insert(key.clone(), canonicalize_value(&object[key]));
            }
            Value::Object(sorted)
        }
        other => other.clone(),
    }
}

fn validate_json_schema(schema_path: &Path, instance: &Value) -> Result<()> {
    let schema_text = fs::read_to_string(schema_path)
        .with_context(|| format!("read schema {}", schema_path.display()))?;
    let schema: Value = serde_json::from_str(&schema_text)
        .with_context(|| format!("parse schema {}", schema_path.display()))?;
    let compiled = jsonschema::JSONSchema::compile(&schema)
        .map_err(|err| anyhow!("compile schema {}: {err}", schema_path.display()))?;
    if let Err(errors) = compiled.validate(instance) {
        let messages: Vec<String> = errors.map(|err| err.to_string()).collect();
        bail!("{}", messages.join("; "));
    }
    Ok(())
}
async fn relation_exists(client: &Client, relation: &str) -> Result<bool> {
    let parts: Vec<_> = relation.split('.').collect();
    let row = client
        .query_one(
            "select to_regclass($1) is not null",
            &[&format!("{}.{}", parts[0], parts[1])],
        )
        .await?;
    Ok(row.get(0))
}
async fn count_relation(client: &Client, relation: &str) -> Result<i64> {
    let row = client
        .query_one(&format!("select count(*)::bigint from {relation}"), &[])
        .await?;
    Ok(row.get(0))
}
fn row_to_json(row: &Row) -> Value {
    json!({"id": row.get::<_, Uuid>(0), "who": row.get::<_, String>(1), "did": row.get::<_, String>(2), "this": row.get::<_, Value>(3), "when": row.get::<_, DateTime<Utc>>(4), "confirmed_by": row.get::<_, Value>(5), "if_ok": row.get::<_, Value>(6), "if_doubt": row.get::<_, Value>(7), "if_not": row.get::<_, Value>(8), "status": row.get::<_, String>(9), "runtime_envelope": row.get::<_, Value>(10), "tuple_hash": row.get::<_, Option<String>>(11), "content_hash": row.get::<_, Option<String>>(12), "previous_act_refs": row.get::<_, Vec<String>>(13), "evidence_state": row.get::<_, String>(14), "promotion_state": row.get::<_, String>(15), "created_at": row.get::<_, DateTime<Utc>>(16)})
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn validates_spine_manifest() {
        let m = LabManifest {
            lab_id: "lab".into(),
            profile: "p".into(),
            semantic_write_spine: "ops.logline_acts".into(),
            projection_surface: "supabase".into(),
            canon_refs: vec![],
            hooks: vec![],
            benches: vec![],
        };
        assert!(ensure_spine(&m).is_ok());
    }
    #[test]
    fn receipt_requires_evidence_refs() {
        let act = LogLineAct {
            who: "r".into(),
            did: "prepare_receipt_candidate".into(),
            this_: json!({}),
            when: None,
            confirmed_by: json!({}),
            if_ok: json!({}),
            if_doubt: json!({}),
            if_not: json!({}),
            status: "candidate".into(),
            runtime_envelope: json!({}),
            previous_act_refs: vec![],
            evidence_state: None,
            promotion_state: None,
        };
        assert_eq!(
            act.this_
                .get("evidence_refs")
                .and_then(Value::as_array)
                .map(Vec::len)
                .unwrap_or(0),
            0
        );
    }
    #[test]
    fn hashes_are_prefixed_sha256() {
        assert!(hash_value(&json!({"a":1})).starts_with("sha256:"));
    }

    #[test]
    fn canonical_hash_ignores_object_key_order() {
        assert_eq!(
            hash_value(&json!({"a":1,"b":2})),
            hash_value(&json!({"b":2,"a":1}))
        );
    }

    #[test]
    fn logline_act_schema_rejects_missing_slots() {
        let invalid = json!({"who":"x","did":"y"});
        assert!(
            validate_json_schema(Path::new("schemas/logline-act.schema.json"), &invalid).is_err()
        );
    }
}
