pub mod cli;
pub mod commands;
pub mod db;
pub mod hash;
pub mod hooks;
pub mod io;
pub mod labd;
pub mod model;
pub mod packets;
pub mod schema;

#[cfg(test)]
mod tests {
    use crate::{hash::hash_value, model::LabManifest, schema::validate_json_schema};
    use serde_json::json;
    use std::path::Path;

    #[test]
    fn validates_spine_manifest() {
        let manifest = LabManifest {
            lab_id: "lab".into(),
            profile: "p".into(),
            semantic_write_spine: "ops.logline_acts".into(),
            projection_surface: "supabase".into(),
            canon_refs: vec![],
            hooks: vec![],
            benches: vec![],
        };
        assert!(manifest.ensure_spine().is_ok());
    }

    #[test]
    fn receipt_requires_evidence_refs() {
        let invalid = json!({
            "who":"r",
            "did":"prepare_receipt_candidate",
            "this":{},
            "confirmed_by":{},
            "if_ok":{},
            "if_doubt":{},
            "if_not":{},
            "status":"candidate"
        });
        assert!(
            validate_json_schema(Path::new("schemas/receipt-candidate.schema.json"), &invalid)
                .is_err()
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
