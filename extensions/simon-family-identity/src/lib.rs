use serde::{Deserialize, Serialize};
use serde_json::Value;

wit_bindgen::generate!({
    world: "sandboxed-tool",
    path: "wit/tool.wit",
});

const REGISTRY_PATH: &str = "channels/simon_telegram_channel/state/simon_family_profiles.json";

struct SimonFamilyIdentityTool;

impl exports::near::agent::tool::Guest for SimonFamilyIdentityTool {
    fn execute(req: exports::near::agent::tool::Request) -> exports::near::agent::tool::Response {
        match execute_inner(&req.params) {
            Ok(output) => exports::near::agent::tool::Response {
                output: Some(output),
                error: None,
            },
            Err(error) => exports::near::agent::tool::Response {
                output: None,
                error: Some(error),
            },
        }
    }

    fn schema() -> String {
        SCHEMA.to_string()
    }

    fn description() -> String {
        "Canonical Simon family identity registry. Use it to inspect recipient identities, \
         delivery readiness, and per-user workspace seed docs for Simon family deployments."
            .to_string()
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct FamilyRegistry {
    schema_version: u32,
    family_timezone: String,
    users: Vec<FamilyUserProfile>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct FamilyUserProfile {
    canonical_id: String,
    display_name: String,
    role: String,
    status: String,
    preferred_language: String,
    timezone: String,
    delivery_preferences: DeliveryPreferences,
    enabled_routines: Vec<String>,
    channel_bindings: Vec<ChannelBinding>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct DeliveryPreferences {
    telegram_enabled: bool,
    proactive_daily_briefing: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct ChannelBinding {
    channel: String,
    target: Option<String>,
    paired: bool,
    readiness: String,
    proactive_enabled: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct RecipientRequest {
    #[serde(rename = "action")]
    _action: String,
    #[serde(rename = "recipientIdentity")]
    recipient_identity: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct DeliveryRequest {
    #[serde(rename = "action")]
    _action: String,
    #[serde(default)]
    channel: Option<String>,
    #[serde(default)]
    routine: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct RegistryRequest {
    #[serde(rename = "action")]
    _action: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ToolResponse<T> {
    ok: bool,
    action: String,
    source: String,
    #[serde(flatten)]
    payload: T,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct RegistryPayload {
    registry: FamilyRegistry,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct RecipientPayload {
    profile: FamilyUserProfile,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct DeliveryPayload {
    recipients: Vec<DeliveryTarget>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct DeliveryTarget {
    canonical_id: String,
    display_name: String,
    status: String,
    channel: String,
    target: Option<String>,
    readiness: String,
    proactive_enabled: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SeedDocsPayload {
    recipient_identity: String,
    files: Vec<SeedDoc>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SeedDoc {
    path: String,
    content: String,
}

fn execute_inner(params: &str) -> Result<String, String> {
    let raw: Value = serde_json::from_str(params).map_err(|_| INVALID_JSON.to_string())?;
    let action = raw
        .get("action")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();

    let (source, registry) = load_registry();

    match action.as_str() {
        "get_family_registry" => {
            let _request: RegistryRequest =
                serde_json::from_value(raw).map_err(|_| INVALID_PARAMETERS.to_string())?;
            serialize_response(action, source, RegistryPayload { registry })
        }
        "get_recipient_profile" => {
            let request: RecipientRequest =
                serde_json::from_value(raw).map_err(|_| INVALID_PARAMETERS.to_string())?;
            let profile = registry
                .users
                .iter()
                .find(|profile| profile.canonical_id == request.recipient_identity)
                .cloned()
                .ok_or_else(|| INVALID_RECIPIENT.to_string())?;
            serialize_response(action, source, RecipientPayload { profile })
        }
        "resolve_delivery_targets" => {
            let request: DeliveryRequest =
                serde_json::from_value(raw).map_err(|_| INVALID_PARAMETERS.to_string())?;
            let channel = request
                .channel
                .unwrap_or_else(|| "simon_telegram_channel".to_string());
            let routine = request
                .routine
                .unwrap_or_else(|| "daily_briefing".to_string());

            let recipients = registry
                .users
                .iter()
                .filter(|profile| profile.enabled_routines.iter().any(|item| item == &routine))
                .filter_map(|profile| {
                    profile
                        .channel_bindings
                        .iter()
                        .find(|binding| binding.channel == channel)
                        .map(|binding| DeliveryTarget {
                            canonical_id: profile.canonical_id.clone(),
                            display_name: profile.display_name.clone(),
                            status: profile.status.clone(),
                            channel: binding.channel.clone(),
                            target: binding.target.clone(),
                            readiness: binding.readiness.clone(),
                            proactive_enabled: binding.proactive_enabled,
                        })
                })
                .collect();

            serialize_response(action, source, DeliveryPayload { recipients })
        }
        "render_user_scope_docs" => {
            let request: RecipientRequest =
                serde_json::from_value(raw).map_err(|_| INVALID_PARAMETERS.to_string())?;
            let profile = registry
                .users
                .iter()
                .find(|profile| profile.canonical_id == request.recipient_identity)
                .cloned()
                .ok_or_else(|| INVALID_RECIPIENT.to_string())?;
            let files = render_seed_docs(&profile);
            serialize_response(
                action,
                source,
                SeedDocsPayload {
                    recipient_identity: profile.canonical_id,
                    files,
                },
            )
        }
        _ => Err(UNSUPPORTED_ACTION.to_string()),
    }
}

fn serialize_response<T: Serialize>(
    action: String,
    source: String,
    payload: T,
) -> Result<String, String> {
    serde_json::to_string(&ToolResponse {
        ok: true,
        action,
        source,
        payload,
    })
    .map_err(|err| err.to_string())
}

fn load_registry() -> (String, FamilyRegistry) {
    if let Some(raw) = near::agent::host::workspace_read(REGISTRY_PATH) {
        if let Ok(registry) = serde_json::from_str::<FamilyRegistry>(raw.trim()) {
            return ("workspace".to_string(), registry);
        }
    }

    ("defaults".to_string(), default_registry())
}

fn default_registry() -> FamilyRegistry {
    FamilyRegistry {
        schema_version: 1,
        family_timezone: "Asia/Jerusalem".to_string(),
        users: vec![
            FamilyUserProfile {
                canonical_id: "alon".to_string(),
                display_name: "Alon".to_string(),
                role: "primary_parent_admin".to_string(),
                status: "active".to_string(),
                preferred_language: "he".to_string(),
                timezone: "Asia/Jerusalem".to_string(),
                delivery_preferences: DeliveryPreferences {
                    telegram_enabled: true,
                    proactive_daily_briefing: true,
                },
                enabled_routines: vec!["daily_briefing".to_string()],
                channel_bindings: vec![ChannelBinding {
                    channel: "simon_telegram_channel".to_string(),
                    target: None,
                    paired: false,
                    readiness: "pending_pairing".to_string(),
                    proactive_enabled: true,
                }],
            },
            FamilyUserProfile {
                canonical_id: "shlomit".to_string(),
                display_name: "Shlomit".to_string(),
                role: "second_parent".to_string(),
                status: "dormant".to_string(),
                preferred_language: "he".to_string(),
                timezone: "Asia/Jerusalem".to_string(),
                delivery_preferences: DeliveryPreferences {
                    telegram_enabled: false,
                    proactive_daily_briefing: false,
                },
                enabled_routines: Vec::new(),
                channel_bindings: vec![ChannelBinding {
                    channel: "simon_telegram_channel".to_string(),
                    target: None,
                    paired: false,
                    readiness: "dormant".to_string(),
                    proactive_enabled: false,
                }],
            },
        ],
    }
}

fn render_seed_docs(profile: &FamilyUserProfile) -> Vec<SeedDoc> {
    vec![
        SeedDoc {
            path: "AGENTS.md".to_string(),
            content: format!(
                "# Agent Instructions\n\n- Treat {} as the private user for this scope.\n- Keep family privacy boundaries explicit.\n- Prefer Simon's canonical identity metadata over message-text guesses.\n- Use Hebrew by default unless {} writes in another language or has a stored preference override.\n",
                profile.display_name, profile.display_name
            ),
        },
        SeedDoc {
            path: "IDENTITY.md".to_string(),
            content: format!(
                "# Identity\n\n- Simon is a calm, direct family assistant.\n- This scope is seeded for {} (`{}`).\n- Role: {}\n- Status: {}\n",
                profile.display_name, profile.canonical_id, profile.role, profile.status
            ),
        },
        SeedDoc {
            path: "SOUL.md".to_string(),
            content: "# Soul\n\n- Keep private things private.\n- Do not leak context across parents.\n- Prefer tool-grounded answers over guesswork.\n- Ask before risky or state-changing actions.\n".to_string(),
        },
        SeedDoc {
            path: "TOOLS.md".to_string(),
            content: format!(
                "# Tools\n\n- Preferred language: {}\n- Preferred timezone: {}\n- Telegram enabled: {}\n- Daily briefing enabled: {}\n",
                profile.preferred_language,
                profile.timezone,
                profile.delivery_preferences.telegram_enabled,
                profile.delivery_preferences.proactive_daily_briefing
            ),
        },
    ]
}

const INVALID_JSON: &str = "Simon Family Identity received invalid JSON parameters.";
const INVALID_PARAMETERS: &str =
    "Simon Family Identity requires a valid action and the matching recipient/channel fields.";
const INVALID_RECIPIENT: &str = "recipientIdentity must match a canonical Simon family user.";
const UNSUPPORTED_ACTION: &str =
    "Supported actions: get_family_registry, get_recipient_profile, resolve_delivery_targets, render_user_scope_docs.";

const SCHEMA: &str = r#"{
  "type": "object",
  "properties": {
    "action": {
      "type": "string",
      "enum": [
        "get_family_registry",
        "get_recipient_profile",
        "resolve_delivery_targets",
        "render_user_scope_docs"
      ]
    },
    "recipientIdentity": {
      "type": "string",
      "enum": ["alon", "shlomit"]
    },
    "channel": {
      "type": "string"
    },
    "routine": {
      "type": "string"
    }
  },
  "required": ["action"],
  "additionalProperties": false
}"#;

#[cfg(target_arch = "wasm32")]
export!(SimonFamilyIdentityTool);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_registry_contains_both_parents() {
        let registry = default_registry();
        assert_eq!(registry.users.len(), 2);
        assert_eq!(registry.users[0].canonical_id, "alon");
        assert_eq!(registry.users[1].canonical_id, "shlomit");
    }

    #[test]
    fn seed_docs_include_identity_file() {
        let profile = default_registry().users.remove(0);
        let files = render_seed_docs(&profile);
        assert!(files.iter().any(|file| file.path == "IDENTITY.md"));
    }
}
