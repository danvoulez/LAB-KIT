use crate::io::read_json_or_yaml;
use anyhow::Result;
use serde_json::{json, Value};
use std::path::Path;
use uuid::Uuid;

pub fn dispatch_packet(process: &str) -> Value {
    json!({
        "dispatch_version":"logline.dispatch.v0",
        "packet_id":format!("dispatch_{}", Uuid::new_v4()),
        "process_id":process,
        "authority_level":"read_only",
        "objective":"prepared by logline-lab CLI",
        "input_sources":[],
        "expected_evidence":[],
        "required_human_check":true,
        "hermes":{"workorder_allowed":true,"suggested_mode":"dry_run"},
        "allowed_actions":[],
        "forbidden_actions":["protected_apply_without_window"],
        "ghosts":[]
    })
}

pub fn workorder_from_dispatch_file(path: &Path) -> Result<Value> {
    let dispatch = read_json_or_yaml(path)?;
    Ok(workorder_from_dispatch(&dispatch))
}

pub fn workorder_from_dispatch(dispatch: &Value) -> Value {
    let mode = dispatch
        .pointer("/hermes/suggested_mode")
        .and_then(Value::as_str)
        .unwrap_or("dry_run");
    json!({
        "workorder_version":"hermes.workorder.v0",
        "workorder_id":format!("wo_{}", Uuid::new_v4()),
        "source_dispatch_packet":dispatch.get("packet_id"),
        "authority_decision_ref":null,
        "execution_window_ref":null,
        "mode":mode,
        "target":{"lab_id":null,"runtime_id":null,"working_directory":null},
        "allowed_actions":[],
        "forbidden_actions":["secrets_in_logs","protected_without_window"],
        "commands":[],
        "expected_outputs":[],
        "evidence_required":[],
        "secret_policy":{"secret_values_allowed_in_logs":false,"redact_before_store":true},
        "projection_targets":[]
    })
}
