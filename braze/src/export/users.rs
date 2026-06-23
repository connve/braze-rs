//! Request and response types for `POST /users/export/ids`.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Request body for `POST /users/export/ids`.
///
/// All identifier fields are optional but at least one must be set. Braze
/// allows up to 50 `external_ids` or `user_aliases` per call; for `device_id`,
/// `email_address`, or `phone` only one of any identifier is allowed.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ExportUsersByIdsRequest {
    /// External identifiers for users to export (up to 50).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub external_ids: Option<Vec<String>>,

    /// User aliases for users to export (up to 50).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_aliases: Option<Vec<UserAlias>>,

    /// Device identifier as returned by the SDK (e.g. `getDeviceId`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub device_id: Option<String>,

    /// Braze's internal unique identifier for the user.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub braze_id: Option<String>,

    /// Email address to look the user up by.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email_address: Option<String>,

    /// Phone number in E.164 format.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phone: Option<String>,

    /// Subset of fields to return on each user. Braze requires this for
    /// accounts onboarded after 2024-08-22; omitting it on older accounts
    /// returns every field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fields_to_export: Option<Vec<String>>,
}

/// User alias used to look up profiles when external IDs are not available.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserAlias {
    /// Alias value.
    pub alias_name: String,
    /// Namespace the alias belongs to.
    pub alias_label: String,
}

/// Successful response from `POST /users/export/ids`.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ExportUsersByIdsResponse {
    /// Status string returned by Braze (typically `"success"`).
    #[serde(default)]
    pub message: String,

    /// User profiles successfully resolved from the supplied identifiers.
    #[serde(default)]
    pub users: Vec<ExportedUser>,

    /// Identifiers Braze could not match to a user.
    #[serde(default)]
    pub invalid_user_ids: Vec<String>,
}

/// A single exported user profile.
///
/// Only the documented fields are typed; everything else lands in `extra` so
/// the SDK does not drop data when Braze adds new fields.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ExportedUser {
    /// When the user profile was first created.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_at: Option<DateTime<Utc>>,

    /// External identifier for this user.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub external_id: Option<String>,

    /// Braze's internal identifier for this user.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub braze_id: Option<String>,

    /// First name on the profile.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub first_name: Option<String>,

    /// Last name on the profile.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_name: Option<String>,

    /// Email address on the profile.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,

    /// Phone number on the profile (E.164).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub phone: Option<String>,

    /// ISO 3166-1 alpha-2 country code.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub country: Option<String>,

    /// Free-form custom attribute map.
    #[serde(default)]
    pub custom_attributes: Value,

    /// Custom events logged for this user in the last 90 days.
    #[serde(default)]
    pub custom_events: Vec<Value>,

    /// Purchases logged for this user in the last 90 days.
    #[serde(default)]
    pub purchases: Vec<Value>,

    /// Devices Braze has seen for this user.
    #[serde(default)]
    pub devices: Vec<Value>,

    /// Push tokens registered for this user.
    #[serde(default)]
    pub push_tokens: Vec<Value>,

    /// Apps this user has used.
    #[serde(default)]
    pub apps: Vec<Value>,

    /// Campaigns this user received.
    #[serde(default)]
    pub campaigns_received: Vec<Value>,

    /// Canvases this user received.
    #[serde(default)]
    pub canvases_received: Vec<Value>,

    /// Any extra fields Braze returned that this SDK does not yet type.
    #[serde(flatten)]
    pub extra: std::collections::BTreeMap<String, Value>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn request_skips_unset_identifiers() {
        let req = ExportUsersByIdsRequest {
            external_ids: Some(vec!["u1".into()]),
            fields_to_export: Some(vec!["email".into()]),
            ..Default::default()
        };
        let json = serde_json::to_value(&req).unwrap();
        let obj = json.as_object().unwrap();
        assert!(obj.contains_key("external_ids"));
        assert!(obj.contains_key("fields_to_export"));
        assert!(!obj.contains_key("device_id"));
        assert!(!obj.contains_key("user_aliases"));
    }

    #[test]
    fn response_tolerates_missing_fields() {
        let raw = r#"{"message":"success"}"#;
        let parsed: ExportUsersByIdsResponse = serde_json::from_str(raw).unwrap();
        assert_eq!(parsed.message, "success");
        assert!(parsed.users.is_empty());
        assert!(parsed.invalid_user_ids.is_empty());
    }

    #[test]
    fn user_captures_unknown_fields_into_extra() {
        let raw = r#"{"external_id":"u1","new_braze_field":42}"#;
        let user: ExportedUser = serde_json::from_str(raw).unwrap();
        assert_eq!(user.external_id.as_deref(), Some("u1"));
        assert_eq!(user.extra.get("new_braze_field"), Some(&Value::from(42)));
    }
}
