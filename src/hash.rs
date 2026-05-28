use serde_json::{Map, Value};
use sha2::{Digest, Sha256};

pub fn hash_value(value: &Value) -> String {
    let canonical = canonicalize_value(value);
    let bytes = serde_json::to_vec(&canonical).expect("json serialization");
    hash_bytes(&bytes)
}

pub fn hash_bytes(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("sha256:{}", hex::encode(hasher.finalize()))
}

pub fn canonicalize_value(value: &Value) -> Value {
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
