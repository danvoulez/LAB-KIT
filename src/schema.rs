use anyhow::{anyhow, bail, Context, Result};
use serde_json::Value;
use std::{fs, path::Path};

pub fn validate_json_schema(schema_path: &Path, instance: &Value) -> Result<()> {
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
