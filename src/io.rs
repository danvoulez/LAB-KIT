use crate::{
    model::{LabManifest, LogLineAct},
    schema::validate_json_schema,
};
use anyhow::{Context, Result};
use serde_json::Value;
use std::{fs, path::Path};

pub fn read_manifest(path: &Path) -> Result<LabManifest> {
    let value =
        read_json_or_yaml(path).with_context(|| format!("read manifest {}", path.display()))?;
    validate_json_schema(Path::new("schemas/lab-manifest.schema.json"), &value)
        .context("manifest schema validation failed")?;
    Ok(serde_json::from_value(value)?)
}

pub fn read_act(path: &Path) -> Result<LogLineAct> {
    let value = read_json_or_yaml(path)?;
    validate_json_schema(Path::new("schemas/logline-act.schema.json"), &value)
        .context("LogLine Act schema validation failed")?;
    Ok(serde_json::from_value(value)?)
}

pub fn read_json_or_yaml(path: &Path) -> Result<Value> {
    let text = fs::read_to_string(path)?;
    if path.extension().and_then(|ext| ext.to_str()) == Some("json") {
        Ok(serde_json::from_str(&text)?)
    } else {
        Ok(serde_yaml::from_str(&text)?)
    }
}
