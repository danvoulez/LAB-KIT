use anyhow::{bail, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

pub const INIT_ACT_DID: &str = "instantiate_logline_lab";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LabManifest {
    pub lab_id: String,
    pub profile: String,
    pub semantic_write_spine: String,
    pub projection_surface: String,
    #[serde(default)]
    pub canon_refs: Vec<String>,
    #[serde(default)]
    pub hooks: Vec<String>,
    #[serde(default)]
    pub benches: Vec<String>,
}

impl LabManifest {
    pub fn ensure_spine(&self) -> Result<()> {
        if self.semantic_write_spine != "ops.logline_acts" {
            bail!("manifest must declare semantic_write_spine: ops.logline_acts");
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogLineAct {
    pub who: String,
    pub did: String,
    #[serde(rename = "this")]
    pub this_: Value,
    #[serde(default)]
    pub when: Option<DateTime<Utc>>,
    #[serde(default)]
    pub confirmed_by: Value,
    #[serde(default)]
    pub if_ok: Value,
    #[serde(default)]
    pub if_doubt: Value,
    #[serde(default)]
    pub if_not: Value,
    pub status: String,
    #[serde(default)]
    pub runtime_envelope: Value,
    #[serde(default)]
    pub previous_act_refs: Vec<String>,
    #[serde(default)]
    pub evidence_state: Option<String>,
    #[serde(default)]
    pub promotion_state: Option<String>,
}

impl LogLineAct {
    pub fn validate(&self) -> Result<()> {
        if self.who.trim().is_empty() || self.did.trim().is_empty() || self.status.trim().is_empty()
        {
            bail!("who, did, and status are required");
        }
        if self.this_.is_null() {
            bail!("this must be present");
        }
        Ok(())
    }
}

pub fn initial_lab_act(manifest: &LabManifest) -> LogLineAct {
    LogLineAct {
        who: "operator".into(),
        did: INIT_ACT_DID.into(),
        this_: json!({
            "lab_id": manifest.lab_id,
            "kit": "logline-lab-kit",
            "profile": manifest.profile,
            "semantic_write_spine": manifest.semantic_write_spine,
            "projection_surface": manifest.projection_surface
        }),
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

pub fn initial_ghost_act(lab_id: &str) -> LogLineAct {
    LogLineAct {
        who: lab_id.into(),
        did: "open_ghost".into(),
        this_: json!({
            "ghost_key":"initial-lab-review",
            "what_missing":"Human review of first lab installation remains open.",
            "next_act":"run logline-lab doctor and prepare receipt candidate with evidence refs"
        }),
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
