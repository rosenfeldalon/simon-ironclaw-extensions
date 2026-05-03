use serde::{Deserialize, Serialize};
use serde_json::Value;

wit_bindgen::generate!({
    world: "sandboxed-tool",
    path: "wit/tool.wit",
});

struct SimonSetupTool;

impl exports::near::agent::tool::Guest for SimonSetupTool {
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
        "Simon install-pack bootstrap tool. Use it to generate the install manifest, canonical \
         family registry preview, workspace seed docs, and operator runbook for a fresh Simon deployment."
            .to_string()
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct SetupProfileInput {
    canonical_id: String,
    display_name: String,
    role: String,
    status: String,
    preferred_language: String,
    timezone: String,
    proactive_daily_briefing: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct SetupRequest {
    #[serde(rename = "action")]
    _action: String,
    #[serde(default)]
    family_timezone: Option<String>,
    #[serde(default)]
    profiles: Option<Vec<SetupProfileInput>>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct Response<T> {
    ok: bool,
    action: String,
    #[serde(flatten)]
    payload: T,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct InstallPackPayload {
    install_order: Vec<String>,
    bootstrap_order: Vec<String>,
    extensions: Vec<InstallableExtension>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct InstallableExtension {
    name: String,
    kind: String,
    purpose: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct RegistryPayload {
    registry: FamilyRegistry,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SeedDocsPayload {
    recipients: Vec<RecipientSeedDocs>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct RecipientSeedDocs {
    recipient_identity: String,
    files: Vec<SeedDoc>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SeedDoc {
    path: String,
    content: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct RunbookPayload {
    summary: String,
    steps: Vec<String>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct FamilyRegistry {
    schema_version: u32,
    family_timezone: String,
    users: Vec<FamilyUserProfile>,
}

#[derive(Clone, Debug, Serialize)]
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

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct DeliveryPreferences {
    telegram_enabled: bool,
    proactive_daily_briefing: bool,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ChannelBinding {
    channel: String,
    target: Option<String>,
    paired: bool,
    readiness: String,
    proactive_enabled: bool,
}

fn execute_inner(params: &str) -> Result<String, String> {
    let raw: Value = serde_json::from_str(params).map_err(|_| INVALID_JSON.to_string())?;
    let action = raw
        .get("action")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();
    let request: SetupRequest =
        serde_json::from_value(raw).map_err(|_| INVALID_PARAMETERS.to_string())?;
    let registry = build_registry(request.family_timezone, request.profiles);

    match action.as_str() {
        "build_install_pack_manifest" => serialize(action, build_install_pack_manifest()),
        "build_family_registry" => serialize(action, RegistryPayload { registry }),
        "render_user_scope_docs" => {
            let recipients = registry
                .users
                .iter()
                .map(|profile| RecipientSeedDocs {
                    recipient_identity: profile.canonical_id.clone(),
                    files: render_seed_docs(profile),
                })
                .collect();
            serialize(action, SeedDocsPayload { recipients })
        }
        "build_bootstrap_runbook" => serialize(
            action,
            RunbookPayload {
                summary: "Install the Simon pack, run setup, pair recipients one at a time, validate outbound delivery, then enable proactive routines.".to_string(),
                steps: vec![
                    "Install simon_telegram_channel, simon_google_calendar, simon_family_identity, simon_daily_briefing, and simon_setup in that order.".to_string(),
                    "Run simon_setup to generate the canonical family registry preview and per-user workspace seeds.".to_string(),
                    "Pair Alon first and validate a manual Telegram outbound send.".to_string(),
                    "Pair Shlomit only after readiness checks pass, then validate a separate outbound send.".to_string(),
                    "Enable daily briefing routines only for recipients whose channel binding is ready.".to_string(),
                ],
            },
        ),
        _ => Err(UNSUPPORTED_ACTION.to_string()),
    }
}

fn serialize<T: Serialize>(action: String, payload: T) -> Result<String, String> {
    serde_json::to_string(&Response {
        ok: true,
        action,
        payload,
    })
    .map_err(|err| err.to_string())
}

fn build_install_pack_manifest() -> InstallPackPayload {
    InstallPackPayload {
        install_order: vec![
            "simon_telegram_channel".to_string(),
            "simon_google_calendar".to_string(),
            "simon_family_identity".to_string(),
            "simon_daily_briefing".to_string(),
            "simon_setup".to_string(),
        ],
        bootstrap_order: vec![
            "install_extensions".to_string(),
            "run_simon_setup".to_string(),
            "pair_alon".to_string(),
            "validate_alon_outbound".to_string(),
            "pair_shlomit".to_string(),
            "validate_shlomit_outbound".to_string(),
            "enable_multi_recipient_routines".to_string(),
        ],
        extensions: vec![
            InstallableExtension {
                name: "simon_telegram_channel".to_string(),
                kind: "wasm_channel".to_string(),
                purpose: "Private Telegram channel and pairing-backed delivery surface."
                    .to_string(),
            },
            InstallableExtension {
                name: "simon_google_calendar".to_string(),
                kind: "wasm_tool".to_string(),
                purpose: "Google Calendar read/write boundary for Simon.".to_string(),
            },
            InstallableExtension {
                name: "simon_family_identity".to_string(),
                kind: "wasm_tool".to_string(),
                purpose: "Canonical Simon family registry and recipient-resolution helper."
                    .to_string(),
            },
            InstallableExtension {
                name: "simon_daily_briefing".to_string(),
                kind: "wasm_tool".to_string(),
                purpose: "Shared-facts plus recipient-render daily briefing generator.".to_string(),
            },
            InstallableExtension {
                name: "simon_setup".to_string(),
                kind: "wasm_tool".to_string(),
                purpose: "Install-pack bootstrap planner and workspace seed generator.".to_string(),
            },
        ],
    }
}

fn build_registry(
    family_timezone: Option<String>,
    profiles: Option<Vec<SetupProfileInput>>,
) -> FamilyRegistry {
    let timezone = family_timezone.unwrap_or_else(|| "Asia/Jerusalem".to_string());
    let users = profiles.unwrap_or_else(default_profiles);

    FamilyRegistry {
        schema_version: 1,
        family_timezone: timezone.clone(),
        users: users
            .into_iter()
            .map(|profile| FamilyUserProfile {
                canonical_id: profile.canonical_id,
                display_name: profile.display_name,
                role: profile.role,
                status: profile.status.clone(),
                preferred_language: profile.preferred_language,
                timezone: profile.timezone,
                delivery_preferences: DeliveryPreferences {
                    telegram_enabled: profile.status == "active",
                    proactive_daily_briefing: profile.proactive_daily_briefing,
                },
                enabled_routines: if profile.proactive_daily_briefing {
                    vec!["daily_briefing".to_string()]
                } else {
                    Vec::new()
                },
                channel_bindings: vec![ChannelBinding {
                    channel: "simon_telegram_channel".to_string(),
                    target: None,
                    paired: false,
                    readiness: if profile.status == "active" {
                        "pending_pairing".to_string()
                    } else {
                        "dormant".to_string()
                    },
                    proactive_enabled: profile.proactive_daily_briefing,
                }],
            })
            .collect(),
    }
}

fn default_profiles() -> Vec<SetupProfileInput> {
    vec![
        SetupProfileInput {
            canonical_id: "alon".to_string(),
            display_name: "Alon".to_string(),
            role: "primary_parent_admin".to_string(),
            status: "active".to_string(),
            preferred_language: "he".to_string(),
            timezone: "Asia/Jerusalem".to_string(),
            proactive_daily_briefing: true,
        },
        SetupProfileInput {
            canonical_id: "shlomit".to_string(),
            display_name: "Shlomit".to_string(),
            role: "second_parent".to_string(),
            status: "dormant".to_string(),
            preferred_language: "he".to_string(),
            timezone: "Asia/Jerusalem".to_string(),
            proactive_daily_briefing: false,
        },
    ]
}

fn render_seed_docs(profile: &FamilyUserProfile) -> Vec<SeedDoc> {
    vec![
        SeedDoc {
            path: "AGENTS.md".to_string(),
            content: format!(
                "# Agent Instructions\n\n- This scope is for {} (`{}`).\n- Respect separate-parent privacy boundaries.\n- Prefer canonical Simon metadata over message-text identity guesses.\n",
                profile.display_name, profile.canonical_id
            ),
        },
        SeedDoc {
            path: "IDENTITY.md".to_string(),
            content: format!(
                "# Identity\n\n- Simon is the family assistant.\n- Recipient: {} (`{}`).\n- Role: {}\n- Status: {}\n",
                profile.display_name, profile.canonical_id, profile.role, profile.status
            ),
        },
        SeedDoc {
            path: "SOUL.md".to_string(),
            content: "# Soul\n\n- Private things stay private.\n- Ask before risky actions.\n- Stay grounded in tool data.\n".to_string(),
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

const INVALID_JSON: &str = "Simon Setup received invalid JSON parameters.";
const INVALID_PARAMETERS: &str =
    "Simon Setup requires a supported action and optional family_timezone/profiles fields.";
const UNSUPPORTED_ACTION: &str =
    "Supported actions: build_install_pack_manifest, build_family_registry, render_user_scope_docs, build_bootstrap_runbook.";

const SCHEMA: &str = r#"{
  "type": "object",
  "properties": {
    "action": {
      "type": "string",
      "enum": [
        "build_install_pack_manifest",
        "build_family_registry",
        "render_user_scope_docs",
        "build_bootstrap_runbook"
      ]
    },
    "family_timezone": {
      "type": "string"
    },
    "profiles": {
      "type": "array"
    }
  },
  "required": ["action"],
  "additionalProperties": false
}"#;

#[cfg(target_arch = "wasm32")]
export!(SimonSetupTool);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn install_pack_manifest_contains_expected_order() {
        let payload = build_install_pack_manifest();
        assert_eq!(payload.install_order[0], "simon_telegram_channel");
        assert_eq!(payload.install_order[4], "simon_setup");
    }

    #[test]
    fn default_registry_marks_shlomit_dormant() {
        let registry = build_registry(None, None);
        let shlomit = registry
            .users
            .iter()
            .find(|profile| profile.canonical_id == "shlomit")
            .unwrap();
        assert_eq!(shlomit.status, "dormant");
        assert!(!shlomit.delivery_preferences.telegram_enabled);
    }
}
