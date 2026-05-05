use chrono::{DateTime, Datelike, LocalResult, NaiveDate, TimeZone, Utc, Weekday};
use chrono_tz::Tz;
use serde::{Deserialize, Serialize};
use serde_json::Value;

wit_bindgen::generate!({
    world: "sandboxed-tool",
    path: "wit/tool.wit",
});

const TOOL_NAME: &str = "simon_daily_briefing";
const OAUTH_TOKEN_SECRET: &str = "simon_google_calendar_oauth_token";
const DEFAULT_TIME_ZONE: &str = "Asia/Jerusalem";
const FAMILY_CALENDAR_ID_PATH: &str = ".system/simon_google_calendar/family_calendar_id";
const FAMILY_REGISTRY_PATH: &str =
    "channels/simon_telegram_channel/state/simon_family_profiles.json";
const CALENDAR_API_BASE: &str = "https://www.googleapis.com/calendar/v3";
const DEFAULT_MAX_RESULTS: u32 = 50;

const ACTION_GENERATE_DAILY_BRIEFING: &str = "generate_daily_briefing";
const ACTION_GENERATE_FAMILY_FACTS: &str = "generate_family_briefing_facts";
const ACTION_RENDER_DAILY_BRIEFING: &str = "render_daily_briefing";

struct SimonDailyBriefingTool;

impl exports::near::agent::tool::Guest for SimonDailyBriefingTool {
    fn execute(req: exports::near::agent::tool::Request) -> exports::near::agent::tool::Response {
        match execute_inner(&req.params, HostRuntime) {
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
        "Simon's deterministic daily briefing tool. It can generate shared family schedule facts \
         for one local day, then render recipient-specific briefing messages for canonical Simon \
         identities such as alon and shlomit. It is read-only and never writes to calendars."
            .to_string()
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
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
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum RecipientIdentity {
    Alon,
    Shlomit,
}

impl RecipientIdentity {
    fn as_str(self) -> &'static str {
        match self {
            RecipientIdentity::Alon => "alon",
            RecipientIdentity::Shlomit => "shlomit",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
enum Language {
    #[default]
    En,
    He,
}

impl Language {
    fn as_str(self) -> &'static str {
        match self {
            Language::En => "en",
            Language::He => "he",
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct FactsRequest {
    #[serde(rename = "action")]
    _action: String,
    #[serde(default, rename = "requestId")]
    request_id: Option<String>,
    #[serde(default)]
    date: Option<String>,
    #[serde(default)]
    timezone: Option<String>,
    #[serde(rename = "calendarAlias")]
    calendar_alias: CalendarAlias,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct GenerateDailyBriefingRequest {
    #[serde(rename = "action")]
    _action: String,
    #[serde(default, rename = "requestId")]
    request_id: Option<String>,
    #[serde(default)]
    date: Option<String>,
    #[serde(default)]
    timezone: Option<String>,
    #[serde(rename = "calendarAlias")]
    calendar_alias: CalendarAlias,
    #[serde(rename = "recipientIdentity")]
    recipient_identity: RecipientIdentity,
    #[serde(default)]
    language: Option<Language>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct RenderRequest {
    #[serde(rename = "action")]
    _action: String,
    #[serde(default, rename = "requestId")]
    request_id: Option<String>,
    #[serde(rename = "recipientIdentity")]
    recipient_identity: RecipientIdentity,
    #[serde(default)]
    language: Option<Language>,
    facts: BriefingFactsPayload,
}

#[derive(Clone, Debug, Deserialize)]
struct GoogleEventsResponse {
    #[serde(default)]
    items: Vec<GoogleEvent>,
}

#[derive(Clone, Debug, Deserialize)]
struct GoogleEvent {
    #[serde(default)]
    summary: Option<String>,
    #[serde(default)]
    start: GoogleEventTime,
    #[serde(default)]
    end: GoogleEventTime,
    #[serde(default)]
    location: Option<String>,
}

#[derive(Clone, Default, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GoogleEventTime {
    date: Option<String>,
    date_time: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
struct BriefingEvent {
    title: String,
    start: String,
    end: String,
    all_day: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    location: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
struct BriefingFactsPayload {
    calendar_alias: CalendarAlias,
    timezone: String,
    date: String,
    window_start: String,
    window_end: String,
    all_day_events: Vec<BriefingEvent>,
    timed_events: Vec<BriefingEvent>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct BriefingFactsSuccess {
    ok: bool,
    request_id: Option<String>,
    action: &'static str,
    calendar_alias: &'static str,
    timezone: String,
    date: String,
    window_start: String,
    window_end: String,
    event_count: usize,
    all_day_events: Vec<BriefingEvent>,
    timed_events: Vec<BriefingEvent>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct RenderedBriefingSuccess {
    ok: bool,
    request_id: Option<String>,
    action: String,
    recipient_identity: &'static str,
    recipient_display_name: String,
    recipient_status: String,
    calendar_alias: &'static str,
    timezone: String,
    language: &'static str,
    date: String,
    event_count: usize,
    all_day_events: Vec<BriefingEvent>,
    timed_events: Vec<BriefingEvent>,
    message_text: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct BriefingError {
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

#[derive(Clone, Debug)]
struct DayWindow {
    timezone: Tz,
    window_start: String,
    window_end: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct FamilyRegistry {
    #[serde(default)]
    users: Vec<FamilyUserProfile>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct FamilyUserProfile {
    canonical_id: String,
    display_name: String,
    status: String,
    #[serde(default)]
    preferred_language: Option<String>,
    #[serde(default)]
    #[serde(rename = "timezone")]
    _timezone: Option<String>,
}

trait BriefingRuntime {
    fn now_millis(&self) -> u64;
    fn secret_exists(&self, secret_name: &str) -> bool;
    fn family_calendar_id(&self) -> Option<String>;
    fn family_registry_json(&self) -> Option<String>;
    fn list_events(
        &self,
        calendar_id: &str,
        time_min: &str,
        time_max: &str,
        timezone: &str,
    ) -> Result<Vec<GoogleEvent>, String>;
}

struct HostRuntime;

impl BriefingRuntime for HostRuntime {
    fn now_millis(&self) -> u64 {
        near::agent::host::now_millis()
    }

    fn secret_exists(&self, secret_name: &str) -> bool {
        near::agent::host::secret_exists(secret_name)
    }

    fn family_calendar_id(&self) -> Option<String> {
        near::agent::host::workspace_read(FAMILY_CALENDAR_ID_PATH)
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
    }

    fn family_registry_json(&self) -> Option<String> {
        near::agent::host::workspace_read(FAMILY_REGISTRY_PATH)
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
    }

    fn list_events(
        &self,
        calendar_id: &str,
        time_min: &str,
        time_max: &str,
        timezone: &str,
    ) -> Result<Vec<GoogleEvent>, String> {
        near::agent::host::log(
            near::agent::host::LogLevel::Info,
            &format!("{} GET /calendar/v3/calendars/<alias>/events", TOOL_NAME),
        );

        let url = format!(
            "{}/calendars/{}/events?maxResults={}&singleEvents=true&orderBy=startTime&timeMin={}&timeMax={}&timeZone={}",
            CALENDAR_API_BASE,
            url_encode(calendar_id),
            DEFAULT_MAX_RESULTS,
            url_encode(time_min),
            url_encode(time_max),
            url_encode(timezone),
        );

        let response = near::agent::host::http_request("GET", &url, "{}", None, Some(30_000))
            .map_err(|_| "HTTP_REQUEST_FAILED".to_string())?;

        if response.status < 200 || response.status >= 300 {
            near::agent::host::log(
                near::agent::host::LogLevel::Warn,
                &format!(
                    "{} calendar lookup returned HTTP {}",
                    TOOL_NAME, response.status
                ),
            );
            return Err("GOOGLE_HTTP_ERROR".to_string());
        }

        let body =
            String::from_utf8(response.body).map_err(|_| "INVALID_GOOGLE_RESPONSE".to_string())?;
        let parsed: GoogleEventsResponse =
            serde_json::from_str(&body).map_err(|_| "INVALID_GOOGLE_RESPONSE".to_string())?;
        Ok(parsed.items)
    }
}

fn execute_inner<R: BriefingRuntime>(params: &str, runtime: R) -> Result<String, String> {
    let raw: Value = match serde_json::from_str(params) {
        Ok(value) => value,
        Err(_) => return serialize_error(None, String::new(), "INVALID_JSON", INVALID_JSON),
    };

    let action_name = raw_action_name(&raw);
    let request_id = raw_request_id(&raw);

    match action_name.as_str() {
        ACTION_GENERATE_DAILY_BRIEFING => {
            let request: GenerateDailyBriefingRequest = match serde_json::from_value(raw) {
                Ok(value) => value,
                Err(_) => {
                    return serialize_error(
                        request_id,
                        action_name,
                        "INVALID_PARAMETERS",
                        INVALID_PARAMETERS,
                    )
                }
            };

            let facts = match generate_facts(
                request.request_id.clone(),
                ACTION_GENERATE_DAILY_BRIEFING.to_string(),
                request.date.as_deref(),
                request.timezone.as_deref(),
                request.calendar_alias,
                &runtime,
            ) {
                Ok(facts) => facts,
                Err(json) => return Ok(json),
            };
            render_success(
                request.request_id,
                ACTION_GENERATE_DAILY_BRIEFING.to_string(),
                request.recipient_identity,
                request.language,
                facts,
                &runtime,
            )
        }
        ACTION_GENERATE_FAMILY_FACTS => {
            let request: FactsRequest = match serde_json::from_value(raw) {
                Ok(value) => value,
                Err(_) => {
                    return serialize_error(
                        request_id,
                        action_name,
                        "INVALID_PARAMETERS",
                        FACTS_INVALID_PARAMETERS,
                    )
                }
            };

            let facts = match generate_facts(
                request.request_id.clone(),
                ACTION_GENERATE_FAMILY_FACTS.to_string(),
                request.date.as_deref(),
                request.timezone.as_deref(),
                request.calendar_alias,
                &runtime,
            ) {
                Ok(facts) => facts,
                Err(json) => return Ok(json),
            };

            serde_json::to_string(&facts).map_err(|err| err.to_string())
        }
        ACTION_RENDER_DAILY_BRIEFING => {
            let request: RenderRequest = match serde_json::from_value(raw) {
                Ok(value) => value,
                Err(_) => {
                    return serialize_error(
                        request_id,
                        action_name,
                        "INVALID_PARAMETERS",
                        RENDER_INVALID_PARAMETERS,
                    )
                }
            };

            render_success(
                request.request_id,
                ACTION_RENDER_DAILY_BRIEFING.to_string(),
                request.recipient_identity,
                request.language,
                facts_payload_to_success(request.facts),
                &runtime,
            )
        }
        _ => serialize_error(
            request_id,
            action_name,
            "UNSUPPORTED_ACTION",
            UNSUPPORTED_ACTION,
        ),
    }
}

const INVALID_JSON: &str = "Daily Briefing received invalid JSON parameters.";
const INVALID_PARAMETERS: &str =
    "generate_daily_briefing requires calendarAlias and recipientIdentity. date/timezone/language are optional.";
const FACTS_INVALID_PARAMETERS: &str =
    "generate_family_briefing_facts requires calendarAlias. date and timezone are optional.";
const RENDER_INVALID_PARAMETERS: &str =
    "render_daily_briefing requires recipientIdentity and a facts payload.";
const UNSUPPORTED_ACTION: &str =
    "Daily Briefing supports generate_daily_briefing, generate_family_briefing_facts, and render_daily_briefing.";
const UNSUPPORTED_CALENDAR_ALIAS: &str =
    "calendarAlias must be one of Simon's configured Daily Briefing aliases.";
const INVALID_TIMEZONE: &str = "timezone must be a valid IANA timezone.";
const INVALID_DATE: &str = "date must be a valid local calendar day in YYYY-MM-DD format.";
const AUTH_REQUIRED: &str = "Simon Google Calendar OAuth is not configured for Daily Briefing.";
const CALENDAR_ALIAS_NOT_CONFIGURED: &str =
    "The Family calendar alias is not configured for Simon Daily Briefing.";
const CALENDAR_LOOKUP_FAILED: &str = "Daily Briefing could not read the Family calendar right now.";
const INVALID_RECIPIENT: &str =
    "recipientIdentity must be one of Simon's canonical parent identities.";

fn generate_facts<R: BriefingRuntime>(
    request_id: Option<String>,
    action: String,
    requested_date: Option<&str>,
    requested_timezone: Option<&str>,
    calendar_alias: CalendarAlias,
    runtime: &R,
) -> Result<BriefingFactsSuccess, String> {
    if calendar_alias != CalendarAlias::Family {
        return Err(error_json(
            request_id,
            action,
            "UNSUPPORTED_CALENDAR_ALIAS",
            UNSUPPORTED_CALENDAR_ALIAS,
        ));
    }

    let timezone = requested_timezone.unwrap_or(DEFAULT_TIME_ZONE).trim();
    if timezone.parse::<Tz>().is_err() {
        return Err(error_json(
            request_id,
            action,
            "INVALID_TIMEZONE",
            INVALID_TIMEZONE,
        ));
    }

    let date = match resolve_requested_date(requested_date, timezone, runtime) {
        Some(date) => date,
        None => return Err(error_json(request_id, action, "INVALID_DATE", INVALID_DATE)),
    };

    let Some(window) = compute_day_window(&date, timezone) else {
        return Err(error_json(request_id, action, "INVALID_DATE", INVALID_DATE));
    };

    if !runtime.secret_exists(OAUTH_TOKEN_SECRET) {
        return Err(error_json(
            request_id,
            action,
            "AUTH_REQUIRED",
            AUTH_REQUIRED,
        ));
    }

    let Some(calendar_id) = runtime.family_calendar_id() else {
        return Err(error_json(
            request_id,
            action,
            "CALENDAR_ALIAS_NOT_CONFIGURED",
            CALENDAR_ALIAS_NOT_CONFIGURED,
        ));
    };

    let google_events = match runtime.list_events(
        &calendar_id,
        &window.window_start,
        &window.window_end,
        timezone,
    ) {
        Ok(events) => events,
        Err(_) => {
            return Err(error_json(
                request_id,
                action,
                "CALENDAR_LOOKUP_FAILED",
                CALENDAR_LOOKUP_FAILED,
            ))
        }
    };

    let (all_day_events, timed_events) = shape_and_group_events(google_events, window.timezone);
    let event_count = all_day_events.len() + timed_events.len();

    Ok(BriefingFactsSuccess {
        ok: true,
        request_id,
        action: ACTION_GENERATE_FAMILY_FACTS,
        calendar_alias: calendar_alias.as_str(),
        timezone: timezone.to_string(),
        date,
        window_start: window.window_start,
        window_end: window.window_end,
        event_count,
        all_day_events,
        timed_events,
    })
}

fn render_success<R: BriefingRuntime>(
    request_id: Option<String>,
    action: String,
    recipient_identity: RecipientIdentity,
    requested_language: Option<Language>,
    facts: BriefingFactsSuccess,
    runtime: &R,
) -> Result<String, String> {
    let profile = load_recipient_profile(runtime, recipient_identity).ok_or_else(|| {
        serialize_error(
            request_id.clone(),
            action.clone(),
            "INVALID_RECIPIENT",
            INVALID_RECIPIENT,
        )
        .unwrap_err_or_json()
    })?;

    let timezone: Tz = facts
        .timezone
        .parse()
        .map_err(|_| "facts timezone must be a valid IANA timezone".to_string())?;
    let date = NaiveDate::parse_from_str(&facts.date, "%Y-%m-%d")
        .map_err(|_| "facts date invalid".to_string())?;
    let language = requested_language
        .or_else(|| profile.preferred_language)
        .unwrap_or(Language::He);

    let message_text = build_message(
        language,
        date,
        &facts.all_day_events,
        &facts.timed_events,
        timezone,
    );

    let rendered = RenderedBriefingSuccess {
        ok: true,
        request_id,
        action,
        recipient_identity: recipient_identity.as_str(),
        recipient_display_name: profile.display_name,
        recipient_status: profile.status,
        calendar_alias: facts.calendar_alias,
        timezone: facts.timezone,
        language: language.as_str(),
        date: facts.date,
        event_count: facts.event_count,
        all_day_events: facts.all_day_events,
        timed_events: facts.timed_events,
        message_text,
    };

    serde_json::to_string(&rendered).map_err(|err| err.to_string())
}

trait JsonResultExt {
    fn unwrap_err_or_json(self) -> String;
}

impl JsonResultExt for Result<String, String> {
    fn unwrap_err_or_json(self) -> String {
        match self {
            Ok(value) => value,
            Err(err) => err,
        }
    }
}

fn error_json(
    request_id: Option<String>,
    action: String,
    code: &'static str,
    message: &'static str,
) -> String {
    match serialize_error(request_id, action, code, message) {
        Ok(json) => json,
        Err(err) => err,
    }
}

fn load_recipient_profile<R: BriefingRuntime>(
    runtime: &R,
    recipient_identity: RecipientIdentity,
) -> Option<ResolvedRecipientProfile> {
    let canonical_id = recipient_identity.as_str();

    if let Some(raw) = runtime.family_registry_json() {
        if let Ok(registry) = serde_json::from_str::<FamilyRegistry>(&raw) {
            if let Some(profile) = registry
                .users
                .into_iter()
                .find(|profile| profile.canonical_id == canonical_id)
            {
                return Some(ResolvedRecipientProfile {
                    display_name: profile.display_name,
                    status: profile.status,
                    preferred_language: parse_profile_language(
                        profile.preferred_language.as_deref(),
                    ),
                });
            }
        }
    }

    Some(default_recipient_profile(recipient_identity))
}

#[derive(Clone, Debug)]
struct ResolvedRecipientProfile {
    display_name: String,
    status: String,
    preferred_language: Option<Language>,
}

fn default_recipient_profile(recipient_identity: RecipientIdentity) -> ResolvedRecipientProfile {
    match recipient_identity {
        RecipientIdentity::Alon => ResolvedRecipientProfile {
            display_name: "Alon".to_string(),
            status: "active".to_string(),
            preferred_language: Some(Language::He),
        },
        RecipientIdentity::Shlomit => ResolvedRecipientProfile {
            display_name: "Shlomit".to_string(),
            status: "dormant".to_string(),
            preferred_language: Some(Language::He),
        },
    }
}

fn parse_profile_language(value: Option<&str>) -> Option<Language> {
    match value.unwrap_or_default().trim().to_lowercase().as_str() {
        "en" => Some(Language::En),
        "he" => Some(Language::He),
        _ => None,
    }
}

fn facts_payload_to_success(payload: BriefingFactsPayload) -> BriefingFactsSuccess {
    let event_count = payload.all_day_events.len() + payload.timed_events.len();
    BriefingFactsSuccess {
        ok: true,
        request_id: None,
        action: ACTION_GENERATE_FAMILY_FACTS,
        calendar_alias: payload.calendar_alias.as_str(),
        timezone: payload.timezone,
        date: payload.date,
        window_start: payload.window_start,
        window_end: payload.window_end,
        event_count,
        all_day_events: payload.all_day_events,
        timed_events: payload.timed_events,
    }
}

fn compute_day_window(date: &str, timezone: &str) -> Option<DayWindow> {
    let date = NaiveDate::parse_from_str(date, "%Y-%m-%d").ok()?;
    let timezone: Tz = timezone.parse().ok()?;
    let start = local_midnight(timezone, date)?;
    let end = local_midnight(timezone, date.succ_opt()?)?;
    Some(DayWindow {
        timezone,
        window_start: start.to_rfc3339(),
        window_end: end.to_rfc3339(),
    })
}

fn resolve_requested_date<R: BriefingRuntime>(
    requested_date: Option<&str>,
    timezone: &str,
    runtime: &R,
) -> Option<String> {
    match requested_date {
        Some(date) => {
            let trimmed = date.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_string())
            }
        }
        None => local_date_from_now(runtime.now_millis(), timezone),
    }
}

fn local_date_from_now(now_millis: u64, timezone: &str) -> Option<String> {
    let timezone: Tz = timezone.parse().ok()?;
    let now = Utc.timestamp_millis_opt(now_millis as i64).single()?;
    Some(now.with_timezone(&timezone).date_naive().to_string())
}

fn local_midnight(timezone: Tz, date: NaiveDate) -> Option<DateTime<Tz>> {
    match timezone.with_ymd_and_hms(date.year(), date.month(), date.day(), 0, 0, 0) {
        LocalResult::Single(value) => Some(value),
        LocalResult::Ambiguous(first, _) => Some(first),
        LocalResult::None => None,
    }
}

fn shape_and_group_events(
    google_events: Vec<GoogleEvent>,
    timezone: Tz,
) -> (Vec<BriefingEvent>, Vec<BriefingEvent>) {
    let mut all_day_events = Vec::new();
    let mut timed_events = Vec::new();

    for event in google_events {
        let shaped = shape_event(event);
        if shaped.all_day {
            all_day_events.push(shaped);
        } else {
            timed_events.push(shaped);
        }
    }

    all_day_events.sort_by(|left, right| {
        left.start
            .cmp(&right.start)
            .then_with(|| left.title.cmp(&right.title))
    });

    timed_events.sort_by(|left, right| {
        parsed_datetime(&left.start, timezone)
            .cmp(&parsed_datetime(&right.start, timezone))
            .then_with(|| left.title.cmp(&right.title))
    });

    (all_day_events, timed_events)
}

fn shape_event(event: GoogleEvent) -> BriefingEvent {
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

    BriefingEvent {
        title: non_empty(event.summary).unwrap_or_else(|| "(No title)".to_string()),
        start,
        end,
        all_day,
        location: non_empty(event.location),
    }
}

fn build_message(
    language: Language,
    date: NaiveDate,
    all_day_events: &[BriefingEvent],
    timed_events: &[BriefingEvent],
    timezone: Tz,
) -> String {
    let mut lines = vec![header_line(language, date)];

    if all_day_events.is_empty() && timed_events.is_empty() {
        lines.push(String::new());
        lines.push(match language {
            Language::En => "Nothing is scheduled on the family calendar today.".to_string(),
            Language::He => "אין אירועים בלוח המשפחתי להיום.".to_string(),
        });
        return lines.join("\n");
    }

    if !all_day_events.is_empty() {
        lines.push(String::new());
        lines.push(match language {
            Language::En => "All day".to_string(),
            Language::He => "כל היום".to_string(),
        });
        for event in all_day_events {
            lines.push(format_event_line(event, None));
        }
    }

    if !timed_events.is_empty() {
        lines.push(String::new());
        lines.push(match language {
            Language::En => "Scheduled".to_string(),
            Language::He => "ביומן היום".to_string(),
        });
        for event in timed_events {
            lines.push(format_event_line(
                event,
                Some(display_time_range(&event.start, &event.end, timezone)),
            ));
        }
    }

    lines.join("\n")
}

fn header_line(language: Language, date: NaiveDate) -> String {
    let weekday = weekday_label(language, date.weekday());
    match language {
        Language::En => format!(
            "Family Briefing · {} · {}",
            weekday,
            date.format("%Y-%m-%d")
        ),
        Language::He => format!("תדריך משפחתי · {} · {}", weekday, date.format("%Y-%m-%d")),
    }
}

fn weekday_label(language: Language, weekday: Weekday) -> &'static str {
    match (language, weekday) {
        (Language::En, Weekday::Mon) => "Monday",
        (Language::En, Weekday::Tue) => "Tuesday",
        (Language::En, Weekday::Wed) => "Wednesday",
        (Language::En, Weekday::Thu) => "Thursday",
        (Language::En, Weekday::Fri) => "Friday",
        (Language::En, Weekday::Sat) => "Saturday",
        (Language::En, Weekday::Sun) => "Sunday",
        (Language::He, Weekday::Mon) => "יום שני",
        (Language::He, Weekday::Tue) => "יום שלישי",
        (Language::He, Weekday::Wed) => "יום רביעי",
        (Language::He, Weekday::Thu) => "יום חמישי",
        (Language::He, Weekday::Fri) => "יום שישי",
        (Language::He, Weekday::Sat) => "שבת",
        (Language::He, Weekday::Sun) => "יום ראשון",
    }
}

fn format_event_line(event: &BriefingEvent, time_prefix: Option<String>) -> String {
    let base = match time_prefix {
        Some(prefix) => format!("- {} {}", prefix, event.title),
        None => format!("- {}", event.title),
    };

    match &event.location {
        Some(location) => format!("{} ({})", base, clean_location_for_message(location)),
        None => base,
    }
}

fn clean_location_for_message(location: &str) -> String {
    location.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn display_time_range(start: &str, end: &str, timezone: Tz) -> String {
    match (
        parsed_datetime(start, timezone),
        parsed_datetime(end, timezone),
    ) {
        (Some(start), Some(end)) => format!("{}-{}", start.format("%H:%M"), end.format("%H:%M")),
        _ => fallback_time_range(start, end),
    }
}

fn fallback_time_range(start: &str, end: &str) -> String {
    let start = fallback_clock_time(start);
    let end = fallback_clock_time(end);
    format!("{}-{}", start, end)
}

fn fallback_clock_time(value: &str) -> String {
    value
        .split('T')
        .nth(1)
        .and_then(|tail| tail.get(0..5))
        .unwrap_or(value)
        .to_string()
}

fn parsed_datetime(value: &str, timezone: Tz) -> Option<DateTime<Tz>> {
    DateTime::parse_from_rfc3339(value)
        .ok()
        .map(|value| value.with_timezone(&timezone))
}

fn non_empty(value: Option<String>) -> Option<String> {
    value
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn serialize_error(
    request_id: Option<String>,
    action: String,
    code: &'static str,
    message: &'static str,
) -> Result<String, String> {
    serde_json::to_string(&BriefingError {
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

const SCHEMA: &str = r#"{
  "type": "object",
  "description": "Generate Simon's shared family briefing facts or render a recipient-specific daily briefing message.",
  "properties": {
    "action": {
      "type": "string",
      "enum": [
        "generate_daily_briefing",
        "generate_family_briefing_facts",
        "render_daily_briefing"
      ]
    },
    "requestId": {
      "type": "string",
      "description": "Optional caller-generated request identifier."
    },
    "date": {
      "type": "string",
      "description": "Optional local family day in YYYY-MM-DD format. Defaults to today in the requested timezone."
    },
    "timezone": {
      "type": "string",
      "description": "Optional IANA timezone. Defaults to Asia/Jerusalem."
    },
    "calendarAlias": {
      "type": "string",
      "enum": ["family"]
    },
    "recipientIdentity": {
      "type": "string",
      "enum": ["alon", "shlomit"]
    },
    "language": {
      "type": "string",
      "enum": ["en", "he"]
    },
    "facts": {
      "type": "object",
      "description": "Shared facts payload returned by generate_family_briefing_facts."
    }
  },
  "required": ["action"],
  "additionalProperties": false
}"#;

#[cfg(target_arch = "wasm32")]
export!(SimonDailyBriefingTool);

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    struct MockRuntime {
        now_millis: u64,
        secret_exists: bool,
        family_calendar_id: Option<String>,
        family_registry_json: Option<String>,
        events: Vec<GoogleEvent>,
        error: Option<String>,
    }

    impl BriefingRuntime for MockRuntime {
        fn now_millis(&self) -> u64 {
            self.now_millis
        }

        fn secret_exists(&self, _secret_name: &str) -> bool {
            self.secret_exists
        }

        fn family_calendar_id(&self) -> Option<String> {
            self.family_calendar_id.clone()
        }

        fn family_registry_json(&self) -> Option<String> {
            self.family_registry_json.clone()
        }

        fn list_events(
            &self,
            _calendar_id: &str,
            _time_min: &str,
            _time_max: &str,
            _timezone: &str,
        ) -> Result<Vec<GoogleEvent>, String> {
            match &self.error {
                Some(err) => Err(err.clone()),
                None => Ok(self.events.clone()),
            }
        }
    }

    fn base_runtime() -> MockRuntime {
        MockRuntime {
            now_millis: 1_777_680_000_000,
            secret_exists: true,
            family_calendar_id: Some("family-calendar".to_string()),
            family_registry_json: None,
            events: vec![
                GoogleEvent {
                    summary: Some("Morning dropoff".to_string()),
                    start: GoogleEventTime {
                        date: None,
                        date_time: Some("2026-05-02T08:15:00+03:00".to_string()),
                    },
                    end: GoogleEventTime {
                        date: None,
                        date_time: Some("2026-05-02T09:05:00+03:00".to_string()),
                    },
                    location: Some("Givatayim".to_string()),
                },
                GoogleEvent {
                    summary: Some("Dentist".to_string()),
                    start: GoogleEventTime {
                        date: None,
                        date_time: Some("2026-05-02T12:40:00+03:00".to_string()),
                    },
                    end: GoogleEventTime {
                        date: None,
                        date_time: Some("2026-05-02T13:40:00+03:00".to_string()),
                    },
                    location: None,
                },
            ],
            error: None,
        }
    }

    fn decode_json(output: &str) -> Value {
        serde_json::from_str(output).expect("valid json output")
    }

    #[test]
    fn legacy_generate_daily_briefing_returns_message_and_events() {
        let output = execute_inner(
            r#"{
                "action":"generate_daily_briefing",
                "date":"2026-05-02",
                "timezone":"Asia/Jerusalem",
                "calendarAlias":"family",
                "recipientIdentity":"alon"
            }"#,
            base_runtime(),
        )
        .unwrap();

        let json = decode_json(&output);
        assert_eq!(json["ok"], true);
        assert_eq!(json["recipientIdentity"], "alon");
        assert_eq!(json["eventCount"], 2);
        assert!(json["messageText"]
            .as_str()
            .unwrap()
            .contains("תדריך משפחתי"));
        assert!(json["messageText"].as_str().unwrap().contains("ביומן היום"));
    }

    #[test]
    fn hebrew_message_collapses_multiline_locations() {
        let runtime = MockRuntime {
            events: vec![GoogleEvent {
                summary: Some("טיפול זוגי אצל איילת".to_string()),
                start: GoogleEventTime {
                    date: None,
                    date_time: Some("2026-05-05T12:00:00+03:00".to_string()),
                },
                end: GoogleEventTime {
                    date: None,
                    date_time: Some("2026-05-05T13:00:00+03:00".to_string()),
                },
                location: Some("Visozki K Z Street 4\nTel Aviv-Yafo, Israel".to_string()),
            }],
            ..base_runtime()
        };

        let output = execute_inner(
            r#"{
                "action":"generate_daily_briefing",
                "date":"2026-05-05",
                "timezone":"Asia/Jerusalem",
                "calendarAlias":"family",
                "recipientIdentity":"alon"
            }"#,
            runtime,
        )
        .unwrap();

        let message = decode_json(&output)["messageText"]
            .as_str()
            .unwrap()
            .to_string();
        assert!(message.contains("ביומן היום"));
        assert!(message.contains("Visozki K Z Street 4 Tel Aviv-Yafo, Israel"));
        assert!(!message.contains("4\nTel Aviv"));
    }

    #[test]
    fn generate_family_facts_returns_shared_payload_without_message() {
        let output = execute_inner(
            r#"{
                "action":"generate_family_briefing_facts",
                "date":"2026-05-02",
                "timezone":"Asia/Jerusalem",
                "calendarAlias":"family"
            }"#,
            base_runtime(),
        )
        .unwrap();

        let json = decode_json(&output);
        assert_eq!(json["ok"], true);
        assert_eq!(json["action"], "generate_family_briefing_facts");
        assert_eq!(json["eventCount"], 2);
        assert!(json.get("messageText").is_none());
    }

    #[test]
    fn render_daily_briefing_uses_profile_language_when_language_omitted() {
        let runtime = MockRuntime {
            family_registry_json: Some(
                r#"{
                    "users":[
                        {
                            "canonicalId":"alon",
                            "displayName":"Alon",
                            "status":"active",
                            "preferredLanguage":"en",
                            "timezone":"Asia/Jerusalem"
                        }
                    ]
                }"#
                .to_string(),
            ),
            ..base_runtime()
        };

        let output = execute_inner(
            r#"{
                "action":"render_daily_briefing",
                "recipientIdentity":"alon",
                "facts":{
                    "calendarAlias":"family",
                    "timezone":"Asia/Jerusalem",
                    "date":"2026-05-02",
                    "windowStart":"2026-05-01T21:00:00+00:00",
                    "windowEnd":"2026-05-02T21:00:00+00:00",
                    "allDayEvents":[],
                    "timedEvents":[
                        {
                            "title":"Morning dropoff",
                            "start":"2026-05-02T08:15:00+03:00",
                            "end":"2026-05-02T09:05:00+03:00",
                            "allDay":false
                        }
                    ]
                }
            }"#,
            runtime,
        )
        .unwrap();

        let json = decode_json(&output);
        assert_eq!(json["language"], "en");
        assert!(json["messageText"]
            .as_str()
            .unwrap()
            .contains("Family Briefing"));
    }

    #[test]
    fn omitted_date_defaults_from_runtime_clock() {
        let runtime = MockRuntime {
            now_millis: 1_777_853_200_000,
            ..base_runtime()
        };

        let output = execute_inner(
            r#"{
                "action":"generate_family_briefing_facts",
                "timezone":"Asia/Jerusalem",
                "calendarAlias":"family"
            }"#,
            runtime,
        )
        .unwrap();

        let json = decode_json(&output);
        assert_eq!(json["date"], "2026-05-04");
    }

    #[test]
    fn invalid_timezone_returns_error_json() {
        let output = execute_inner(
            r#"{
                "action":"generate_family_briefing_facts",
                "date":"2026-05-02",
                "timezone":"Mars/Base",
                "calendarAlias":"family"
            }"#,
            base_runtime(),
        )
        .unwrap();

        let json = decode_json(&output);
        assert_eq!(json["ok"], false);
        assert_eq!(json["error"]["code"], "INVALID_TIMEZONE");
    }
}
