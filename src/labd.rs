use crate::{commands, db, model::LogLineAct, packets};
use anyhow::Result;
use axum::{
    extract::State,
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde_json::{json, Value};
use std::{net::SocketAddr, sync::Arc};
use tower_http::{cors::CorsLayer, trace::TraceLayer};

pub struct AppState {
    database_url: Option<String>,
}

pub async fn serve(database_url: Option<String>, bind: SocketAddr) -> Result<()> {
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
    println!("labd listening on http://{bind}");
    let listener = tokio::net::TcpListener::bind(bind).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

async fn http_status(State(state): State<Arc<AppState>>) -> (StatusCode, Json<Value>) {
    match async {
        let client = db::connect_required(state.database_url.as_deref()).await?;
        commands::status_value(&client).await
    }
    .await
    {
        Ok(value) => (StatusCode::OK, Json(value)),
        Err(err) => (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({"error":err.to_string()})),
        ),
    }
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
        act.validate()?;
        let client = db::connect_required(state.database_url.as_deref()).await?;
        let id = db::insert_act(&client, &act).await?;
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
        let client = db::connect_required(state.database_url.as_deref()).await?;
        db::run_projector_refresh(&client).await
    }
    .await
    {
        Ok(_) => (StatusCode::OK, Json(json!({"projectors":"refreshed"}))),
        Err(err) => (
            StatusCode::BAD_REQUEST,
            Json(json!({"error":err.to_string()})),
        ),
    }
}

async fn http_clock_tick(State(state): State<Arc<AppState>>) -> (StatusCode, Json<Value>) {
    match commands::clock_tick(state.database_url.as_deref(), "api").await {
        Ok(_) => (StatusCode::CREATED, Json(json!({"tick":"emitted"}))),
        Err(err) => (
            StatusCode::BAD_REQUEST,
            Json(json!({"error":err.to_string()})),
        ),
    }
}

async fn http_dispatch(Json(payload): Json<Value>) -> Json<Value> {
    let process = payload
        .get("process_id")
        .and_then(Value::as_str)
        .unwrap_or("unknown");
    Json(packets::dispatch_packet(process))
}

async fn http_workorder(Json(payload): Json<Value>) -> Json<Value> {
    Json(packets::workorder_from_dispatch(&payload))
}

async fn db_list(state: &AppState, relation: &str) -> (StatusCode, Json<Value>) {
    match async {
        let client = db::connect_required(state.database_url.as_deref()).await?;
        let values = db::list_relation(&client, relation, 100).await?;
        Ok::<_, anyhow::Error>(Value::Array(values))
    }
    .await
    {
        Ok(value) => (StatusCode::OK, Json(value)),
        Err(err) => (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({"error":err.to_string()})),
        ),
    }
}
