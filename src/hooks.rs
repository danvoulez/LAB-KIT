use crate::io::read_json_or_yaml;
use anyhow::{bail, Result};
use serde_json::{json, Value};
use std::{
    fs,
    path::{Path, PathBuf},
};

pub fn run_hook(hook: &str, payload: Option<&Path>) -> Result<Value> {
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
    if result.get("status").and_then(Value::as_str) == Some("blocked") {
        bail!("{}", serde_json::to_string_pretty(&result)?);
    }
    Ok(result)
}

pub fn evaluate_hook(hook: &Value, payload: Option<&Value>) -> Result<Value> {
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
