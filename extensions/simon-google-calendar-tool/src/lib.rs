//! Simon-specific read-only Google Calendar tool for IronClaw.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

wit_bindgen::generate!({
    world: "sandboxed-tool",
    path: "wit/tool.wit",
});

const TOOL_NAME: &str = "simon_google_calendar";
const OAUTH_TOKEN_SECRET: &str = "simon_google_calendar_oauth_token";
const TIME_ZONE: &str = "Asia/Jerusalem";
const CALENDAR_API_BASE: &str = "https://www.googleapis.com/calendar/v3";
const DEFAULT_MAX_RESULTS: u32 = 10;
const MAX_RESULTS_LIMIT: u32 = 20;

struct SimonGoogleCalendarTool;

impl exports::near::agent::tool::Guest for SimonGoogleCalendarTool {
    fn execute(req: exports::near::agent::tool::Request) -> exports::near::agent::tool::Response {
        match execute_inner(
            &req.params,
            req.context.as_deref(),
            HostGoogleCalendarClient,
        ) {
            Ok(output) => exports::near::agent::tool::Response {
                output: Some(output),
                error: None,
            },
            Err(err) => exports::near::agent::tool::Response {
                output: None,
                error: Some(err),
            },
        }
    }

    fn schema() -> String {
        let schema = schemars::schema_for!(CalendarAction);
        serde_json::to_string(&schema).expect("schema serialization is infallible")
    }

    fn description() -> String {
        "Simon-specific read-only Google Calendar lookup. Use for bounded Family calendar \
         list/search requests after trusted actor identity is available from IronClaw context. \
         The tool accepts calendar aliases, never raw calendar IDs, and returns shaped DTOs \
         with opaque event references. V1 has no create, edit, delete, invite, reminder, or \
         reschedule actions."
            .to_string()
    }
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(tag = "action")]
enum CalendarAction {
    #[serde(rename = "calendar.events.list")]
    ListEvents {
        #[serde(default, rename = "requestId")]
        request_id: Option<String>,
        #[serde(rename = "calendarAlias")]
        calendar_alias: CalendarAlias,
        #[serde(rename = "timeMin")]
        time_min: String,
        #[serde(rename = "timeMax")]
        time_max: String,
        #[serde(default, rename = "maxResults")]
        max_results: Option<u32>,
    },
    #[serde(rename = "calendar.events.find")]
    FindEvents {
        #[serde(default, rename = "requestId")]
        request_id: Option<String>,
        #[serde(rename = "calendarAlias")]
        calendar_alias: CalendarAlias,
        #[serde(rename = "timeMin")]
        time_min: String,
        #[serde(rename = "timeMax")]
        time_max: String,
        query: String,
        #[serde(default, rename = "maxResults")]
        max_results: Option<u32>,
    },
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, JsonSchema, PartialEq)]
#[serde(rename_all = "snake_case")]
enum CalendarAlias {
    Family,
}

impl CalendarAlias {
    fn as_str(self) -> &'static str {
        match self {
            CalendarAlias::Family => "family",
        }
    }

    fn calendar_id(self) -> &'static str {
        match self {
            // V1 deliberately keeps raw calendar IDs out of source and bundle
            // metadata. Use a non-sensitive OAuth test account whose primary
            // calendar stands in for the Family alias until private alias
            // configuration is added.
            CalendarAlias::Family => "primary",
        }
    }
}

#[derive(Debug, Deserialize)]
struct JobContext {
    user_id: Option<String>,
    requester_id: Option<String>,
    metadata: Option<Value>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct CalendarSuccess {
    ok: bool,
    request_id: Option<String>,
    action: &'static str,
    actor: String,
    calendar_alias: &'static str,
    time_zone: &'static str,
    count: usize,
    events: Vec<CalendarEventDto>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct CalendarEventDto {
    event_ref: String,
    title: String,
    start: String,
    end: String,
    all_day: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    location: Option<String>,
    status: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct CalendarError {
    ok: bool,
    request_id: Option<String>,
    action: String,
    error: ErrorBody,
}

#[derive(Debug, Serialize)]
struct ErrorBody {
    code: &'static str,
    message: &'static str,
}

#[derive(Debug, Deserialize)]
struct GoogleEventsResponse {
    #[serde(default)]
    items: Vec<GoogleEvent>,
}

#[derive(Debug, Deserialize)]
struct GoogleEvent {
    #[serde(default)]
    id: String,
    #[serde(default)]
    summary: Option<String>,
    #[serde(default)]
    start: GoogleEventTime,
    #[serde(default)]
    end: GoogleEventTime,
    #[serde(default)]
    location: Option<String>,
    #[serde(default)]
    status: Option<String>,
}

#[derive(Default, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GoogleEventTime {
    date: Option<String>,
    date_time: Option<String>,
}

trait GoogleCalendarClient {
    fn list_events(
        &self,
        calendar_id: &str,
        time_min: &str,
        time_max: &str,
        max_results: u32,
        query: Option<&str>,
    ) -> Result<Vec<GoogleEvent>, String>;
}

struct HostGoogleCalendarClient;

impl GoogleCalendarClient for HostGoogleCalendarClient {
    fn list_events(
        &self,
        calendar_id: &str,
        time_min: &str,
        time_max: &str,
        max_results: u32,
        query: Option<&str>,
    ) -> Result<Vec<GoogleEvent>, String> {
        let mut params = vec![
            format!("maxResults={}", max_results),
            "singleEvents=true".to_string(),
            "orderBy=startTime".to_string(),
            format!("timeMin={}", url_encode(time_min)),
            format!("timeMax={}", url_encode(time_max)),
        ];
        if let Some(query) = query {
            params.push(format!("q={}", url_encode(query)));
        }

        let url = format!(
            "{}/calendars/{}/events?{}",
            CALENDAR_API_BASE,
            url_encode(calendar_id),
            params.join("&")
        );

        near::agent::host::log(
            near::agent::host::LogLevel::Info,
            &format!("{} GET /calendar/v3/calendars/<alias>/events", TOOL_NAME),
        );

        let response = near::agent::host::http_request("GET", &url, "{}", None, Some(30_000))?;
        if response.status < 200 || response.status >= 300 {
            return Err(format!(
                "Google Calendar API returned status {}",
                response.status
            ));
        }
        let body = String::from_utf8(response.body)
            .map_err(|err| format!("Google Calendar returned invalid UTF-8: {}", err))?;
        let parsed: GoogleEventsResponse = serde_json::from_str(&body)
            .map_err(|err| format!("Invalid Google response: {}", err))?;
        Ok(parsed.items)
    }
}

fn execute_inner<C: GoogleCalendarClient>(
    params: &str,
    context: Option<&str>,
    client: C,
) -> Result<String, String> {
    let raw: Value =
        serde_json::from_str(params).map_err(|err| format!("Invalid JSON: {}", err))?;
    let action_name = raw_action_name(&raw);
    let request_id = raw_request_id(&raw);
    let context = parse_context(context);

    let actor = actor_from_context(context.as_ref());
    if !is_allowed_actor(actor.as_deref()) {
        return serialize_error(
            request_id,
            action_name,
            "UNAUTHORIZED_ACTOR",
            UNAUTHORIZED_ACTOR,
        );
    }
    let actor = actor.expect("allowed actor is present");

    if !matches!(
        action_name.as_str(),
        "calendar.events.list" | "calendar.events.find"
    ) {
        return serialize_error(
            request_id,
            action_name,
            "UNSUPPORTED_ACTION",
            UNSUPPORTED_ACTION,
        );
    }
    if raw.get("calendarAlias").and_then(Value::as_str) != Some("family") {
        return serialize_error(
            request_id,
            action_name,
            "UNSUPPORTED_CALENDAR_ALIAS",
            UNSUPPORTED_CALENDAR_ALIAS,
        );
    }

    let action: CalendarAction =
        serde_json::from_value(raw).map_err(|err| format!("Invalid parameters: {}", err))?;
    let request = NormalizedRequest::from_action(action)?;
    if !valid_time_window(&request.time_min, &request.time_max) {
        return serialize_error(
            request.request_id,
            request.action.to_string(),
            "INVALID_TIME_WINDOW",
            INVALID_TIME_WINDOW,
        );
    }
    let Some(max_results) = normalized_max_results(request.max_results) else {
        return serialize_error(
            request.request_id,
            request.action.to_string(),
            "INVALID_MAX_RESULTS",
            INVALID_MAX_RESULTS,
        );
    };

    if !near::agent::host::secret_exists(OAUTH_TOKEN_SECRET) {
        return serialize_error(
            request.request_id,
            request.action.to_string(),
            "AUTH_REQUIRED",
            AUTH_REQUIRED,
        );
    }

    let google_events = client.list_events(
        request.calendar_alias.calendar_id(),
        &request.time_min,
        &request.time_max,
        max_results,
        request.query.as_deref(),
    )?;
    let events = google_events
        .into_iter()
        .map(|event| shape_event(request.calendar_alias, event))
        .collect::<Vec<_>>();

    let success = CalendarSuccess {
        ok: true,
        request_id: request.request_id,
        action: request.action,
        actor,
        calendar_alias: request.calendar_alias.as_str(),
        time_zone: TIME_ZONE,
        count: events.len(),
        events,
    };
    serde_json::to_string(&success).map_err(|err| err.to_string())
}

struct NormalizedRequest {
    request_id: Option<String>,
    action: &'static str,
    calendar_alias: CalendarAlias,
    time_min: String,
    time_max: String,
    query: Option<String>,
    max_results: Option<u32>,
}

impl NormalizedRequest {
    fn from_action(action: CalendarAction) -> Result<Self, String> {
        match action {
            CalendarAction::ListEvents {
                request_id,
                calendar_alias,
                time_min,
                time_max,
                max_results,
            } => Ok(Self {
                request_id,
                action: "calendar.events.list",
                calendar_alias,
                time_min,
                time_max,
                query: None,
                max_results,
            }),
            CalendarAction::FindEvents {
                request_id,
                calendar_alias,
                time_min,
                time_max,
                query,
                max_results,
            } => Ok(Self {
                request_id,
                action: "calendar.events.find",
                calendar_alias,
                time_min,
                time_max,
                query: Some(query),
                max_results,
            }),
        }
    }
}

const UNAUTHORIZED_ACTOR: &str = "This caller is not approved to access Simon calendar data.";
const UNSUPPORTED_ACTION: &str = "Only read-only calendar event lookup actions are supported.";
const UNSUPPORTED_CALENDAR_ALIAS: &str =
    "calendarAlias must be one of the configured Simon calendar aliases.";
const INVALID_TIME_WINDOW: &str =
    "timeMin and timeMax must be RFC3339 timestamps and timeMax must be after timeMin.";
const INVALID_MAX_RESULTS: &str = "maxResults must be an integer from 1 to 20.";
const AUTH_REQUIRED: &str = "Simon Google Calendar OAuth is not configured.";

fn parse_context(context: Option<&str>) -> Option<JobContext> {
    context.and_then(|value| serde_json::from_str(value).ok())
}

fn actor_from_context(context: Option<&JobContext>) -> Option<String> {
    let context = context?;
    if let Some(requester_id) = context
        .requester_id
        .as_deref()
        .filter(|value| !value.is_empty())
    {
        return Some(requester_id.to_string());
    }
    if context.user_id.as_deref() == Some("local_ironclaw_bot") {
        return Some("local_ironclaw_bot".to_string());
    }
    let metadata = context.metadata.as_ref()?.as_object()?;
    for key in ["simon_identity", "canonical_id", "actor"] {
        if let Some(value) = metadata.get(key).and_then(Value::as_str) {
            if !value.is_empty() {
                return Some(value.to_string());
            }
        }
    }
    None
}

fn is_allowed_actor(actor: Option<&str>) -> bool {
    matches!(actor, Some("alon" | "local_ironclaw_bot"))
}

fn valid_time_window(time_min: &str, time_max: &str) -> bool {
    if time_min.is_empty() || time_max.is_empty() {
        return false;
    }
    // RFC3339 strings with the same offset sort chronologically. This contract
    // requires explicit Asia/Jerusalem windows, so lexical ordering is enough
    // for the tool-side guard while Google handles exact timestamp parsing.
    time_min.contains('T') && time_max.contains('T') && time_max > time_min
}

fn normalized_max_results(value: Option<u32>) -> Option<u32> {
    let value = value.unwrap_or(DEFAULT_MAX_RESULTS);
    if (1..=MAX_RESULTS_LIMIT).contains(&value) {
        Some(value)
    } else {
        None
    }
}

fn shape_event(calendar_alias: CalendarAlias, event: GoogleEvent) -> CalendarEventDto {
    let start = event
        .start
        .date_time
        .clone()
        .or(event.start.date.clone())
        .unwrap_or_default();
    let end = event
        .end
        .date_time
        .clone()
        .or(event.end.date.clone())
        .unwrap_or_default();
    let all_day = event.start.date.is_some() && event.start.date_time.is_none();
    CalendarEventDto {
        event_ref: safe_event_ref(calendar_alias, &event.id),
        title: event.summary.unwrap_or_else(|| "(No title)".to_string()),
        start,
        end,
        all_day,
        location: event.location.filter(|value| !value.is_empty()),
        status: event.status.unwrap_or_else(|| "confirmed".to_string()),
    }
}

fn safe_event_ref(calendar_alias: CalendarAlias, raw_event_id: &str) -> String {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in format!("{}:{}", calendar_alias.as_str(), raw_event_id).bytes() {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("evt_{hash:016x}")
}

fn serialize_error(
    request_id: Option<String>,
    action: String,
    code: &'static str,
    message: &'static str,
) -> Result<String, String> {
    serde_json::to_string(&CalendarError {
        ok: false,
        request_id,
        action,
        error: ErrorBody { code, message },
    })
    .map_err(|err| err.to_string())
}

fn raw_action_name(value: &Value) -> String {
    value
        .get("action")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string()
}

fn raw_request_id(value: &Value) -> Option<String> {
    value
        .get("requestId")
        .and_then(Value::as_str)
        .map(str::to_string)
}

fn url_encode(value: &str) -> String {
    let mut encoded = String::with_capacity(value.len());
    for byte in value.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                encoded.push(byte as char);
            }
            _ => {
                encoded.push('%');
                encoded.push(char::from(HEX[(byte >> 4) as usize]));
                encoded.push(char::from(HEX[(byte & 0x0F) as usize]));
            }
        }
    }
    encoded
}

const HEX: [u8; 16] = *b"0123456789ABCDEF";

#[cfg(target_arch = "wasm32")]
export!(SimonGoogleCalendarTool);

#[cfg(test)]
mod tests {
    use super::*;

    impl Clone for GoogleEvent {
        fn clone(&self) -> Self {
            Self {
                id: self.id.clone(),
                summary: self.summary.clone(),
                start: self.start.clone(),
                end: self.end.clone(),
                location: self.location.clone(),
                status: self.status.clone(),
            }
        }
    }

    impl Clone for GoogleEventTime {
        fn clone(&self) -> Self {
            Self {
                date: self.date.clone(),
                date_time: self.date_time.clone(),
            }
        }
    }

    fn timed_event() -> GoogleEvent {
        GoogleEvent {
            id: "raw-google-event-id".to_string(),
            summary: Some("Meeting with Team".to_string()),
            start: GoogleEventTime {
                date: None,
                date_time: Some("2026-04-27T10:00:00+03:00".to_string()),
            },
            end: GoogleEventTime {
                date: None,
                date_time: Some("2026-04-27T11:00:00+03:00".to_string()),
            },
            location: Some("optional".to_string()),
            status: Some("confirmed".to_string()),
        }
    }

    #[test]
    fn actor_prefers_requester_id_from_context() {
        let context = JobContext {
            user_id: Some("owner".to_string()),
            requester_id: Some("alon".to_string()),
            metadata: None,
        };
        assert_eq!(actor_from_context(Some(&context)).as_deref(), Some("alon"));
    }

    #[test]
    fn shlomit_is_not_allowed_in_v1() {
        assert!(!is_allowed_actor(Some("shlomit")));
    }

    #[test]
    fn event_ref_does_not_expose_raw_google_event_id() {
        let shaped = shape_event(CalendarAlias::Family, timed_event());
        assert!(shaped.event_ref.starts_with("evt_"));
        assert!(!shaped.event_ref.contains("raw-google-event-id"));
        assert_eq!(shaped.title, "Meeting with Team");
        assert!(!shaped.all_day);
    }

    #[test]
    fn all_day_event_is_normalized() {
        let event = GoogleEvent {
            id: "all-day".to_string(),
            summary: Some("School vacation".to_string()),
            start: GoogleEventTime {
                date: Some("2026-04-27".to_string()),
                date_time: None,
            },
            end: GoogleEventTime {
                date: Some("2026-04-28".to_string()),
                date_time: None,
            },
            location: None,
            status: None,
        };
        let shaped = shape_event(CalendarAlias::Family, event);
        assert_eq!(shaped.start, "2026-04-27");
        assert_eq!(shaped.end, "2026-04-28");
        assert!(shaped.all_day);
        assert_eq!(shaped.status, "confirmed");
    }

    #[test]
    fn invalid_time_window_is_rejected_before_api_shape() {
        assert!(!valid_time_window(
            "2026-04-28T00:00:00+03:00",
            "2026-04-27T00:00:00+03:00"
        ));
    }

    #[test]
    fn schema_exposes_only_read_actions() {
        let schema = serde_json::to_string(&schemars::schema_for!(CalendarAction)).unwrap();
        assert!(schema.contains("calendar.events.list"));
        assert!(schema.contains("calendar.events.find"));
        assert!(!schema.contains("create"));
        assert!(!schema.contains("delete"));
    }
}
