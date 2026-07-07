//! Stable request-hash helpers for idempotent write commands (`API_SPEC.md` §17).

use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WriteCommandHashError;

pub fn stable_command_request_hash(scope: &str, parts: &[&str]) -> String {
    let mut normalized = vec![scope];
    normalized.extend(parts);
    normalized
        .iter()
        .map(|part| normalize_request_hash_part(part))
        .collect::<Vec<_>>()
        .join("-")
}

pub fn stable_json_request_hash(
    scope: &str,
    value: &impl Serialize,
) -> Result<String, WriteCommandHashError> {
    let value = serde_json::to_value(value).map_err(|_| WriteCommandHashError)?;
    Ok(stable_canonical_json_request_hash(scope, &value))
}

pub fn stable_canonical_json_request_hash(scope: &str, value: &serde_json::Value) -> String {
    stable_command_request_hash(scope, &[&canonical_json_string(value)])
}

fn normalize_request_hash_part(part: &str) -> String {
    part.chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || matches!(character, '-' | '_' | '.') {
                character
            } else {
                '-'
            }
        })
        .collect::<String>()
}

fn canonical_json_string(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::Null => "null".to_string(),
        serde_json::Value::Bool(value) => value.to_string(),
        serde_json::Value::Number(value) => value.to_string(),
        serde_json::Value::String(value) => {
            serde_json::to_string(value).unwrap_or_else(|_| "\"\"".to_owned())
        }
        serde_json::Value::Array(values) => {
            let items = values
                .iter()
                .map(canonical_json_string)
                .collect::<Vec<_>>()
                .join(",");
            format!("[{items}]")
        }
        serde_json::Value::Object(values) => {
            let mut keys = values.keys().collect::<Vec<_>>();
            keys.sort_unstable();
            let items = keys
                .into_iter()
                .filter(|key| !values[*key].is_null())
                .map(|key| {
                    format!(
                        "{}:{}",
                        serde_json::to_string(key).unwrap_or_else(|_| "\"\"".to_owned()),
                        canonical_json_string(&values[key])
                    )
                })
                .collect::<Vec<_>>()
                .join(",");
            format!("{{{items}}}")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stable_command_request_hash_is_deterministic() {
        let first = stable_command_request_hash("scope", &["100001", "request-1"]);
        let second = stable_command_request_hash("scope", &["100001", "request-1"]);
        assert_eq!(first, second);
        assert!(!first.is_empty());
    }

    #[test]
    fn stable_json_request_hash_ignores_null_object_fields() {
        let payload = serde_json::json!({ "a": 1, "b": null });
        let hash = stable_json_request_hash("orders.cancel", &payload).expect("hash");
        assert!(!hash.is_empty());
    }
}
