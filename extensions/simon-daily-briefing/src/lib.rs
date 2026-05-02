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
const CALENDAR_API_BASE: &str = "https://www.googleapis.com/calendar/v3";
const DEFAULT_MAX_RESULTS: u32 = 50;

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
        "Simon's deterministic daily briefing tool. Use this for proactive family schedule \
         briefings after simon_google_calendar is configured. It reads the Family calendar, \
         groups all-day and timed events for one local day, and returns the final Telegram-ready \
         message text plus structured summary fields. This tool is read-only."
            .to_string()
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct BriefingParams {
    action: String,
    #[serde(default, rename = "requestId")]
    request_id: Option<String>,
    #[serde(default)]
    date: Option<String>,
    timezone: String,
    #[serde(rename = "calendarAlias")]
    calendar_alias: CalendarAlias,
    #[serde(rename = "recipientIdentity")]
    recipient_identity: RecipientIdentity,
    #[serde(default)]
    language: Option<Language>,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
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

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
enum Language {
    #[default]
    En,
    He,
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

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
struct BriefingEvent {
    title: String,
    start: String,
    end: String,
    all_day: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    location: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct BriefingSuccess {
    ok: bool,
    request_id: Option<String>,
    action: &'static str,
    recipient_identity: &'static str,
    calendar_alias: &'static str,
    timezone: String,
    date: String,
    window_start: String,
    window_end: String,
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
    date: NaiveDate,
    timezone: Tz,
    window_start: String,
    window_end: String,
}

trait BriefingRuntime {
    fn now_millis(&self) -> u64;
    fn secret_exists(&self, secret_name: &str) -> bool;
    fn family_calendar_id(&self) -> Option<String>;
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

    if action_name != "generate_daily_briefing" {
        return serialize_error(
            request_id,
            action_name,
            "UNSUPPORTED_ACTION",
            UNSUPPORTED_ACTION,
        );
    }

    let request: BriefingParams = match serde_json::from_value(raw) {
        Ok(value) => value,
        Err(_) => {
            return serialize_error(
                request_id,
                action_name,
                "INVALID_PARAMETERS",
                INVALID_PARAMETERS,
            );
        }
    };

    if request.action != "generate_daily_briefing" {
        return serialize_error(
            request.request_id,
            request.action,
            "UNSUPPORTED_ACTION",
            UNSUPPORTED_ACTION,
        );
    }

    if request.calendar_alias != CalendarAlias::Family {
        return serialize_error(
            request.request_id,
            request.action,
            "UNSUPPORTED_CALENDAR_ALIAS",
            UNSUPPORTED_CALENDAR_ALIAS,
        );
    }

    if request.timezone != DEFAULT_TIME_ZONE {
        return serialize_error(
            request.request_id,
            request.action,
            "UNSUPPORTED_TIMEZONE",
            UNSUPPORTED_TIMEZONE,
        );
    }

    let date = match resolve_requested_date(request.date.as_deref(), &request.timezone, &runtime) {
        Some(date) => date,
        None => {
            return serialize_error(
                request.request_id,
                request.action,
                "INVALID_DATE",
                INVALID_DATE,
            )
        }
    };

    let Some(window) = compute_day_window(&date, &request.timezone) else {
        return serialize_error(
            request.request_id,
            request.action,
            "INVALID_DATE",
            INVALID_DATE,
        );
    };

    if !runtime.secret_exists(OAUTH_TOKEN_SECRET) {
        return serialize_error(
            request.request_id,
            request.action,
            "AUTH_REQUIRED",
            AUTH_REQUIRED,
        );
    }

    let Some(calendar_id) = runtime.family_calendar_id() else {
        return serialize_error(
            request.request_id,
            request.action,
            "CALENDAR_ALIAS_NOT_CONFIGURED",
            CALENDAR_ALIAS_NOT_CONFIGURED,
        );
    };

    let google_events = match runtime.list_events(
        &calendar_id,
        &window.window_start,
        &window.window_end,
        &request.timezone,
    ) {
        Ok(events) => events,
        Err(_) => {
            return serialize_error(
                request.request_id,
                request.action,
                "CALENDAR_LOOKUP_FAILED",
                CALENDAR_LOOKUP_FAILED,
            );
        }
    };

    let (all_day_events, timed_events) = shape_and_group_events(google_events, window.timezone);
    let language = request.language.unwrap_or(Language::He);
    let message_text = build_message(
        language,
        request.calendar_alias,
        window.date,
        &all_day_events,
        &timed_events,
        window.timezone,
    );

    let event_count = all_day_events.len() + timed_events.len();
    let success = BriefingSuccess {
        ok: true,
        request_id: request.request_id,
        action: "generate_daily_briefing",
        recipient_identity: request.recipient_identity.as_str(),
        calendar_alias: request.calendar_alias.as_str(),
        timezone: request.timezone,
        date,
        window_start: window.window_start,
        window_end: window.window_end,
        event_count,
        all_day_events,
        timed_events,
        message_text,
    };

    serde_json::to_string(&success).map_err(|err| err.to_string())
}

const INVALID_JSON: &str = "Daily Briefing received invalid JSON parameters.";
const INVALID_PARAMETERS: &str =
    "Daily Briefing requires timezone, calendarAlias, and recipientIdentity. date is optional.";
const UNSUPPORTED_ACTION: &str = "Daily Briefing supports only the generate_daily_briefing action.";
const UNSUPPORTED_CALENDAR_ALIAS: &str =
    "calendarAlias must be one of Simon's configured Daily Briefing aliases.";
const UNSUPPORTED_TIMEZONE: &str = "Daily Briefing currently supports only Asia/Jerusalem.";
const INVALID_DATE: &str = "date must be a valid local calendar day in YYYY-MM-DD format.";
const AUTH_REQUIRED: &str = "Simon Google Calendar OAuth is not configured for Daily Briefing.";
const CALENDAR_ALIAS_NOT_CONFIGURED: &str =
    "The Family calendar alias is not configured for Simon Daily Briefing.";
const CALENDAR_LOOKUP_FAILED: &str = "Daily Briefing could not read the Family calendar right now.";

fn compute_day_window(date: &str, timezone: &str) -> Option<DayWindow> {
    let date = NaiveDate::parse_from_str(date, "%Y-%m-%d").ok()?;
    let timezone: Tz = timezone.parse().ok()?;
    let start = local_midnight(timezone, date)?;
    let end = local_midnight(timezone, date.succ_opt()?)?;
    Some(DayWindow {
        date,
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
    _calendar_alias: CalendarAlias,
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
            Language::He => "מתוזמן".to_string(),
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
        Some(location) => format!("{} ({})", base, location),
        None => base,
    }
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
  "description": "Generate Simon's deterministic daily family briefing for one local day and one parent identity.",
  "properties": {
    "action": {
      "type": "string",
      "const": "generate_daily_briefing"
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
      "description": "IANA timezone. V1 supports Asia/Jerusalem."
    },
    "calendarAlias": {
      "type": "string",
      "enum": ["family"],
      "description": "Configured Simon calendar alias."
    },
    "recipientIdentity": {
      "type": "string",
      "enum": ["alon", "shlomit"],
      "description": "Canonical parent identity that this briefing is being prepared for."
    },
    "language": {
      "type": "string",
      "enum": ["en", "he"],
      "description": "Optional message language for static headings. Defaults to he."
    }
  },
  "required": ["action", "timezone", "calendarAlias", "recipientIdentity"],
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

        fn list_events(
            &self,
            _calendar_id: &str,
            _time_min: &str,
            _time_max: &str,
            _timezone: &str,
        ) -> Result<Vec<GoogleEvent>, String> {
            match &self.error {
                Some(error) => Err(error.clone()),
                None => Ok(self.events.clone()),
            }
        }
    }

    fn request_json(date: &str) -> String {
        format!(
            r#"{{
              "action":"generate_daily_briefing",
              "requestId":"req-1",
              "date":"{}",
              "timezone":"Asia/Jerusalem",
              "calendarAlias":"family",
              "recipientIdentity":"alon",
              "language":"en"
            }}"#,
            date
        )
    }

    fn hebrew_request_json(date: &str) -> String {
        format!(
            r#"{{
              "action":"generate_daily_briefing",
              "date":"{}",
              "timezone":"Asia/Jerusalem",
              "calendarAlias":"family",
              "recipientIdentity":"shlomit",
              "language":"he"
            }}"#,
            date
        )
    }

    fn request_json_without_date() -> String {
        r#"{
          "action":"generate_daily_briefing",
          "requestId":"req-1",
          "timezone":"Asia/Jerusalem",
          "calendarAlias":"family",
          "recipientIdentity":"alon"
        }"#
        .to_string()
    }

    fn timed_event(title: &str, start: &str, end: &str, location: Option<&str>) -> GoogleEvent {
        GoogleEvent {
            summary: Some(title.to_string()),
            start: GoogleEventTime {
                date: None,
                date_time: Some(start.to_string()),
            },
            end: GoogleEventTime {
                date: None,
                date_time: Some(end.to_string()),
            },
            location: location.map(str::to_string),
        }
    }

    fn all_day_event(title: &str, start: &str, end: &str) -> GoogleEvent {
        GoogleEvent {
            summary: Some(title.to_string()),
            start: GoogleEventTime {
                date: Some(start.to_string()),
                date_time: None,
            },
            end: GoogleEventTime {
                date: Some(end.to_string()),
                date_time: None,
            },
            location: None,
        }
    }

    fn parse_output(output: &str) -> Value {
        serde_json::from_str(output).expect("valid json output")
    }

    const MAY_2_2026_UTC_MILLIS: u64 = 1_777_675_200_000;

    #[test]
    fn empty_day_returns_compact_message() {
        let runtime = MockRuntime {
            now_millis: MAY_2_2026_UTC_MILLIS,
            secret_exists: true,
            family_calendar_id: Some("family-calendar-id".to_string()),
            events: Vec::new(),
            error: None,
        };

        let output = execute_inner(&request_json("2026-05-02"), runtime).unwrap();
        let parsed = parse_output(&output);

        assert_eq!(parsed["ok"], true);
        assert_eq!(parsed["eventCount"], 0);
        assert!(parsed["messageText"]
            .as_str()
            .unwrap()
            .contains("Nothing is scheduled on the family calendar today."));
    }

    #[test]
    fn all_day_events_are_grouped_separately() {
        let runtime = MockRuntime {
            now_millis: MAY_2_2026_UTC_MILLIS,
            secret_exists: true,
            family_calendar_id: Some("family-calendar-id".to_string()),
            events: vec![all_day_event("School vacation", "2026-05-02", "2026-05-03")],
            error: None,
        };

        let output = execute_inner(&request_json("2026-05-02"), runtime).unwrap();
        let parsed = parse_output(&output);

        assert_eq!(parsed["allDayEvents"].as_array().unwrap().len(), 1);
        assert_eq!(parsed["timedEvents"].as_array().unwrap().len(), 0);
        assert!(parsed["messageText"].as_str().unwrap().contains("All day"));
        assert!(parsed["messageText"]
            .as_str()
            .unwrap()
            .contains("School vacation"));
    }

    #[test]
    fn mixed_and_overlapping_events_are_sorted_and_grouped() {
        let runtime = MockRuntime {
            now_millis: MAY_2_2026_UTC_MILLIS,
            secret_exists: true,
            family_calendar_id: Some("family-calendar-id".to_string()),
            events: vec![
                timed_event(
                    "Pickup",
                    "2026-05-02T15:00:00+03:00",
                    "2026-05-02T15:30:00+03:00",
                    None,
                ),
                all_day_event("Birthday", "2026-05-02", "2026-05-03"),
                timed_event(
                    "Doctor",
                    "2026-05-02T09:00:00+03:00",
                    "2026-05-02T09:30:00+03:00",
                    Some("Clinic"),
                ),
                timed_event(
                    "Overlap",
                    "2026-05-02T09:15:00+03:00",
                    "2026-05-02T10:00:00+03:00",
                    None,
                ),
            ],
            error: None,
        };

        let output = execute_inner(&request_json("2026-05-02"), runtime).unwrap();
        let parsed = parse_output(&output);
        let timed = parsed["timedEvents"].as_array().unwrap();

        assert_eq!(parsed["allDayEvents"].as_array().unwrap().len(), 1);
        assert_eq!(timed.len(), 3);
        assert_eq!(timed[0]["title"], "Doctor");
        assert_eq!(timed[1]["title"], "Overlap");
        assert_eq!(timed[2]["title"], "Pickup");
        assert!(parsed["messageText"]
            .as_str()
            .unwrap()
            .contains("09:00-09:30 Doctor (Clinic)"));
    }

    #[test]
    fn hebrew_and_english_titles_pass_through() {
        let runtime = MockRuntime {
            now_millis: MAY_2_2026_UTC_MILLIS,
            secret_exists: true,
            family_calendar_id: Some("family-calendar-id".to_string()),
            events: vec![
                timed_event(
                    "רופא ילדים",
                    "2026-05-02T08:30:00+03:00",
                    "2026-05-02T09:00:00+03:00",
                    None,
                ),
                timed_event(
                    "Playdate",
                    "2026-05-02T16:00:00+03:00",
                    "2026-05-02T17:00:00+03:00",
                    None,
                ),
            ],
            error: None,
        };

        let output = execute_inner(&hebrew_request_json("2026-05-02"), runtime).unwrap();
        let message = parse_output(&output)["messageText"]
            .as_str()
            .unwrap()
            .to_string();

        assert!(message.contains("רופא ילדים"));
        assert!(message.contains("Playdate"));
        assert!(message.contains("תדריך משפחתי"));
    }

    #[test]
    fn asia_jerusalem_day_window_uses_local_offset() {
        let may = compute_day_window("2026-05-02", "Asia/Jerusalem").unwrap();
        let december = compute_day_window("2026-12-15", "Asia/Jerusalem").unwrap();

        assert!(may.window_start.ends_with("+03:00"));
        assert!(may.window_end.ends_with("+03:00"));
        assert!(december.window_start.ends_with("+02:00"));
        assert!(december.window_end.ends_with("+02:00"));
    }

    #[test]
    fn missing_auth_is_reported_without_host_error() {
        let runtime = MockRuntime {
            now_millis: MAY_2_2026_UTC_MILLIS,
            secret_exists: false,
            family_calendar_id: Some("family-calendar-id".to_string()),
            events: Vec::new(),
            error: None,
        };

        let output = execute_inner(&request_json("2026-05-02"), runtime).unwrap();
        let parsed = parse_output(&output);

        assert_eq!(parsed["ok"], false);
        assert_eq!(parsed["error"]["code"], "AUTH_REQUIRED");
    }

    #[test]
    fn missing_family_alias_is_reported() {
        let runtime = MockRuntime {
            now_millis: MAY_2_2026_UTC_MILLIS,
            secret_exists: true,
            family_calendar_id: None,
            events: Vec::new(),
            error: None,
        };

        let output = execute_inner(&request_json("2026-05-02"), runtime).unwrap();
        let parsed = parse_output(&output);

        assert_eq!(parsed["ok"], false);
        assert_eq!(parsed["error"]["code"], "CALENDAR_ALIAS_NOT_CONFIGURED");
    }

    #[test]
    fn calendar_lookup_errors_are_redacted() {
        let runtime = MockRuntime {
            now_millis: MAY_2_2026_UTC_MILLIS,
            secret_exists: true,
            family_calendar_id: Some("family-calendar-id".to_string()),
            events: Vec::new(),
            error: Some("raw google payload with private id abc123".to_string()),
        };

        let output = execute_inner(&request_json("2026-05-02"), runtime).unwrap();
        let parsed = parse_output(&output);
        let serialized = parsed.to_string();

        assert_eq!(parsed["ok"], false);
        assert_eq!(parsed["error"]["code"], "CALENDAR_LOOKUP_FAILED");
        assert!(!serialized.contains("abc123"));
        assert!(!serialized.contains("raw google payload"));
    }

    #[test]
    fn missing_date_defaults_to_local_today_in_jerusalem_and_hebrew() {
        let runtime = MockRuntime {
            now_millis: MAY_2_2026_UTC_MILLIS,
            secret_exists: true,
            family_calendar_id: Some("family-calendar-id".to_string()),
            events: Vec::new(),
            error: None,
        };

        let output = execute_inner(&request_json_without_date(), runtime).unwrap();
        let parsed = parse_output(&output);

        assert_eq!(parsed["ok"], true);
        assert_eq!(parsed["date"], "2026-05-02");
        assert!(parsed["messageText"]
            .as_str()
            .unwrap()
            .contains("תדריך משפחתי"));
    }
}
