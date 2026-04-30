//! Simon-specific Google Calendar tool for IronClaw.

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
const FAMILY_CALENDAR_ID_PATH: &str = ".system/simon_google_calendar/family_calendar_id";
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
        "Simon-specific Google Calendar access for the Family calendar. Use after trusted \
         actor identity is available from IronClaw context. Supports bounded event list/search \
         plus create, update, and delete. The tool accepts calendar aliases, never raw calendar \
         IDs, and returns shaped DTOs with opaque event references."
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
    #[serde(rename = "calendar.events.create")]
    CreateEvent {
        #[serde(default, rename = "requestId")]
        request_id: Option<String>,
        #[serde(rename = "calendarAlias")]
        calendar_alias: CalendarAlias,
        title: String,
        start: String,
        end: String,
        #[serde(default)]
        location: Option<String>,
        #[serde(default)]
        notes: Option<String>,
    },
    #[serde(rename = "calendar.events.update")]
    UpdateEvent {
        #[serde(default, rename = "requestId")]
        request_id: Option<String>,
        #[serde(rename = "calendarAlias")]
        calendar_alias: CalendarAlias,
        #[serde(rename = "eventRef")]
        event_ref: String,
        #[serde(default)]
        title: Option<String>,
        #[serde(default)]
        start: Option<String>,
        #[serde(default)]
        end: Option<String>,
        #[serde(default)]
        location: Option<String>,
        #[serde(default)]
        notes: Option<String>,
    },
    #[serde(rename = "calendar.events.delete")]
    DeleteEvent {
        #[serde(default, rename = "requestId")]
        request_id: Option<String>,
        #[serde(rename = "calendarAlias")]
        calendar_alias: CalendarAlias,
        #[serde(rename = "eventRef")]
        event_ref: String,
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

    fn calendar_id(self) -> String {
        match self {
            CalendarAlias::Family => near::agent::host::workspace_read(FAMILY_CALENDAR_ID_PATH)
                .map(|value| value.trim().to_string())
                .filter(|value| !value.is_empty())
                .unwrap_or_else(|| "primary".to_string()),
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
struct CalendarMutationSuccess {
    ok: bool,
    request_id: Option<String>,
    action: &'static str,
    actor: String,
    calendar_alias: &'static str,
    time_zone: &'static str,
    event: CalendarEventDto,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct CalendarDeleteSuccess {
    ok: bool,
    request_id: Option<String>,
    action: &'static str,
    actor: String,
    calendar_alias: &'static str,
    event_ref: String,
    deleted: bool,
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

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct GoogleEventWrite {
    #[serde(skip_serializing_if = "Option::is_none")]
    summary: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    start: Option<GoogleEventWriteTime>,
    #[serde(skip_serializing_if = "Option::is_none")]
    end: Option<GoogleEventWriteTime>,
    #[serde(skip_serializing_if = "Option::is_none")]
    location: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct GoogleEventWriteTime {
    date_time: String,
    time_zone: &'static str,
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

    fn create_event(
        &self,
        calendar_id: &str,
        event: &GoogleEventWrite,
    ) -> Result<GoogleEvent, String>;

    fn update_event(
        &self,
        calendar_id: &str,
        event_id: &str,
        patch: &GoogleEventWrite,
    ) -> Result<GoogleEvent, String>;

    fn delete_event(&self, calendar_id: &str, event_id: &str) -> Result<(), String>;
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

    fn create_event(
        &self,
        calendar_id: &str,
        event: &GoogleEventWrite,
    ) -> Result<GoogleEvent, String> {
        let url = format!(
            "{}/calendars/{}/events",
            CALENDAR_API_BASE,
            url_encode(calendar_id)
        );
        request_event("POST", &url, event, "POST")
    }

    fn update_event(
        &self,
        calendar_id: &str,
        event_id: &str,
        patch: &GoogleEventWrite,
    ) -> Result<GoogleEvent, String> {
        let url = format!(
            "{}/calendars/{}/events/{}",
            CALENDAR_API_BASE,
            url_encode(calendar_id),
            url_encode(event_id)
        );
        request_event("PATCH", &url, patch, "PATCH")
    }

    fn delete_event(&self, calendar_id: &str, event_id: &str) -> Result<(), String> {
        let url = format!(
            "{}/calendars/{}/events/{}",
            CALENDAR_API_BASE,
            url_encode(calendar_id),
            url_encode(event_id)
        );
        near::agent::host::log(
            near::agent::host::LogLevel::Info,
            &format!(
                "{} DELETE /calendar/v3/calendars/<alias>/events/<event>",
                TOOL_NAME
            ),
        );
        let response = near::agent::host::http_request("DELETE", &url, "{}", None, Some(30_000))?;
        if response.status < 200 || response.status >= 300 {
            return Err(format!(
                "Google Calendar API returned status {}",
                response.status
            ));
        }
        Ok(())
    }
}

fn request_event(
    method: &str,
    url: &str,
    event: &GoogleEventWrite,
    log_method: &str,
) -> Result<GoogleEvent, String> {
    near::agent::host::log(
        near::agent::host::LogLevel::Info,
        &format!(
            "{} {} /calendar/v3/calendars/<alias>/events",
            TOOL_NAME, log_method
        ),
    );
    let body = serde_json::to_vec(event).map_err(|err| err.to_string())?;
    let response = near::agent::host::http_request(
        method,
        url,
        r#"{"Content-Type":"application/json"}"#,
        Some(&body),
        Some(30_000),
    )?;
    if response.status < 200 || response.status >= 300 {
        return Err(format!(
            "Google Calendar API returned status {}",
            response.status
        ));
    }
    let body = String::from_utf8(response.body)
        .map_err(|err| format!("Google Calendar returned invalid UTF-8: {}", err))?;
    serde_json::from_str(&body).map_err(|err| format!("Invalid Google response: {}", err))
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
        "calendar.events.list"
            | "calendar.events.find"
            | "calendar.events.create"
            | "calendar.events.update"
            | "calendar.events.delete"
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
    let auth_request_id = request.request_id();
    let auth_action = request.action().to_string();

    if !near::agent::host::secret_exists(OAUTH_TOKEN_SECRET) {
        return serialize_error(auth_request_id, auth_action, "AUTH_REQUIRED", AUTH_REQUIRED);
    }

    match request {
        NormalizedRequest::ListOrFind(read) => {
            if !valid_time_window(&read.time_min, &read.time_max) {
                return serialize_error(
                    read.request_id,
                    read.action.to_string(),
                    "INVALID_TIME_WINDOW",
                    INVALID_TIME_WINDOW,
                );
            }
            let Some(max_results) = normalized_max_results(read.max_results) else {
                return serialize_error(
                    read.request_id,
                    read.action.to_string(),
                    "INVALID_MAX_RESULTS",
                    INVALID_MAX_RESULTS,
                );
            };
            let google_events = client.list_events(
                &read.calendar_alias.calendar_id(),
                &read.time_min,
                &read.time_max,
                max_results,
                read.query.as_deref(),
            )?;
            let events = google_events
                .into_iter()
                .map(|event| shape_event(read.calendar_alias, event))
                .collect::<Vec<_>>();

            let success = CalendarSuccess {
                ok: true,
                request_id: read.request_id,
                action: read.action,
                actor,
                calendar_alias: read.calendar_alias.as_str(),
                time_zone: TIME_ZONE,
                count: events.len(),
                events,
            };
            serde_json::to_string(&success).map_err(|err| err.to_string())
        }
        NormalizedRequest::Create(write) => {
            if !valid_time_window(&write.start, &write.end) || write.title.trim().is_empty() {
                return serialize_error(
                    write.request_id,
                    write.action.to_string(),
                    "INVALID_EVENT_INPUT",
                    INVALID_EVENT_INPUT,
                );
            }
            let event = GoogleEventWrite {
                summary: Some(write.title),
                start: Some(write_time(write.start)),
                end: Some(write_time(write.end)),
                location: clean_optional(write.location),
                description: clean_optional(write.notes),
            };
            let google_event = client.create_event(&write.calendar_alias.calendar_id(), &event)?;
            serialize_mutation_success(
                write.request_id,
                write.action,
                actor,
                write.calendar_alias,
                google_event,
            )
        }
        NormalizedRequest::Update(write) => {
            if !valid_optional_window(write.start.as_deref(), write.end.as_deref()) {
                return serialize_error(
                    write.request_id,
                    write.action.to_string(),
                    "INVALID_EVENT_INPUT",
                    INVALID_EVENT_INPUT,
                );
            }
            if write.title.is_none()
                && write.start.is_none()
                && write.end.is_none()
                && write.location.is_none()
                && write.notes.is_none()
            {
                return serialize_error(
                    write.request_id,
                    write.action.to_string(),
                    "INVALID_EVENT_INPUT",
                    INVALID_EVENT_INPUT,
                );
            }
            let Some(event_id) = event_id_from_ref(&write.event_ref) else {
                return serialize_error(
                    write.request_id,
                    write.action.to_string(),
                    "INVALID_EVENT_REF",
                    INVALID_EVENT_REF,
                );
            };
            let event = GoogleEventWrite {
                summary: clean_optional(write.title),
                start: write.start.map(write_time),
                end: write.end.map(write_time),
                location: clean_optional(write.location),
                description: clean_optional(write.notes),
            };
            let google_event =
                client.update_event(&write.calendar_alias.calendar_id(), &event_id, &event)?;
            serialize_mutation_success(
                write.request_id,
                write.action,
                actor,
                write.calendar_alias,
                google_event,
            )
        }
        NormalizedRequest::Delete(write) => {
            let Some(event_id) = event_id_from_ref(&write.event_ref) else {
                return serialize_error(
                    write.request_id,
                    write.action.to_string(),
                    "INVALID_EVENT_REF",
                    INVALID_EVENT_REF,
                );
            };
            client.delete_event(&write.calendar_alias.calendar_id(), &event_id)?;
            let success = CalendarDeleteSuccess {
                ok: true,
                request_id: write.request_id,
                action: write.action,
                actor,
                calendar_alias: write.calendar_alias.as_str(),
                event_ref: write.event_ref,
                deleted: true,
            };
            serde_json::to_string(&success).map_err(|err| err.to_string())
        }
    }
}

struct ReadRequest {
    request_id: Option<String>,
    action: &'static str,
    calendar_alias: CalendarAlias,
    time_min: String,
    time_max: String,
    query: Option<String>,
    max_results: Option<u32>,
}

struct CreateRequest {
    request_id: Option<String>,
    action: &'static str,
    calendar_alias: CalendarAlias,
    title: String,
    start: String,
    end: String,
    location: Option<String>,
    notes: Option<String>,
}

struct UpdateRequest {
    request_id: Option<String>,
    action: &'static str,
    calendar_alias: CalendarAlias,
    event_ref: String,
    title: Option<String>,
    start: Option<String>,
    end: Option<String>,
    location: Option<String>,
    notes: Option<String>,
}

struct DeleteRequest {
    request_id: Option<String>,
    action: &'static str,
    calendar_alias: CalendarAlias,
    event_ref: String,
}

enum NormalizedRequest {
    ListOrFind(ReadRequest),
    Create(CreateRequest),
    Update(UpdateRequest),
    Delete(DeleteRequest),
}

impl NormalizedRequest {
    fn request_id(&self) -> Option<String> {
        match self {
            Self::ListOrFind(request) => request.request_id.clone(),
            Self::Create(request) => request.request_id.clone(),
            Self::Update(request) => request.request_id.clone(),
            Self::Delete(request) => request.request_id.clone(),
        }
    }

    fn action(&self) -> &'static str {
        match self {
            Self::ListOrFind(request) => request.action,
            Self::Create(request) => request.action,
            Self::Update(request) => request.action,
            Self::Delete(request) => request.action,
        }
    }

    fn from_action(action: CalendarAction) -> Result<Self, String> {
        match action {
            CalendarAction::ListEvents {
                request_id,
                calendar_alias,
                time_min,
                time_max,
                max_results,
            } => Ok(Self::ListOrFind(ReadRequest {
                request_id,
                action: "calendar.events.list",
                calendar_alias,
                time_min,
                time_max,
                query: None,
                max_results,
            })),
            CalendarAction::FindEvents {
                request_id,
                calendar_alias,
                time_min,
                time_max,
                query,
                max_results,
            } => Ok(Self::ListOrFind(ReadRequest {
                request_id,
                action: "calendar.events.find",
                calendar_alias,
                time_min,
                time_max,
                query: Some(query),
                max_results,
            })),
            CalendarAction::CreateEvent {
                request_id,
                calendar_alias,
                title,
                start,
                end,
                location,
                notes,
            } => Ok(Self::Create(CreateRequest {
                request_id,
                action: "calendar.events.create",
                calendar_alias,
                title,
                start,
                end,
                location,
                notes,
            })),
            CalendarAction::UpdateEvent {
                request_id,
                calendar_alias,
                event_ref,
                title,
                start,
                end,
                location,
                notes,
            } => Ok(Self::Update(UpdateRequest {
                request_id,
                action: "calendar.events.update",
                calendar_alias,
                event_ref,
                title,
                start,
                end,
                location,
                notes,
            })),
            CalendarAction::DeleteEvent {
                request_id,
                calendar_alias,
                event_ref,
            } => Ok(Self::Delete(DeleteRequest {
                request_id,
                action: "calendar.events.delete",
                calendar_alias,
                event_ref,
            })),
        }
    }
}

const UNAUTHORIZED_ACTOR: &str = "This caller is not approved to access Simon calendar data.";
const UNSUPPORTED_ACTION: &str = "Only configured Simon calendar event actions are supported.";
const UNSUPPORTED_CALENDAR_ALIAS: &str =
    "calendarAlias must be one of the configured Simon calendar aliases.";
const INVALID_TIME_WINDOW: &str =
    "timeMin and timeMax must be RFC3339 timestamps and timeMax must be after timeMin.";
const INVALID_MAX_RESULTS: &str = "maxResults must be an integer from 1 to 20.";
const INVALID_EVENT_INPUT: &str =
    "Event writes require a title and valid RFC3339 start/end timestamps, or at least one valid update field.";
const INVALID_EVENT_REF: &str = "eventRef is not a valid Simon calendar event reference.";
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
    matches!(actor, Some("alon" | "default" | "local_ironclaw_bot"))
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

fn valid_optional_window(start: Option<&str>, end: Option<&str>) -> bool {
    match (start, end) {
        (Some(start), Some(end)) => valid_time_window(start, end),
        (Some(value), None) | (None, Some(value)) => value.contains('T'),
        (None, None) => true,
    }
}

fn normalized_max_results(value: Option<u32>) -> Option<u32> {
    let value = value.unwrap_or(DEFAULT_MAX_RESULTS);
    if (1..=MAX_RESULTS_LIMIT).contains(&value) {
        Some(value)
    } else {
        None
    }
}

fn write_time(date_time: String) -> GoogleEventWriteTime {
    GoogleEventWriteTime {
        date_time,
        time_zone: TIME_ZONE,
    }
}

fn clean_optional(value: Option<String>) -> Option<String> {
    value
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
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
    format!(
        "evt_{}_{}",
        calendar_alias.as_str(),
        hex_encode(raw_event_id.as_bytes())
    )
}

fn event_id_from_ref(event_ref: &str) -> Option<String> {
    let encoded = event_ref.strip_prefix("evt_family_")?;
    let bytes = hex_decode(encoded)?;
    String::from_utf8(bytes)
        .ok()
        .filter(|value| !value.is_empty())
}

fn hex_encode(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push(char::from(HEX[(byte >> 4) as usize]));
        out.push(char::from(HEX[(byte & 0x0F) as usize]));
    }
    out
}

fn hex_decode(value: &str) -> Option<Vec<u8>> {
    if value.len() % 2 != 0 {
        return None;
    }
    let mut bytes = Vec::with_capacity(value.len() / 2);
    let raw = value.as_bytes();
    for chunk in raw.chunks_exact(2) {
        let high = hex_value(chunk[0])?;
        let low = hex_value(chunk[1])?;
        bytes.push((high << 4) | low);
    }
    Some(bytes)
}

fn hex_value(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

fn serialize_mutation_success(
    request_id: Option<String>,
    action: &'static str,
    actor: String,
    calendar_alias: CalendarAlias,
    event: GoogleEvent,
) -> Result<String, String> {
    let success = CalendarMutationSuccess {
        ok: true,
        request_id,
        action,
        actor,
        calendar_alias: calendar_alias.as_str(),
        time_zone: TIME_ZONE,
        event: shape_event(calendar_alias, event),
    };
    serde_json::to_string(&success).map_err(|err| err.to_string())
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
    fn hosted_gateway_owner_is_allowed() {
        assert!(is_allowed_actor(Some("default")));
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
    fn event_ref_round_trips_without_plain_raw_id() {
        let raw_id = "family_event@example";
        let event_ref = safe_event_ref(CalendarAlias::Family, raw_id);
        assert!(!event_ref.contains(raw_id));
        assert_eq!(event_id_from_ref(&event_ref).as_deref(), Some(raw_id));
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
    fn schema_exposes_read_and_write_actions() {
        let schema = serde_json::to_string(&schemars::schema_for!(CalendarAction)).unwrap();
        assert!(schema.contains("calendar.events.list"));
        assert!(schema.contains("calendar.events.find"));
        assert!(schema.contains("calendar.events.create"));
        assert!(schema.contains("calendar.events.update"));
        assert!(schema.contains("calendar.events.delete"));
    }
}
