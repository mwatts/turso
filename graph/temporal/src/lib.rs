//! Cypher temporal and duration values as a statically registered scalar
//! extension.
//!
//! Values follow the Cypher/GQL temporal model (shared with JS Temporal):
//! civil dates and times, zoned datetimes rendered as RFC 9557 text
//! (`2018-03-22T12:00+01:00[Europe/Berlin]`), and durations kept as the
//! four-component month/day/second/nanosecond vector that Cypher requires
//! (fields never cross: `P1DT25H` keeps 25 hours). The arithmetic is
//! implemented with `jiff`, whose bundled IANA tz database provides named
//! zones without a system dependency.

use std::sync::atomic::{AtomicUsize, Ordering};

use jiff::{
    civil,
    tz::{Offset, TimeZone},
    Span, Timestamp, Unit, Zoned,
};
use turso_core::Connection;
use turso_ext::{scalar, ExtensionApi, Value as ExtValue};

/// How many times [`install_temporal_extension`] has been invoked in this
/// process. Always available (not `cfg(test)`) so integration tests in
/// dependent crates can assert install policy (every `GraphConnection::install`
/// installs, including dialect-pinned, for InternalHelper symbol safety).
pub static INSTALL_COUNT: AtomicUsize = AtomicUsize::new(0);

/// Registers the temporal functions on a connection. Safe to call more
/// than once; later registrations replace the earlier entries.
pub fn install_temporal_extension(connection: &Connection) {
    INSTALL_COUNT.fetch_add(1, Ordering::SeqCst);
    connection.register_static_extension(|ext_api: &mut ExtensionApi| unsafe {
        let register = |name: *const std::ffi::c_char, function| {
            (ext_api.register_scalar_function)(
                ext_api.ctx,
                name,
                -1,
                false,
                0,
                function,
                None,
                None,
            );
        };
        register(c"duration_make".as_ptr(), duration_make);
        register(c"duration_parse".as_ptr(), duration_parse);
        register(c"duration_get".as_ptr(), duration_get);
        register(c"duration_add".as_ptr(), duration_add);
        register(c"duration_neg".as_ptr(), duration_neg);
        register(c"duration_between".as_ptr(), duration_between);
        register(c"temporal_make".as_ptr(), temporal_make);
        register(c"temporal_truncate".as_ptr(), temporal_truncate);
        register(c"temporal_parse".as_ptr(), temporal_parse);
        register(c"temporal_get".as_ptr(), temporal_get);
        register(c"temporal_now".as_ptr(), temporal_now);
        register(c"datetime_add_duration".as_ptr(), datetime_add_duration);
        register(c"datetime_sub_duration".as_ptr(), datetime_sub_duration);
        register(c"jsonb_get".as_ptr(), jsonb_get);
        register(c"jsonb_get_text".as_ptr(), jsonb_get_text);
        register(c"jsonb_get_path".as_ptr(), jsonb_get_path);
        register(c"jsonb_exists".as_ptr(), jsonb_exists);
        register(c"jsonb_exists_any".as_ptr(), jsonb_exists_any);
        register(c"jsonb_exists_all".as_ptr(), jsonb_exists_all);
        register(c"jsonb_contains".as_ptr(), jsonb_contains);
        register(c"cypher_raise".as_ptr(), cypher_raise);
        register(c"cypher_equals".as_ptr(), cypher_equals);
        register(c"cypher_add".as_ptr(), cypher_add);
        register(c"cypher_sub".as_ptr(), cypher_sub);
        register(c"cypher_concat".as_ptr(), cypher_concat);
        register(c"cypher_div".as_ptr(), cypher_div);
        register(c"split".as_ptr(), split);
    });
}

/// Every scalar name `install_temporal_extension` registers, in the same
/// order. `GraphDialect::resolve_function` treats this list as the
/// dialect-owned function surface.
pub const FUNCTION_NAMES: &[&str] = &[
    "duration_make",
    "duration_parse",
    "duration_get",
    "duration_add",
    "duration_neg",
    "duration_between",
    "temporal_make",
    "temporal_truncate",
    "temporal_parse",
    "temporal_get",
    "temporal_now",
    "datetime_add_duration",
    "datetime_sub_duration",
    "jsonb_get",
    "jsonb_get_text",
    "jsonb_get_path",
    "jsonb_exists",
    "jsonb_exists_any",
    "jsonb_exists_all",
    "jsonb_contains",
    "cypher_raise",
    "cypher_equals",
    "cypher_add",
    "cypher_sub",
    "cypher_concat",
    "cypher_div",
    "split",
];

/// Execute a temporal/cypher scalar by name outside the extension ABI.
/// Returns `None` for names this crate does not own.
pub fn dispatch(name: &str, args: &[ExtValue]) -> Option<ExtValue> {
    Some(match name {
        "duration_make" => duration_make_impl(args),
        "duration_parse" => duration_parse_impl(args),
        "duration_get" => duration_get_impl(args),
        "duration_add" => duration_add_impl(args),
        "duration_neg" => duration_neg_impl(args),
        "duration_between" => duration_between_impl(args),
        "temporal_make" => temporal_make_impl(args),
        "temporal_truncate" => temporal_truncate_impl(args),
        "temporal_parse" => temporal_parse_impl(args),
        "temporal_get" => temporal_get_impl(args),
        "temporal_now" => temporal_now_impl(args),
        "datetime_add_duration" => shift_datetime(args, 1),
        "datetime_sub_duration" => shift_datetime(args, -1),
        "jsonb_get" => jsonb_get_impl(args),
        "jsonb_get_text" => jsonb_get_text_impl(args),
        "jsonb_get_path" => jsonb_get_path_impl(args),
        "jsonb_exists" => jsonb_exists_impl(args),
        "jsonb_exists_any" => exists_over(args, false),
        "jsonb_exists_all" => exists_over(args, true),
        "jsonb_contains" => jsonb_contains_impl(args),
        "cypher_raise" => cypher_raise_impl(args),
        "cypher_equals" => cypher_equals_impl(args),
        "cypher_add" => cypher_add_impl(args),
        "cypher_sub" => cypher_sub_impl(args),
        "cypher_concat" => cypher_concat_impl(args),
        "cypher_div" => cypher_div_impl(args),
        "split" => split_impl(args),
        _ => return None,
    })
}

// ---------------------------------------------------------------------------
// Durations: four-component vectors encoded as canonical ISO-8601 text.
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct Dur {
    months: i64,
    days: i64,
    seconds: i64,
    nanos: i64,
}

impl Dur {
    fn normalized(mut self) -> Self {
        let extra = self.nanos.div_euclid(1_000_000_000);
        self.seconds += extra;
        self.nanos = self.nanos.rem_euclid(1_000_000_000);
        self
    }

    fn render(self) -> String {
        let normalized = self.normalized();
        let years = normalized.months / 12;
        let months = normalized.months % 12;
        let mut out = String::from("P");
        if years != 0 {
            out.push_str(&format!("{years}Y"));
        }
        if months != 0 {
            out.push_str(&format!("{months}M"));
        }
        if normalized.days != 0 {
            out.push_str(&format!("{}D", normalized.days));
        }
        if normalized.seconds != 0 || normalized.nanos != 0 || out.len() == 1 {
            out.push('T');
            let total_nanos =
                i128::from(normalized.seconds) * 1_000_000_000 + i128::from(normalized.nanos);
            let negative = total_nanos < 0;
            let magnitude = total_nanos.abs();
            let total_seconds = magnitude / 1_000_000_000;
            let nanos = magnitude % 1_000_000_000;
            let hours = total_seconds / 3600;
            let minutes = (total_seconds % 3600) / 60;
            let seconds = total_seconds % 60;
            let signed = |value: i128| {
                if negative {
                    format!("-{value}")
                } else {
                    value.to_string()
                }
            };
            if hours != 0 {
                out.push_str(&format!("{}H", signed(hours)));
            }
            if minutes != 0 {
                out.push_str(&format!("{}M", signed(minutes)));
            }
            if seconds != 0 || nanos != 0 || (hours == 0 && minutes == 0) {
                if nanos == 0 {
                    out.push_str(&format!("{}S", signed(seconds)));
                } else {
                    let fraction = format!("{nanos:09}");
                    out.push_str(&format!(
                        "{}.{}S",
                        signed(seconds),
                        fraction.trim_end_matches('0')
                    ));
                }
            }
        }
        out
    }

    fn parse(text: &str) -> Option<Self> {
        let text = text.trim();
        let rest = text.strip_prefix('P')?;
        let (date_part, time_part) = match rest.split_once('T') {
            Some((date, time)) => (date, Some(time)),
            None => (rest, None),
        };
        let mut value = Dur::default();
        let mut number = String::new();
        for character in date_part.chars() {
            if character.is_ascii_digit() || character == '-' || character == '+' {
                number.push(character);
                continue;
            }
            let quantity: i64 = number.parse().ok()?;
            number.clear();
            match character {
                'Y' => value.months += quantity * 12,
                'M' => value.months += quantity,
                'W' => value.days += quantity * 7,
                'D' => value.days += quantity,
                _ => return None,
            }
        }
        if !number.is_empty() {
            return None;
        }
        if let Some(time_part) = time_part {
            let mut number = String::new();
            for character in time_part.chars() {
                if character.is_ascii_digit() || matches!(character, '-' | '+' | '.') {
                    number.push(character);
                    continue;
                }
                match character {
                    'H' => value.seconds += number.parse::<i64>().ok()? * 3600,
                    'M' => value.seconds += number.parse::<i64>().ok()? * 60,
                    'S' => {
                        let (seconds, nanos) = parse_fractional_seconds(&number)?;
                        value.seconds += seconds;
                        value.nanos += nanos;
                    }
                    _ => return None,
                }
                number.clear();
            }
            if !number.is_empty() {
                return None;
            }
        }
        Some(value.normalized())
    }
}

fn parse_fractional_seconds(text: &str) -> Option<(i64, i64)> {
    match text.split_once('.') {
        None => Some((text.parse().ok()?, 0)),
        Some((whole, fraction)) => {
            let seconds: i64 = whole.parse().ok()?;
            let mut digits = fraction.to_owned();
            if digits.len() > 9 || digits.is_empty() {
                return None;
            }
            while digits.len() < 9 {
                digits.push('0');
            }
            let nanos: i64 = digits.parse().ok()?;
            Some((
                seconds,
                if whole.starts_with('-') {
                    -nanos
                } else {
                    nanos
                },
            ))
        }
    }
}

fn span_to_dur(span: Span) -> Dur {
    Dur {
        months: i64::from(span.get_years()) * 12 + i64::from(span.get_months()),
        days: i64::from(span.get_weeks()) * 7 + i64::from(span.get_days()),
        seconds: i64::from(span.get_hours()) * 3600 + span.get_minutes() * 60 + span.get_seconds(),
        nanos: span.get_milliseconds() * 1_000_000
            + span.get_microseconds() * 1_000
            + span.get_nanoseconds(),
    }
    .normalized()
}

// ---------------------------------------------------------------------------
// Temporal values: the five Cypher temporal kinds over jiff types.
// ---------------------------------------------------------------------------

#[derive(Clone, Debug)]
enum Temporal {
    Date(civil::Date),
    LocalTime(civil::Time),
    Time(civil::Time, Offset),
    LocalDateTime(civil::DateTime),
    DateTime(Zoned),
}

enum ZoneSuffix {
    Missing,
    Utc,
    Fixed(Offset),
}

/// Splits a trailing `Z` or `±HH:MM` offset from a temporal string. A date
/// like `2018-04-27` is safe: its final `-` is not followed by `dd:dd`.
fn strip_zone(text: &str) -> (&str, ZoneSuffix) {
    if let Some(rest) = text.strip_suffix('Z') {
        return (rest, ZoneSuffix::Utc);
    }
    if text.len() >= 6 {
        let tail = &text[text.len() - 6..];
        let bytes = tail.as_bytes();
        if (bytes[0] == b'+' || bytes[0] == b'-') && bytes[3] == b':' {
            if let (Ok(hours), Ok(minutes)) = (tail[1..3].parse::<i32>(), tail[4..6].parse::<i32>())
            {
                let sign = if bytes[0] == b'+' { 1 } else { -1 };
                if let Ok(offset) = Offset::from_seconds(sign * (hours * 3600 + minutes * 60)) {
                    return (&text[..text.len() - 6], ZoneSuffix::Fixed(offset));
                }
            }
        }
    }
    (text, ZoneSuffix::Missing)
}

fn zone_of(suffix: &ZoneSuffix) -> Option<TimeZone> {
    match suffix {
        ZoneSuffix::Missing => None,
        ZoneSuffix::Utc => Some(TimeZone::fixed(Offset::UTC)),
        ZoneSuffix::Fixed(offset) => Some(TimeZone::fixed(*offset)),
    }
}

fn parse_temporal(text: &str) -> Option<Temporal> {
    let text = text.trim();
    if text.contains('[') {
        return text.parse::<Zoned>().ok().map(Temporal::DateTime);
    }
    let (core, zone) = strip_zone(text);
    let has_date = core.len() >= 8 && core.as_bytes().get(4) == Some(&b'-');
    if has_date {
        if !core.contains('T') {
            let date = core.parse::<civil::Date>().ok()?;
            return Some(match zone_of(&zone) {
                None => Temporal::Date(date),
                Some(tz) => Temporal::DateTime(
                    date.to_datetime(civil::Time::midnight())
                        .to_zoned(tz)
                        .ok()?,
                ),
            });
        }
        let datetime = core.parse::<civil::DateTime>().ok()?;
        return Some(match zone_of(&zone) {
            None => Temporal::LocalDateTime(datetime),
            Some(tz) => Temporal::DateTime(datetime.to_zoned(tz).ok()?),
        });
    }
    let time = core.parse::<civil::Time>().ok()?;
    Some(match zone {
        ZoneSuffix::Missing => Temporal::LocalTime(time),
        ZoneSuffix::Utc => Temporal::Time(time, Offset::UTC),
        ZoneSuffix::Fixed(offset) => Temporal::Time(time, offset),
    })
}

/// Renders a civil time with Cypher's reduced precision: minutes always,
/// seconds only when the seconds or fraction are non-zero, and the fraction
/// trimmed of trailing zeros.
fn render_time(time: civil::Time) -> String {
    let mut out = format!("{:02}:{:02}", time.hour(), time.minute());
    let nanos = time.subsec_nanosecond();
    if time.second() != 0 || nanos != 0 {
        out.push_str(&format!(":{:02}", time.second()));
        if nanos != 0 {
            let digits = format!("{nanos:09}");
            out.push('.');
            out.push_str(digits.trim_end_matches('0'));
        }
    }
    out
}

fn render_date(date: civil::Date) -> String {
    format!("{:04}-{:02}-{:02}", date.year(), date.month(), date.day())
}

fn render_offset(offset: Offset) -> String {
    let seconds = offset.seconds();
    if seconds == 0 {
        return "Z".to_owned();
    }
    let sign = if seconds < 0 { '-' } else { '+' };
    let seconds = seconds.abs();
    format!("{sign}{:02}:{:02}", seconds / 3600, (seconds % 3600) / 60)
}

fn render_temporal(value: &Temporal) -> String {
    match value {
        Temporal::Date(date) => render_date(*date),
        Temporal::LocalTime(time) => render_time(*time),
        Temporal::Time(time, offset) => format!("{}{}", render_time(*time), render_offset(*offset)),
        Temporal::LocalDateTime(datetime) => format!(
            "{}T{}",
            render_date(datetime.date()),
            render_time(datetime.time())
        ),
        Temporal::DateTime(zoned) => {
            let mut out = format!(
                "{}T{}{}",
                render_date(zoned.date()),
                render_time(zoned.time()),
                render_offset(zoned.offset())
            );
            // UTC renders as a bare Z; jiff reports fixed zero offsets as
            // the named UTC zone, and Cypher never brackets UTC.
            if let Some(name) = zoned.time_zone().iana_name() {
                if name != "UTC" {
                    out.push_str(&format!("[{name}]"));
                }
            }
            out
        }
    }
}

/// Parses a Cypher timezone component: an IANA name, `Z`, or `±HH:MM`.
fn parse_timezone(name: &str) -> Option<TimeZone> {
    let name = name.trim();
    if name == "Z" {
        return Some(TimeZone::fixed(Offset::UTC));
    }
    if let Ok(zone) = TimeZone::get(name) {
        return Some(zone);
    }
    let ("", ZoneSuffix::Fixed(offset)) = strip_zone(name) else {
        return None;
    };
    Some(TimeZone::fixed(offset))
}

fn temporal_date(value: &Temporal) -> Option<civil::Date> {
    match value {
        Temporal::Date(date) => Some(*date),
        Temporal::LocalDateTime(datetime) => Some(datetime.date()),
        Temporal::DateTime(zoned) => Some(zoned.date()),
        _ => None,
    }
}

fn temporal_time(value: &Temporal) -> Option<civil::Time> {
    match value {
        Temporal::LocalTime(time) | Temporal::Time(time, _) => Some(*time),
        Temporal::LocalDateTime(datetime) => Some(datetime.time()),
        Temporal::DateTime(zoned) => Some(zoned.time()),
        Temporal::Date(_) => None,
    }
}

fn temporal_zone(value: &Temporal) -> Option<TimeZone> {
    match value {
        Temporal::Time(_, offset) => Some(TimeZone::fixed(*offset)),
        Temporal::DateTime(zoned) => Some(zoned.time_zone().clone()),
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// Scalar functions.
// ---------------------------------------------------------------------------

fn integer(value: &ExtValue) -> Option<i64> {
    value.to_integer()
}

fn text(value: &ExtValue) -> Option<String> {
    value.to_text().map(|text| text.to_owned())
}

fn duration_value(value: &ExtValue) -> Option<Dur> {
    Dur::parse(&text(value)?)
}

fn temporal_argument(value: &ExtValue) -> Option<Temporal> {
    parse_temporal(&text(value)?)
}

#[scalar(name = "duration_make")]
fn duration_make(args: &[ExtValue]) -> ExtValue {
    duration_make_impl(args)
}

fn duration_make_impl(args: &[ExtValue]) -> ExtValue {
    let (Some(months), Some(days), Some(seconds), Some(nanos)) = (
        args.first().and_then(integer),
        args.get(1).and_then(integer),
        args.get(2).and_then(integer),
        args.get(3).and_then(integer),
    ) else {
        return ExtValue::null();
    };
    ExtValue::from_text(
        Dur {
            months,
            days,
            seconds,
            nanos,
        }
        .render(),
    )
}

#[scalar(name = "duration_parse")]
fn duration_parse(args: &[ExtValue]) -> ExtValue {
    duration_parse_impl(args)
}

fn duration_parse_impl(args: &[ExtValue]) -> ExtValue {
    match args.first().and_then(duration_value) {
        Some(value) => ExtValue::from_text(value.render()),
        None => ExtValue::null(),
    }
}

#[scalar(name = "duration_get")]
fn duration_get(args: &[ExtValue]) -> ExtValue {
    duration_get_impl(args)
}

fn duration_get_impl(args: &[ExtValue]) -> ExtValue {
    let (Some(value), Some(unit)) = (
        args.first().and_then(duration_value),
        args.get(1).and_then(text),
    ) else {
        return ExtValue::null();
    };
    let result = match unit.as_str() {
        "years" => value.months / 12,
        "quarters" => value.months / 3,
        "months" => value.months,
        "weeks" => value.days / 7,
        "days" => value.days,
        "hours" => value.seconds / 3600,
        "minutes" => value.seconds / 60,
        "seconds" => value.seconds,
        "milliseconds" => value.seconds * 1_000 + value.nanos / 1_000_000,
        "microseconds" => value.seconds * 1_000_000 + value.nanos / 1_000,
        "nanoseconds" => value.seconds * 1_000_000_000 + value.nanos,
        "monthsOfYear" => value.months % 12,
        "minutesOfHour" => (value.seconds % 3600) / 60,
        "secondsOfMinute" => value.seconds % 60,
        "millisecondsOfSecond" => value.nanos / 1_000_000,
        "microsecondsOfSecond" => value.nanos / 1_000,
        "nanosecondsOfSecond" => value.nanos,
        _ => return ExtValue::null(),
    };
    ExtValue::from_integer(result)
}

#[scalar(name = "duration_add")]
fn duration_add(args: &[ExtValue]) -> ExtValue {
    duration_add_impl(args)
}

fn duration_add_impl(args: &[ExtValue]) -> ExtValue {
    let (Some(left), Some(right)) = (
        args.first().and_then(duration_value),
        args.get(1).and_then(duration_value),
    ) else {
        return ExtValue::null();
    };
    ExtValue::from_text(
        Dur {
            months: left.months + right.months,
            days: left.days + right.days,
            seconds: left.seconds + right.seconds,
            nanos: left.nanos + right.nanos,
        }
        .render(),
    )
}

#[scalar(name = "duration_neg")]
fn duration_neg(args: &[ExtValue]) -> ExtValue {
    duration_neg_impl(args)
}

fn duration_neg_impl(args: &[ExtValue]) -> ExtValue {
    match args.first().and_then(duration_value) {
        Some(value) => ExtValue::from_text(
            Dur {
                months: -value.months,
                days: -value.days,
                seconds: -value.seconds,
                nanos: -value.nanos,
            }
            .render(),
        ),
        None => ExtValue::null(),
    }
}

/// `duration_between(start, end, mode)` where mode is `between`, `months`,
/// `days`, or `seconds`, matching duration.between and the duration.in*
/// variants: `between` yields months/days/time, the others truncate to
/// whole units of their kind.
#[scalar(name = "duration_between")]
fn duration_between(args: &[ExtValue]) -> ExtValue {
    duration_between_impl(args)
}

fn duration_between_impl(args: &[ExtValue]) -> ExtValue {
    let (Some(start), Some(end)) = (
        args.first().and_then(temporal_argument),
        args.get(1).and_then(temporal_argument),
    ) else {
        return ExtValue::null();
    };
    let mode = args
        .get(2)
        .and_then(text)
        .unwrap_or_else(|| "between".to_owned());
    let largest = match mode.as_str() {
        "days" => Unit::Day,
        "seconds" => Unit::Second,
        _ => Unit::Month,
    };
    let span = match (&start, &end) {
        (Temporal::DateTime(start), Temporal::DateTime(end)) => {
            // Calendar spans require one zone; compute in the start's.
            let end = end.with_time_zone(start.time_zone().clone());
            start.until((largest, &end)).ok()
        }
        _ => {
            // A value missing a date (a time) borrows the other argument's
            // date so date-to-time differences stay within one day, per
            // openCypher's component-filling rule; missing times default to
            // midnight from their own value.
            let promote = |value: &Temporal, other: &Temporal| -> Option<civil::DateTime> {
                let date = temporal_date(value)
                    .or_else(|| temporal_date(other))
                    .or_else(|| civil::Date::new(1970, 1, 1).ok())?;
                Some(date.to_datetime(temporal_time(value).unwrap_or(civil::Time::midnight())))
            };
            let (Some(start), Some(end)) = (promote(&start, &end), promote(&end, &start)) else {
                return ExtValue::null();
            };
            start.until((largest, end)).ok()
        }
    };
    let Some(span) = span else {
        return ExtValue::null();
    };
    let mut value = span_to_dur(span);
    match mode.as_str() {
        "months" => {
            value.days = 0;
            value.seconds = 0;
            value.nanos = 0;
        }
        "days" => {
            value.seconds = 0;
            value.nanos = 0;
        }
        _ => {}
    }
    ExtValue::from_text(value.render())
}

/// `temporal_make(kind, components_json)` builds a temporal value from a
/// Cypher component map: calendar (year/month/day), week (year/week/
/// dayOfWeek), ordinal (year/ordinalDay), quarter (year/quarter/
/// dayOfQuarter) dates; time components down to nanoseconds; `timezone`;
/// composition from existing `date`/`time`/`datetime` values; and epoch
/// seconds/milliseconds.
#[scalar(name = "temporal_make")]
fn temporal_make(args: &[ExtValue]) -> ExtValue {
    temporal_make_impl(args)
}

fn temporal_make_impl(args: &[ExtValue]) -> ExtValue {
    let (Some(kind), Some(json)) = (args.first().and_then(text), args.get(1).and_then(text)) else {
        return ExtValue::null();
    };
    let Ok(serde_json::Value::Object(map)) = serde_json::from_str::<serde_json::Value>(&json)
    else {
        return ExtValue::null();
    };
    match build_temporal(&kind, &map) {
        Some(value) => ExtValue::from_text(render_temporal(&value)),
        None => ExtValue::null(),
    }
}

/// `temporal_truncate(kind, unit, value[, components_json])` truncates a
/// temporal value to the start of `unit`, applies optional component
/// overrides, and coerces onto the requested kind — the `<kind>.truncate`
/// Cypher functions.
#[scalar(name = "temporal_truncate")]
fn temporal_truncate(args: &[ExtValue]) -> ExtValue {
    temporal_truncate_impl(args)
}

fn temporal_truncate_impl(args: &[ExtValue]) -> ExtValue {
    let (Some(kind), Some(unit), Some(value)) = (
        args.first().and_then(text),
        args.get(1).and_then(text),
        args.get(2).and_then(temporal_argument),
    ) else {
        return ExtValue::null();
    };
    let date = temporal_date(&value);
    let time = temporal_time(&value).unwrap_or(civil::Time::midnight());
    let zone = temporal_zone(&value);
    let Some((date, time)) = truncate_parts(date, time, &unit) else {
        return ExtValue::null();
    };
    let overrides = args.get(3).and_then(text).map(|json| {
        match serde_json::from_str::<serde_json::Value>(&json) {
            Ok(serde_json::Value::Object(map)) => Some(map),
            _ => None,
        }
    });
    let assembled = match overrides {
        Some(None) => None,
        Some(Some(map)) => {
            let zone = match component_text(&map, "timezone") {
                Some(name) => parse_timezone(&name),
                None => zone,
            };
            (|| {
                // Sub-second overrides replace their own field on the
                // truncated base: truncate('millisecond', .645876123,
                // {nanosecond: 2}) keeps .645 and yields .645000002.
                let base_subsec = i64::from(time.subsec_nanosecond());
                let merged_subsec = component_i64(&map, "millisecond")
                    .unwrap_or(base_subsec / 1_000_000)
                    * 1_000_000
                    + component_i64(&map, "microsecond").unwrap_or(base_subsec / 1_000 % 1_000)
                        * 1_000
                    + component_i64(&map, "nanosecond").unwrap_or(base_subsec % 1_000);
                let mut map = map;
                for key in ["millisecond", "microsecond", "nanosecond"] {
                    map.remove(key);
                }
                let date = build_date(&map, date)?;
                let time = build_time(&map, Some(time))?;
                let time = civil::Time::new(
                    time.hour(),
                    time.minute(),
                    time.second(),
                    i32::try_from(merged_subsec).ok()?,
                )
                .ok()?;
                assemble_temporal(&kind, date, time, zone)
            })()
        }
        None => assemble_temporal(&kind, date, time, zone),
    };
    match assembled {
        Some(value) => ExtValue::from_text(render_temporal(&value)),
        None => ExtValue::null(),
    }
}

/// Truncates date/time parts to the start of `unit`. Time-only values pass
/// `None` for the date and keep it `None`.
fn truncate_parts(
    date: Option<civil::Date>,
    time: civil::Time,
    unit: &str,
) -> Option<(Option<civil::Date>, civil::Time)> {
    let midnight = civil::Time::midnight();
    if let Some(date) = date {
        let year = date.year();
        let start_of = |year: i16| civil::Date::new(year, 1, 1).ok();
        let truncated = match unit {
            "millennium" => (start_of(year.div_euclid(1000) * 1000)?, midnight),
            "century" => (start_of(year.div_euclid(100) * 100)?, midnight),
            "decade" => (start_of(year.div_euclid(10) * 10)?, midnight),
            "year" => (start_of(year)?, midnight),
            "weekYear" => (
                civil::ISOWeekDate::new(date.iso_week_date().year(), 1, civil::Weekday::Monday)
                    .ok()?
                    .date(),
                midnight,
            ),
            "quarter" => (
                civil::Date::new(year, (date.month() - 1) / 3 * 3 + 1, 1).ok()?,
                midnight,
            ),
            "month" => (civil::Date::new(year, date.month(), 1).ok()?, midnight),
            "week" => (
                date.checked_sub(
                    Span::new().days(i64::from(date.weekday().to_monday_zero_offset())),
                )
                .ok()?,
                midnight,
            ),
            "day" => (date, midnight),
            _ => (date, truncate_time(time, unit)?),
        };
        return Some((Some(truncated.0), truncated.1));
    }
    Some((None, truncate_time(time, unit)?))
}

fn truncate_time(time: civil::Time, unit: &str) -> Option<civil::Time> {
    let subsec = time.subsec_nanosecond();
    match unit {
        "day" => Some(civil::Time::midnight()),
        "hour" => civil::Time::new(time.hour(), 0, 0, 0).ok(),
        "minute" => civil::Time::new(time.hour(), time.minute(), 0, 0).ok(),
        "second" => civil::Time::new(time.hour(), time.minute(), time.second(), 0).ok(),
        "millisecond" => civil::Time::new(
            time.hour(),
            time.minute(),
            time.second(),
            subsec / 1_000_000 * 1_000_000,
        )
        .ok(),
        "microsecond" => civil::Time::new(
            time.hour(),
            time.minute(),
            time.second(),
            subsec / 1_000 * 1_000,
        )
        .ok(),
        _ => None,
    }
}

fn component_i64(map: &serde_json::Map<String, serde_json::Value>, key: &str) -> Option<i64> {
    match map.get(key)? {
        serde_json::Value::Number(number) => number
            .as_i64()
            .or_else(|| number.as_f64().map(|real| real as i64)),
        serde_json::Value::String(string) => string.parse().ok(),
        _ => None,
    }
}

fn component_text(map: &serde_json::Map<String, serde_json::Value>, key: &str) -> Option<String> {
    map.get(key)?.as_str().map(|value| value.to_owned())
}

fn build_temporal(
    kind: &str,
    map: &serde_json::Map<String, serde_json::Value>,
) -> Option<Temporal> {
    let base_datetime = component_text(map, "datetime").and_then(|value| parse_temporal(&value));
    let base_date = component_text(map, "date").and_then(|value| parse_temporal(&value));
    let base_time = component_text(map, "time").and_then(|value| parse_temporal(&value));

    let explicit_zone = component_text(map, "timezone").is_some();
    let zone = match component_text(map, "timezone") {
        Some(name) => Some(parse_timezone(&name)?),
        None => [&base_time, &base_datetime]
            .into_iter()
            .flatten()
            .find_map(temporal_zone),
    };

    // An explicit timezone over a zoned base converts the instant into the
    // new zone (12:00+01:00 selected into +05:00 reads 16:00); unzoned
    // bases just attach the zone.
    let convert = |value: &Temporal| -> Option<Temporal> {
        let target = zone.clone()?;
        match value {
            Temporal::Time(time, offset) => {
                // jiff converts through the instant: anchor the wall time on
                // an arbitrary date, resolve it with the old offset, and read
                // it back in the new one.
                let target_offset = target.to_offset(Timestamp::now());
                let anchored = civil::Date::new(2000, 1, 1).ok()?.to_datetime(*time);
                let timestamp = offset.to_timestamp(anchored).ok()?;
                Some(Temporal::Time(
                    target_offset.to_datetime(timestamp).time(),
                    target_offset,
                ))
            }
            Temporal::DateTime(zoned) => Some(Temporal::DateTime(zoned.with_time_zone(target))),
            _ => None,
        }
    };
    let base_datetime = match (&base_datetime, explicit_zone) {
        (Some(value), true) => convert(value).or(base_datetime),
        _ => base_datetime,
    };
    let base_time = match (&base_time, explicit_zone) {
        (Some(value), true) => convert(value).or(base_time),
        _ => base_time,
    };

    // Epoch construction takes precedence for datetimes.
    if kind == "datetime" {
        let timestamp = if let Some(seconds) = component_i64(map, "epochSeconds") {
            Some(
                Timestamp::new(
                    seconds,
                    component_i64(map, "nanosecond").unwrap_or(0) as i32,
                )
                .ok()?,
            )
        } else {
            component_i64(map, "epochMillis")
                .map(Timestamp::from_millisecond)
                .transpose()
                .ok()?
        };
        if let Some(timestamp) = timestamp {
            let zone = zone.unwrap_or_else(|| TimeZone::fixed(Offset::UTC));
            return Some(Temporal::DateTime(timestamp.to_zoned(zone)));
        }
    }

    let inherited_date = [&base_date, &base_datetime]
        .into_iter()
        .flatten()
        .find_map(temporal_date);
    let inherited_time = [&base_time, &base_datetime]
        .into_iter()
        .flatten()
        .find_map(temporal_time);

    let date = build_date(map, inherited_date)?;
    let time = build_time(map, inherited_time)?;
    assemble_temporal(kind, date, time, zone)
}

/// Assembles a temporal value of the requested kind from its parts.
fn assemble_temporal(
    kind: &str,
    date: Option<civil::Date>,
    time: civil::Time,
    zone: Option<TimeZone>,
) -> Option<Temporal> {
    Some(match kind {
        "date" => Temporal::Date(date?),
        "localtime" => Temporal::LocalTime(time),
        "time" => {
            let offset = match &zone {
                Some(zone) => zone.to_offset(Timestamp::now()),
                None => Offset::UTC,
            };
            Temporal::Time(time, offset)
        }
        "localdatetime" => Temporal::LocalDateTime(date?.to_datetime(time)),
        "datetime" => {
            let zone = zone.unwrap_or_else(|| TimeZone::fixed(Offset::UTC));
            Temporal::DateTime(date?.to_datetime(time).to_zoned(zone).ok()?)
        }
        _ => return None,
    })
}

/// Builds the date part; `None` inner value means no date components were
/// given at all (legal only for time kinds).
fn build_date(
    map: &serde_json::Map<String, serde_json::Value>,
    inherited: Option<civil::Date>,
) -> Option<Option<civil::Date>> {
    let year = component_i64(map, "year");
    if year.is_none() && inherited.is_none() {
        return Some(None);
    }
    let year = i16::try_from(year.unwrap_or_else(|| i64::from(inherited.unwrap().year()))).ok()?;
    if let Some(week) = component_i64(map, "week") {
        // Overriding the week keeps the base date's position within the
        // week: {date: <sunday>, week: 1} lands on the Sunday of week 1.
        let default_weekday = inherited
            .map(|date| i64::from(date.weekday().to_monday_one_offset()))
            .unwrap_or(1);
        let weekday = civil::Weekday::from_monday_one_offset(
            i8::try_from(component_i64(map, "dayOfWeek").unwrap_or(default_weekday)).ok()?,
        )
        .ok()?;
        let week_date = civil::ISOWeekDate::new(year, i8::try_from(week).ok()?, weekday).ok()?;
        return Some(Some(week_date.date()));
    }
    if let Some(ordinal) = component_i64(map, "ordinalDay") {
        let start = civil::Date::new(year, 1, 1).ok()?;
        return Some(Some(start.checked_add(Span::new().days(ordinal - 1)).ok()?));
    }
    if let Some(quarter) = component_i64(map, "quarter") {
        let month = i8::try_from((quarter - 1) * 3 + 1).ok()?;
        let start = civil::Date::new(year, month, 1).ok()?;
        // Overriding the quarter keeps the base date's day-of-quarter.
        let default_day_of_quarter = inherited
            .map(|date| {
                let quarter_start_month = (date.month() - 1) / 3 * 3 + 1;
                let quarter_start = civil::Date::new(date.year(), quarter_start_month, 1)
                    .expect("first day of a quarter is always valid");
                i64::from(date.day_of_year()) - i64::from(quarter_start.day_of_year()) + 1
            })
            .unwrap_or(1);
        let day_of_quarter = component_i64(map, "dayOfQuarter").unwrap_or(default_day_of_quarter);
        return Some(Some(
            start
                .checked_add(Span::new().days(day_of_quarter - 1))
                .ok()?,
        ));
    }
    let month = component_i64(map, "month")
        .map(i8::try_from)
        .transpose()
        .ok()?
        .or_else(|| inherited.map(civil::Date::month))
        .unwrap_or(1);
    let day = component_i64(map, "day")
        .map(i8::try_from)
        .transpose()
        .ok()?
        .or_else(|| inherited.map(civil::Date::day))
        .unwrap_or(1);
    Some(Some(civil::Date::new(year, month, day).ok()?))
}

fn build_time(
    map: &serde_json::Map<String, serde_json::Value>,
    inherited: Option<civil::Time>,
) -> Option<civil::Time> {
    let component = |key: &str| component_i64(map, key);
    let explicit_fraction = ["millisecond", "microsecond", "nanosecond"]
        .iter()
        .any(|key| map.contains_key(*key));
    let subsec = if explicit_fraction {
        i32::try_from(
            component("millisecond").unwrap_or(0) * 1_000_000
                + component("microsecond").unwrap_or(0) * 1_000
                + component("nanosecond").unwrap_or(0),
        )
        .ok()?
    } else {
        inherited.map(|time| time.subsec_nanosecond()).unwrap_or(0)
    };
    let field = |key: &str, from_inherited: fn(civil::Time) -> i8| {
        component(key)
            .map(i8::try_from)
            .map(|value| value.ok())
            .unwrap_or_else(|| Some(inherited.map(from_inherited).unwrap_or(0)))
    };
    civil::Time::new(
        field("hour", civil::Time::hour)?,
        field("minute", civil::Time::minute)?,
        field("second", civil::Time::second)?,
        subsec,
    )
    .ok()
}

/// Parses the extended ISO-8601 date forms Cypher accepts: calendar
/// (`2015-07-21`, `20150721`), reduced (`2015-07`, `201507`, `2015`),
/// week (`2015-W30-2`, `2015W302`, `2015-W30`), and ordinal (`2015-202`,
/// `2015202`).
fn parse_date_extended(text: &str) -> Option<civil::Date> {
    if let Ok(date) = text.parse::<civil::Date>() {
        return Some(date);
    }
    if let Some(position) = text.find(['W', 'w']) {
        let year: i16 = text[..position].trim_end_matches('-').parse().ok()?;
        let rest = text[position + 1..].replace('-', "");
        let (week, day) = match rest.len() {
            2 => (rest.parse().ok()?, 1),
            3 => (rest[..2].parse().ok()?, rest[2..].parse().ok()?),
            _ => return None,
        };
        let weekday = civil::Weekday::from_monday_one_offset(day).ok()?;
        return Some(civil::ISOWeekDate::new(year, week, weekday).ok()?.date());
    }
    let parts: Vec<&str> = text.split('-').collect();
    let date =
        |year: &str, month: i8, day: i8| civil::Date::new(year.parse().ok()?, month, day).ok();
    let ordinal = |year: &str, days: &str| {
        let start = civil::Date::new(year.parse().ok()?, 1, 1).ok()?;
        start
            .checked_add(Span::new().days(days.parse::<i64>().ok()? - 1))
            .ok()
    };
    match parts.as_slice() {
        [all] if all.len() == 4 => date(all, 1, 1),
        [all] if all.len() == 6 => date(&all[..4], all[4..].parse().ok()?, 1),
        [all] if all.len() == 7 => ordinal(&all[..4], &all[4..]),
        [all] if all.len() == 8 => date(&all[..4], all[4..6].parse().ok()?, all[6..].parse().ok()?),
        [year, month] if year.len() == 4 && month.len() == 2 => date(year, month.parse().ok()?, 1),
        [year, days] if year.len() == 4 && days.len() == 3 => ordinal(year, days),
        _ => None,
    }
}

/// Parses extended time forms: `21:40:32.142`, `214032.142`, `2140`, `21`.
fn parse_time_extended(text: &str) -> Option<civil::Time> {
    if let Ok(time) = text.parse::<civil::Time>() {
        return Some(time);
    }
    let (whole, fraction) = match text.split_once('.') {
        Some((whole, fraction)) => (whole, fraction),
        None => (text, ""),
    };
    if !whole.chars().all(|c| c.is_ascii_digit()) || whole.is_empty() {
        return None;
    }
    let field =
        |range: Option<&str>| -> Option<i8> { range.map_or(Some(0), |digits| digits.parse().ok()) };
    let (hour, minute, second) = match whole.len() {
        2 => (field(Some(whole))?, 0, 0),
        4 => (field(Some(&whole[..2]))?, field(Some(&whole[2..]))?, 0),
        6 => (
            field(Some(&whole[..2]))?,
            field(Some(&whole[2..4]))?,
            field(Some(&whole[4..]))?,
        ),
        _ => return None,
    };
    let subsec = if fraction.is_empty() {
        0
    } else {
        let mut digits = fraction.to_owned();
        if digits.len() > 9 {
            return None;
        }
        while digits.len() < 9 {
            digits.push('0');
        }
        digits.parse::<i32>().ok()?
    };
    civil::Time::new(hour, minute, second, subsec).ok()
}

/// Splits a trailing offset in the extended forms constructor strings
/// allow: `Z`, `±HH:MM`, `±HHMM`, and `±HH`. Only safe on time-side
/// strings, where `-` cannot be a date separator.
fn strip_zone_extended(text: &str) -> (&str, ZoneSuffix) {
    let (core, zone) = strip_zone(text);
    if !matches!(zone, ZoneSuffix::Missing) {
        return (core, zone);
    }
    for (length, has_minutes) in [(5, true), (3, false)] {
        if text.len() <= length {
            continue;
        }
        let tail = &text[text.len() - length..];
        let bytes = tail.as_bytes();
        if bytes[0] != b'+' && bytes[0] != b'-' {
            continue;
        }
        if !tail[1..].bytes().all(|byte| byte.is_ascii_digit()) {
            continue;
        }
        let hours: i32 = tail[1..3].parse().ok().unwrap_or(-1);
        let minutes: i32 = if has_minutes {
            tail[3..].parse().ok().unwrap_or(-1)
        } else {
            0
        };
        if !(0..=23).contains(&hours) || !(0..=59).contains(&minutes) {
            continue;
        }
        let sign = if bytes[0] == b'+' { 1 } else { -1 };
        if let Ok(offset) = Offset::from_seconds(sign * (hours * 3600 + minutes * 60)) {
            return (&text[..text.len() - length], ZoneSuffix::Fixed(offset));
        }
    }
    (text, ZoneSuffix::Missing)
}

/// Kind-aware string parsing: constructor strings use forms the generic
/// parser cannot disambiguate (a bare `2140` is a time, not a year).
fn parse_temporal_with_kind(kind: &str, text: &str) -> Option<Temporal> {
    let text = text.trim();
    if text.contains('[') {
        return parse_temporal(text);
    }
    match kind {
        "date" => parse_date_extended(text).map(Temporal::Date),
        "localtime" => parse_time_extended(text).map(Temporal::LocalTime),
        "time" => {
            let (core, zone) = strip_zone_extended(text);
            let time = parse_time_extended(core)?;
            Some(match zone {
                ZoneSuffix::Missing | ZoneSuffix::Utc => Temporal::Time(time, Offset::UTC),
                ZoneSuffix::Fixed(offset) => Temporal::Time(time, offset),
            })
        }
        "localdatetime" | "datetime" => {
            let (date_part, time_part, zone) = match text.split_once('T') {
                Some((date, time)) => {
                    let (core, zone) = strip_zone_extended(time);
                    (date, Some(core.to_owned()), zone)
                }
                None => (text, None, ZoneSuffix::Missing),
            };
            let date = parse_date_extended(date_part)?;
            let time = match time_part {
                Some(time) => parse_time_extended(&time)?,
                None => civil::Time::midnight(),
            };
            let datetime = date.to_datetime(time);
            if kind == "localdatetime" {
                return Some(Temporal::LocalDateTime(datetime));
            }
            let tz = zone_of(&zone).unwrap_or_else(|| TimeZone::fixed(Offset::UTC));
            datetime.to_zoned(tz).ok().map(Temporal::DateTime)
        }
        _ => None,
    }
}

#[scalar(name = "temporal_parse")]
fn temporal_parse(args: &[ExtValue]) -> ExtValue {
    temporal_parse_impl(args)
}

fn temporal_parse_impl(args: &[ExtValue]) -> ExtValue {
    let (Some(kind), Some(raw)) = (args.first().and_then(text), args.get(1).and_then(text)) else {
        return ExtValue::null();
    };
    if let Some(value) = parse_temporal_with_kind(&kind, &raw) {
        return ExtValue::from_text(render_temporal(&value));
    }
    let Some(value) = parse_temporal(&raw) else {
        return ExtValue::null();
    };
    // Coerce the parsed value onto the requested kind.
    let coerced = match (kind.as_str(), &value) {
        ("date", _) => temporal_date(&value).map(Temporal::Date),
        ("localtime", _) => temporal_time(&value).map(Temporal::LocalTime),
        ("time", Temporal::Time(..)) => Some(value.clone()),
        ("time", _) => temporal_time(&value).map(|time| Temporal::Time(time, Offset::UTC)),
        ("localdatetime", Temporal::Date(date)) => Some(Temporal::LocalDateTime(
            date.to_datetime(civil::Time::midnight()),
        )),
        ("localdatetime", Temporal::DateTime(zoned)) => {
            Some(Temporal::LocalDateTime(zoned.datetime()))
        }
        ("localdatetime", Temporal::LocalDateTime(_)) => Some(value.clone()),
        ("datetime", Temporal::DateTime(_)) => Some(value.clone()),
        ("datetime", Temporal::LocalDateTime(datetime)) => datetime
            .to_zoned(TimeZone::fixed(Offset::UTC))
            .ok()
            .map(Temporal::DateTime),
        ("datetime", Temporal::Date(date)) => date
            .to_datetime(civil::Time::midnight())
            .to_zoned(TimeZone::fixed(Offset::UTC))
            .ok()
            .map(Temporal::DateTime),
        _ => None,
    };
    match coerced {
        Some(value) => ExtValue::from_text(render_temporal(&value)),
        None => ExtValue::null(),
    }
}

#[scalar(name = "temporal_get")]
fn temporal_get(args: &[ExtValue]) -> ExtValue {
    temporal_get_impl(args)
}

fn temporal_get_impl(args: &[ExtValue]) -> ExtValue {
    let (Some(value), Some(unit)) = (
        args.first().and_then(temporal_argument),
        args.get(1).and_then(text),
    ) else {
        return ExtValue::null();
    };
    let date = temporal_date(&value);
    let time = temporal_time(&value);
    let from_date = |get: fn(civil::Date) -> i64| date.map(get).map(ExtValue::from_integer);
    let from_time = |get: fn(civil::Time) -> i64| time.map(get).map(ExtValue::from_integer);
    let result = match unit.as_str() {
        "year" => from_date(|date| i64::from(date.year())),
        "month" => from_date(|date| i64::from(date.month())),
        "day" => from_date(|date| i64::from(date.day())),
        "week" => from_date(|date| i64::from(date.iso_week_date().week())),
        "weekYear" => from_date(|date| i64::from(date.iso_week_date().year())),
        "quarter" => from_date(|date| i64::from((date.month() - 1) / 3 + 1)),
        "dayOfQuarter" => date.and_then(|date| {
            let month = (date.month() - 1) / 3 * 3 + 1;
            let start = civil::Date::new(date.year(), month, 1).ok()?;
            Some(ExtValue::from_integer(
                i64::from(date.day_of_year()) - i64::from(start.day_of_year()) + 1,
            ))
        }),
        "ordinalDay" | "dayOfYear" => from_date(|date| i64::from(date.day_of_year())),
        "weekday" | "dayOfWeek" => {
            from_date(|date| i64::from(date.weekday().to_monday_one_offset()))
        }
        "hour" => from_time(|time| i64::from(time.hour())),
        "minute" => from_time(|time| i64::from(time.minute())),
        "second" => from_time(|time| i64::from(time.second())),
        "millisecond" => from_time(|time| i64::from(time.subsec_nanosecond()) / 1_000_000),
        "microsecond" => from_time(|time| i64::from(time.subsec_nanosecond()) / 1_000),
        "nanosecond" => from_time(|time| i64::from(time.subsec_nanosecond())),
        "timezone" => match &value {
            Temporal::DateTime(zoned) => Some(ExtValue::from_text(
                zoned
                    .time_zone()
                    .iana_name()
                    .map(str::to_owned)
                    .unwrap_or_else(|| render_offset(zoned.offset())),
            )),
            Temporal::Time(_, offset) => Some(ExtValue::from_text(render_offset(*offset))),
            _ => None,
        },
        "offset" => match &value {
            Temporal::DateTime(zoned) => Some(ExtValue::from_text(render_offset(zoned.offset()))),
            Temporal::Time(_, offset) => Some(ExtValue::from_text(render_offset(*offset))),
            _ => None,
        },
        "offsetMinutes" => match &value {
            Temporal::DateTime(zoned) => Some(ExtValue::from_integer(
                i64::from(zoned.offset().seconds()) / 60,
            )),
            Temporal::Time(_, offset) => {
                Some(ExtValue::from_integer(i64::from(offset.seconds()) / 60))
            }
            _ => None,
        },
        "epochSeconds" => match &value {
            Temporal::DateTime(zoned) => {
                Some(ExtValue::from_integer(zoned.timestamp().as_second()))
            }
            _ => None,
        },
        "epochMillis" => match &value {
            Temporal::DateTime(zoned) => {
                Some(ExtValue::from_integer(zoned.timestamp().as_millisecond()))
            }
            _ => None,
        },
        _ => None,
    };
    result.unwrap_or_else(ExtValue::null)
}

#[scalar(name = "temporal_now")]
fn temporal_now(args: &[ExtValue]) -> ExtValue {
    temporal_now_impl(args)
}

fn temporal_now_impl(args: &[ExtValue]) -> ExtValue {
    let Some(kind) = args.first().and_then(text) else {
        return ExtValue::null();
    };
    let now = Timestamp::now().to_zoned(TimeZone::fixed(Offset::UTC));
    let value = match kind.as_str() {
        "date" => Temporal::Date(now.date()),
        "localtime" => Temporal::LocalTime(now.time()),
        "time" => Temporal::Time(now.time(), Offset::UTC),
        "localdatetime" => Temporal::LocalDateTime(now.datetime()),
        "datetime" => Temporal::DateTime(now),
        _ => return ExtValue::null(),
    };
    ExtValue::from_text(render_temporal(&value))
}

/// Shifts a temporal value by a duration, preserving its kind and zone.
/// Calendar steps clamp at month ends; times wrap; date shifts count only
/// whole days from the duration's time part.
fn shift_temporal(value: Temporal, duration: Dur, sign: i64) -> Option<Temporal> {
    let months = duration.months * sign;
    let days = duration.days * sign;
    let seconds = duration.seconds * sign;
    let nanos = duration.nanos * sign;
    let month_span = Span::new().months(months);
    let day_span = Span::new().days(days);
    let time_span = Span::new().seconds(seconds).nanoseconds(nanos);
    Some(match value {
        Temporal::Date(date) => Temporal::Date(
            date.checked_add(month_span)
                .ok()?
                .checked_add(Span::new().days(days + seconds / 86_400))
                .ok()?,
        ),
        Temporal::LocalTime(time) => Temporal::LocalTime(time.wrapping_add(time_span)),
        Temporal::Time(time, offset) => Temporal::Time(time.wrapping_add(time_span), offset),
        Temporal::LocalDateTime(datetime) => Temporal::LocalDateTime(
            datetime
                .checked_add(month_span)
                .ok()?
                .checked_add(day_span)
                .ok()?
                .checked_add(time_span)
                .ok()?,
        ),
        Temporal::DateTime(zoned) => Temporal::DateTime(
            zoned
                .checked_add(month_span)
                .ok()?
                .checked_add(day_span)
                .ok()?
                .checked_add(time_span)
                .ok()?,
        ),
    })
}

fn shift_datetime(args: &[ExtValue], sign: i64) -> ExtValue {
    let (Some(value), Some(duration)) = (
        args.first().and_then(temporal_argument),
        args.get(1).and_then(duration_value),
    ) else {
        return ExtValue::null();
    };
    match shift_temporal(value, duration, sign) {
        Some(shifted) => ExtValue::from_text(render_temporal(&shifted)),
        None => ExtValue::null(),
    }
}

#[scalar(name = "datetime_add_duration")]
fn datetime_add_duration(args: &[ExtValue]) -> ExtValue {
    shift_datetime(args, 1)
}

#[scalar(name = "datetime_sub_duration")]
fn datetime_sub_duration(args: &[ExtValue]) -> ExtValue {
    shift_datetime(args, -1)
}

// ---------------------------------------------------------------------------
// jsonb operator support: postgres/AGE operator semantics over JSON text.
// ---------------------------------------------------------------------------

/// Raises a runtime error with the given kind and detail — the escape
/// hatch for Cypher runtime errors (TypeError and friends) that plain
/// SQL expressions cannot produce.
#[scalar(name = "cypher_raise")]
fn cypher_raise(args: &[ExtValue]) -> ExtValue {
    cypher_raise_impl(args)
}

fn cypher_raise_impl(args: &[ExtValue]) -> ExtValue {
    let kind = args
        .first()
        .and_then(text)
        .unwrap_or_else(|| "Error".to_owned());
    let detail = args.get(1).and_then(text).unwrap_or_default();
    ExtValue::error_with_message(format!("{kind}: {detail}"))
}

fn json_argument(value: &ExtValue) -> Option<serde_json::Value> {
    serde_json::from_str(&text(value)?).ok()
}

/// Three-valued deep equality over Cypher values encoded as SQL values
/// (lists/maps as JSON text, JSON null as Cypher null). Returns 1, 0, or
/// SQL NULL for Cypher's `null`-propagating `=`.
#[scalar(name = "cypher_equals")]
fn cypher_equals(args: &[ExtValue]) -> ExtValue {
    cypher_equals_impl(args)
}

fn cypher_equals_impl(args: &[ExtValue]) -> ExtValue {
    let (Some(left), Some(right)) = (args.first(), args.get(1)) else {
        return ExtValue::null();
    };
    match deep_equal(&comparison_value(left), &comparison_value(right)) {
        Some(true) => ExtValue::from_integer(1),
        Some(false) => ExtValue::from_integer(0),
        None => ExtValue::null(),
    }
}

/// Cypher `+` over dynamically typed operands: temporal and duration values
/// retain their arithmetic after storage erases their marker types; numbers
/// add, strings concatenate, lists concatenate or append, and null propagates.
#[scalar(name = "cypher_add")]
fn cypher_add(args: &[ExtValue]) -> ExtValue {
    cypher_add_impl(args)
}

fn cypher_add_impl(args: &[ExtValue]) -> ExtValue {
    use turso_ext::ValueType;
    let (Some(left), Some(right)) = (args.first(), args.get(1)) else {
        return ExtValue::null();
    };
    if left.value_type() == ValueType::Null || right.value_type() == ValueType::Null {
        return ExtValue::null();
    }
    if let (Some(left), Some(right)) = (duration_value(left), duration_value(right)) {
        return ExtValue::from_text(
            Dur {
                months: left.months + right.months,
                days: left.days + right.days,
                seconds: left.seconds + right.seconds,
                nanos: left.nanos + right.nanos,
            }
            .render(),
        );
    }
    if let (Some(value), Some(duration)) = (temporal_argument(left), duration_value(right)) {
        return shift_temporal(value, duration, 1)
            .map(|shifted| ExtValue::from_text(render_temporal(&shifted)))
            .unwrap_or_else(ExtValue::null);
    }
    if let (Some(duration), Some(value)) = (duration_value(left), temporal_argument(right)) {
        return shift_temporal(value, duration, 1)
            .map(|shifted| ExtValue::from_text(render_temporal(&shifted)))
            .unwrap_or_else(ExtValue::null);
    }
    let structured = |value: &ExtValue| -> serde_json::Value { comparison_value(value) };
    let (left, right) = (structured(left), structured(right));
    use serde_json::Value as V;
    match (left, right) {
        (V::Array(mut left), V::Array(right)) => {
            left.extend(right);
            ExtValue::from_text(V::Array(left).to_string())
        }
        (V::Array(mut left), scalar) => {
            left.push(scalar);
            ExtValue::from_text(V::Array(left).to_string())
        }
        (scalar, V::Array(mut right)) => {
            right.insert(0, scalar);
            ExtValue::from_text(V::Array(right).to_string())
        }
        (V::Number(left), V::Number(right)) => match (left.as_i64(), right.as_i64()) {
            (Some(left), Some(right)) => ExtValue::from_integer(left.wrapping_add(right)),
            _ => ExtValue::from_float(
                left.as_f64().unwrap_or_default() + right.as_f64().unwrap_or_default(),
            ),
        },
        (V::String(left), V::String(right)) => ExtValue::from_text(format!("{left}{right}")),
        (V::String(left), V::Number(right)) => ExtValue::from_text(format!("{left}{right}")),
        (V::Number(left), V::String(right)) => ExtValue::from_text(format!("{left}{right}")),
        // Maps and booleans have no + in Cypher: a genuine TypeError, not a
        // silent null.
        _ => ExtValue::error_with_message("TypeError: invalid operand types for +".to_owned()),
    }
}

/// Cypher `-` over dynamically typed operands. Persisted graph properties are
/// untyped at bind time, so duration subtraction and temporal shifting must be
/// selected from their runtime values before falling back to numeric math.
#[scalar(name = "cypher_sub")]
fn cypher_sub(args: &[ExtValue]) -> ExtValue {
    cypher_sub_impl(args)
}

fn cypher_sub_impl(args: &[ExtValue]) -> ExtValue {
    use turso_ext::ValueType;
    let (Some(left), Some(right)) = (args.first(), args.get(1)) else {
        return ExtValue::null();
    };
    if left.value_type() == ValueType::Null || right.value_type() == ValueType::Null {
        return ExtValue::null();
    }
    if let (Some(left), Some(right)) = (duration_value(left), duration_value(right)) {
        return ExtValue::from_text(
            Dur {
                months: left.months - right.months,
                days: left.days - right.days,
                seconds: left.seconds - right.seconds,
                nanos: left.nanos - right.nanos,
            }
            .render(),
        );
    }
    if let (Some(value), Some(duration)) = (temporal_argument(left), duration_value(right)) {
        return shift_temporal(value, duration, -1)
            .map(|shifted| ExtValue::from_text(render_temporal(&shifted)))
            .unwrap_or_else(ExtValue::null);
    }
    if left.value_type() == ValueType::Integer && right.value_type() == ValueType::Integer {
        let (Some(left), Some(right)) = (left.to_integer(), right.to_integer()) else {
            return ExtValue::null();
        };
        return ExtValue::from_integer(left.wrapping_sub(right));
    }
    let (Some(left), Some(right)) = (left.to_float(), right.to_float()) else {
        return ExtValue::error_with_message("TypeError: invalid operand types for -".to_owned());
    };
    ExtValue::from_float(left - right)
}

/// Apache AGE agtype `||`: arrays concatenate or absorb a scalar, maps merge,
/// and other values form a two-element array. The binder supplies map-kind
/// flags so entity objects remain scalar values rather than being merged.
#[scalar(name = "cypher_concat")]
fn cypher_concat(args: &[ExtValue]) -> ExtValue {
    cypher_concat_impl(args)
}

fn cypher_concat_impl(args: &[ExtValue]) -> ExtValue {
    let (Some(left), Some(right), Some(left_kind), Some(right_kind)) =
        (args.first(), args.get(1), args.get(2), args.get(3))
    else {
        return ExtValue::null();
    };
    let (left, right) = (comparison_value(left), comparison_value(right));
    let map_kind = |kind: &ExtValue, value: &serde_json::Value| {
        value.is_object() && kind.to_integer() != Some(3)
    };
    let left_map = map_kind(left_kind, &left);
    let right_map = map_kind(right_kind, &right);
    let left_entity = left_kind.to_integer() == Some(3);
    let right_entity = right_kind.to_integer() == Some(3);
    use serde_json::Value as V;
    match (left, right) {
        (V::Array(mut left), V::Array(right)) => {
            left.extend(right);
            ExtValue::from_text(V::Array(left).to_string())
        }
        (V::Array(mut left), right) => {
            left.push(right);
            ExtValue::from_text(V::Array(left).to_string())
        }
        (left, V::Array(mut right)) => {
            right.insert(0, left);
            ExtValue::from_text(V::Array(right).to_string())
        }
        (V::Object(mut left), V::Object(right)) if left_map && right_map => {
            left.extend(right);
            ExtValue::from_text(V::Object(left).to_string())
        }
        (_, right) if left_map && !right_entity => ExtValue::error_with_message(format!(
            "TypeError: invalid right operand for ||: {right}"
        )),
        (left, _) if right_map && !left_entity => {
            ExtValue::error_with_message(format!("TypeError: invalid left operand for ||: {left}"))
        }
        (left, right) => ExtValue::from_text(V::Array(vec![left, right]).to_string()),
    }
}

/// Cypher `/` over dynamically typed operands: integer division truncates
/// and raises on a zero divisor, mixed/float division divides as doubles,
/// null propagates.
#[scalar(name = "cypher_div")]
fn cypher_div(args: &[ExtValue]) -> ExtValue {
    cypher_div_impl(args)
}

fn cypher_div_impl(args: &[ExtValue]) -> ExtValue {
    use turso_ext::ValueType;
    let (Some(left), Some(right)) = (args.first(), args.get(1)) else {
        return ExtValue::null();
    };
    if left.value_type() == ValueType::Null || right.value_type() == ValueType::Null {
        return ExtValue::null();
    }
    if left.value_type() == ValueType::Integer && right.value_type() == ValueType::Integer {
        let (Some(left), Some(right)) = (left.to_integer(), right.to_integer()) else {
            return ExtValue::null();
        };
        if right == 0 {
            return ExtValue::error_with_message("ArithmeticError: / by zero".to_owned());
        }
        return ExtValue::from_integer(left.wrapping_div(right));
    }
    let (Some(left), Some(right)) = (left.to_float(), right.to_float()) else {
        return ExtValue::null();
    };
    if right == 0.0 {
        return ExtValue::error_with_message("ArithmeticError: / by zero".to_owned());
    }
    ExtValue::from_float(left / right)
}

/// Cypher `split(text, delimiter)` as a JSON list. An empty delimiter splits
/// at Unicode scalar boundaries; null propagates from either argument.
#[scalar(name = "split")]
fn split(args: &[ExtValue]) -> ExtValue {
    split_impl(args)
}

fn split_impl(args: &[ExtValue]) -> ExtValue {
    use turso_ext::ValueType;
    let (Some(value), Some(delimiter)) = (args.first(), args.get(1)) else {
        return ExtValue::error_with_message("split() requires exactly two arguments".to_owned());
    };
    if value.value_type() == ValueType::Null || delimiter.value_type() == ValueType::Null {
        return ExtValue::null();
    }
    let (Some(value), Some(delimiter)) = (value.to_text(), delimiter.to_text()) else {
        return ExtValue::error_with_message("split() requires text arguments".to_owned());
    };
    let parts = if delimiter.is_empty() {
        value
            .chars()
            .map(|character| character.to_string())
            .collect::<Vec<_>>()
    } else {
        value
            .split(delimiter)
            .map(str::to_owned)
            .collect::<Vec<_>>()
    };
    ExtValue::from_text(serde_json::to_string(&parts).expect("serializing strings cannot fail"))
}

/// Interprets an SQL value for structural comparison: text that parses as
/// a JSON array or object is a list/map, any other text is a plain string,
/// numbers map directly, SQL NULL is Cypher null.
///
/// Known limitation: lists and maps lower to JSON text, so a genuine Cypher
/// string that happens to be valid JSON (`'[]'`) is indistinguishable from a
/// list at runtime and compares structurally. Removing this ambiguity needs
/// a typed Cypher value boundary rather than a change here.
fn comparison_value(value: &ExtValue) -> serde_json::Value {
    use turso_ext::ValueType;
    match value.value_type() {
        ValueType::Null => serde_json::Value::Null,
        ValueType::Integer => serde_json::Value::from(value.to_integer().unwrap_or_default()),
        ValueType::Float => serde_json::Value::from(value.to_float().unwrap_or_default()),
        ValueType::Text => {
            let text = value.to_text().unwrap_or_default();
            match serde_json::from_str::<serde_json::Value>(text) {
                Ok(parsed @ (serde_json::Value::Array(_) | serde_json::Value::Object(_))) => parsed,
                _ => serde_json::Value::String(text.to_owned()),
            }
        }
        _ => serde_json::Value::String(value.to_text_coerced().unwrap_or_default()),
    }
}

/// `Some(true)`/`Some(false)`/`None` per Cypher equality: null makes any
/// comparison uncertain, a definite element mismatch makes a container
/// definitely unequal, and remaining uncertainty propagates outward.
fn deep_equal(left: &serde_json::Value, right: &serde_json::Value) -> Option<bool> {
    use serde_json::Value as V;
    match (left, right) {
        (V::Null, _) | (_, V::Null) => None,
        (V::Array(left), V::Array(right)) => {
            if left.len() != right.len() {
                return Some(false);
            }
            let mut uncertain = false;
            for (left, right) in left.iter().zip(right) {
                match deep_equal(left, right) {
                    Some(false) => return Some(false),
                    None => uncertain = true,
                    Some(true) => {}
                }
            }
            if uncertain {
                None
            } else {
                Some(true)
            }
        }
        (V::Object(left), V::Object(right)) => {
            if left.len() != right.len() || left.keys().any(|key| !right.contains_key(key)) {
                return Some(false);
            }
            let mut uncertain = false;
            for (key, value) in left {
                match deep_equal(value, &right[key]) {
                    Some(false) => return Some(false),
                    None => uncertain = true,
                    Some(true) => {}
                }
            }
            if uncertain {
                None
            } else {
                Some(true)
            }
        }
        (V::Number(left), V::Number(right)) => {
            Some(left.as_f64().unwrap_or(f64::NAN) == right.as_f64().unwrap_or(f64::NAN))
        }
        (V::Bool(left), V::Bool(right)) => Some(left == right),
        (V::String(left), V::String(right)) => Some(left == right),
        _ => Some(false),
    }
}

fn render_json(value: &serde_json::Value) -> ExtValue {
    match value {
        serde_json::Value::Null => ExtValue::null(),
        other => ExtValue::from_text(other.to_string()),
    }
}

fn json_index<'a>(
    container: &'a serde_json::Value,
    key: &ExtValue,
) -> Option<&'a serde_json::Value> {
    match container {
        serde_json::Value::Object(map) => map.get(&text(key)?),
        serde_json::Value::Array(items) => {
            let index = key.to_integer()?;
            let index = if index < 0 {
                items.len().checked_sub(index.unsigned_abs() as usize)?
            } else {
                index as usize
            };
            items.get(index)
        }
        _ => None,
    }
}

/// `a -> b`: object field or (possibly negative) array index, as JSON.
#[scalar(name = "jsonb_get")]
fn jsonb_get(args: &[ExtValue]) -> ExtValue {
    jsonb_get_impl(args)
}

fn jsonb_get_impl(args: &[ExtValue]) -> ExtValue {
    let (Some(container), Some(key)) = (args.first().and_then(json_argument), args.get(1)) else {
        return ExtValue::null();
    };
    json_index(&container, key).map_or_else(ExtValue::null, render_json)
}

/// `a ->> b`: like `->` but scalar results render as bare text.
#[scalar(name = "jsonb_get_text")]
fn jsonb_get_text(args: &[ExtValue]) -> ExtValue {
    jsonb_get_text_impl(args)
}

fn jsonb_get_text_impl(args: &[ExtValue]) -> ExtValue {
    let (Some(container), Some(key)) = (args.first().and_then(json_argument), args.get(1)) else {
        return ExtValue::null();
    };
    match json_index(&container, key) {
        Some(serde_json::Value::String(text)) => ExtValue::from_text(text.clone()),
        Some(other) if !other.is_null() => ExtValue::from_text(other.to_string()),
        _ => ExtValue::null(),
    }
}

/// `a #> path`: extract along a JSON array of keys/indices.
#[scalar(name = "jsonb_get_path")]
fn jsonb_get_path(args: &[ExtValue]) -> ExtValue {
    jsonb_get_path_impl(args)
}

fn jsonb_get_path_impl(args: &[ExtValue]) -> ExtValue {
    let (Some(container), Some(serde_json::Value::Array(path))) = (
        args.first().and_then(json_argument),
        args.get(1).and_then(json_argument),
    ) else {
        return ExtValue::null();
    };
    let mut current = container;
    for step in path {
        let next = match (&current, &step) {
            (serde_json::Value::Object(map), serde_json::Value::String(key)) => {
                map.get(key).cloned()
            }
            (serde_json::Value::Array(items), serde_json::Value::Number(index)) => index
                .as_i64()
                .and_then(|index| {
                    if index < 0 {
                        items.len().checked_sub(index.unsigned_abs() as usize)
                    } else {
                        Some(index as usize)
                    }
                })
                .and_then(|index| items.get(index).cloned()),
            _ => None,
        };
        match next {
            Some(value) => current = value,
            None => return ExtValue::null(),
        }
    }
    render_json(&current)
}

fn object_or_array_keys(value: &serde_json::Value) -> Vec<String> {
    match value {
        serde_json::Value::Object(map) => map.keys().cloned().collect(),
        serde_json::Value::Array(items) => items
            .iter()
            .filter_map(|item| item.as_str().map(str::to_owned))
            .collect(),
        _ => Vec::new(),
    }
}

/// `a ? key`: object key or string array element existence.
#[scalar(name = "jsonb_exists")]
fn jsonb_exists(args: &[ExtValue]) -> ExtValue {
    jsonb_exists_impl(args)
}

fn jsonb_exists_impl(args: &[ExtValue]) -> ExtValue {
    let (Some(container), Some(key)) = (
        args.first().and_then(json_argument),
        args.get(1).and_then(text),
    ) else {
        return ExtValue::null();
    };
    ExtValue::from_integer(object_or_array_keys(&container).contains(&key) as i64)
}

fn exists_over(args: &[ExtValue], all: bool) -> ExtValue {
    let (Some(container), Some(serde_json::Value::Array(keys))) = (
        args.first().and_then(json_argument),
        args.get(1).and_then(json_argument),
    ) else {
        return ExtValue::null();
    };
    let present = object_or_array_keys(&container);
    let mut candidates = keys
        .iter()
        .filter_map(|key| key.as_str())
        .map(|key| present.iter().any(|have| have == key));
    let result = if all {
        candidates.all(|found| found)
    } else {
        candidates.any(|found| found)
    };
    ExtValue::from_integer(result as i64)
}

/// `a ?| keys`: any key present.
#[scalar(name = "jsonb_exists_any")]
fn jsonb_exists_any(args: &[ExtValue]) -> ExtValue {
    exists_over(args, false)
}

/// `a ?& keys`: every key present.
#[scalar(name = "jsonb_exists_all")]
fn jsonb_exists_all(args: &[ExtValue]) -> ExtValue {
    exists_over(args, true)
}

fn contains_value(container: &serde_json::Value, contained: &serde_json::Value) -> bool {
    match (container, contained) {
        (serde_json::Value::Object(outer), serde_json::Value::Object(inner)) => {
            inner.iter().all(|(key, value)| {
                outer
                    .get(key)
                    .is_some_and(|have| contains_value(have, value))
            })
        }
        (serde_json::Value::Array(outer), serde_json::Value::Array(inner)) => inner
            .iter()
            .all(|value| outer.iter().any(|have| contains_value(have, value))),
        (serde_json::Value::Array(outer), scalar) => {
            outer.iter().any(|have| contains_value(have, scalar))
        }
        (left, right) => left == right,
    }
}

/// `a @> b` (postgres jsonb containment; `<@` swaps the arguments).
#[scalar(name = "jsonb_contains")]
fn jsonb_contains(args: &[ExtValue]) -> ExtValue {
    jsonb_contains_impl(args)
}

fn jsonb_contains_impl(args: &[ExtValue]) -> ExtValue {
    let (Some(container), Some(contained)) = (
        args.first().and_then(json_argument),
        args.get(1).and_then(json_argument),
    ) else {
        return ExtValue::null();
    };
    ExtValue::from_integer(contains_value(&container, &contained) as i64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_and_renders_canonical_iso_durations() {
        let parsed = Dur::parse("P1Y2M3DT4H5M6.000000007S").expect("parses");
        assert_eq!(parsed.months, 14);
        assert_eq!(parsed.days, 3);
        assert_eq!(parsed.seconds, 4 * 3600 + 5 * 60 + 6);
        assert_eq!(parsed.nanos, 7);
        assert_eq!(parsed.render(), "P1Y2M3DT4H5M6.000000007S");
        assert_eq!(Dur::parse("P2W").expect("weeks").days, 14);
        assert_eq!(Dur::default().render(), "PT0S");
    }

    #[test]
    fn renders_fractional_duration_components_canonically() {
        assert_eq!(
            Dur::parse("PT0.400000000S").expect("parses").render(),
            "PT0.4S"
        );
        assert_eq!(Dur::parse("PT-0.4S").expect("parses").render(), "PT-0.4S");
        assert_eq!(
            Dur {
                seconds: -86_400,
                nanos: 100_000_000,
                ..Dur::default()
            }
            .render(),
            "PT-23H-59M-59.9S"
        );
    }

    #[test]
    fn duration_between_preserves_negative_fractional_components() {
        let between = |start: &str, end: &str| {
            duration_between_impl(&[
                ExtValue::from_text(start.to_owned()),
                ExtValue::from_text(end.to_owned()),
                ExtValue::from_text("seconds".to_owned()),
            ])
        };

        assert_eq!(
            between("12:34:54.7", "12:34:54.3").to_text(),
            Some("PT-0.4S")
        );
        assert_eq!(
            between("12:34:54.3", "12:34:54.7").to_text(),
            Some("PT0.4S")
        );
    }

    #[test]
    fn calendar_shift_respects_month_lengths() {
        let start = parse_temporal("2024-01-31T00:00:00Z").expect("parses");
        let shifted = shift_temporal(
            start,
            Dur {
                months: 1,
                ..Dur::default()
            },
            1,
        )
        .expect("shifts");
        assert_eq!(render_temporal(&shifted), "2024-02-29T00:00Z");
    }

    #[test]
    fn renders_reduced_precision_and_zones() {
        let time = parse_temporal("10:35").expect("parses");
        assert_eq!(render_temporal(&time), "10:35");
        let precise = parse_temporal("12:31:14.645876123").expect("parses");
        assert_eq!(render_temporal(&precise), "12:31:14.645876123");
        let zoned = parse_temporal("1984-03-07T12:31:14+01:00[Europe/Stockholm]").expect("parses");
        assert_eq!(
            render_temporal(&zoned),
            "1984-03-07T12:31:14+01:00[Europe/Stockholm]"
        );
    }

    #[test]
    fn parses_extended_date_and_time_string_forms() {
        let date =
            |text: &str| render_temporal(&parse_temporal_with_kind("date", text).expect("parses"));
        assert_eq!(date("2015-07-21"), "2015-07-21");
        assert_eq!(date("20150721"), "2015-07-21");
        assert_eq!(date("2015-07"), "2015-07-01");
        assert_eq!(date("201507"), "2015-07-01");
        assert_eq!(date("2015-W30-2"), "2015-07-21");
        assert_eq!(date("2015W302"), "2015-07-21");
        assert_eq!(date("2015-W30"), "2015-07-20");
        assert_eq!(date("2015-202"), "2015-07-21");
        assert_eq!(date("2015202"), "2015-07-21");
        assert_eq!(date("2015"), "2015-01-01");
        let time = |text: &str| {
            render_temporal(&parse_temporal_with_kind("localtime", text).expect("parses"))
        };
        assert_eq!(time("21:40:32.142"), "21:40:32.142");
        assert_eq!(time("214032.142"), "21:40:32.142");
        assert_eq!(time("2140"), "21:40");
        assert_eq!(time("21"), "21:00");
    }

    #[test]
    fn builds_week_ordinal_and_quarter_dates() {
        let map = |json: &str| -> serde_json::Map<String, serde_json::Value> {
            match serde_json::from_str(json).expect("valid json") {
                serde_json::Value::Object(map) => map,
                _ => unreachable!(),
            }
        };
        let render = |kind: &str, json: &str| {
            render_temporal(&build_temporal(kind, &map(json)).expect("builds"))
        };
        assert_eq!(
            render("date", r#"{"year": 1984, "week": 10, "dayOfWeek": 3}"#),
            "1984-03-07"
        );
        assert_eq!(
            render("date", r#"{"year": 1984, "ordinalDay": 202}"#),
            "1984-07-20"
        );
        assert_eq!(
            render(
                "date",
                r#"{"year": 1984, "quarter": 3, "dayOfQuarter": 45}"#
            ),
            "1984-08-14"
        );
        assert_eq!(
            render(
                "datetime",
                r#"{"year": 1984, "month": 3, "day": 7, "hour": 12, "nanosecond": 645876123, "timezone": "Europe/Stockholm"}"#
            ),
            "1984-03-07T12:00:00.645876123+01:00[Europe/Stockholm]"
        );
        assert_eq!(
            render("datetime", r#"{"date": "1984-03-07", "time": "12:31:14"}"#),
            "1984-03-07T12:31:14Z"
        );
    }

    #[test]
    fn split_preserves_empty_parts_and_unicode_characters() {
        let split_text = |value: &str, delimiter: &str| {
            split_impl(&[
                ExtValue::from_text(value.to_owned()),
                ExtValue::from_text(delimiter.to_owned()),
            ])
        };
        assert_eq!(split_text("a  b", " ").to_text(), Some(r#"["a","","b"]"#));
        assert_eq!(split_text("aé", "").to_text(), Some(r#"["a","é"]"#));
        assert_eq!(split_text("aaa", "aa").to_text(), Some(r#"["","a"]"#));

        let null = split_impl(&[ExtValue::null(), ExtValue::from_text(",".to_owned())]);
        assert_eq!(null.value_type(), turso_ext::ValueType::Null);
        let invalid = split_impl(&[
            ExtValue::from_integer(1),
            ExtValue::from_text(",".to_owned()),
        ]);
        assert_eq!(invalid.value_type(), turso_ext::ValueType::Error);
    }

    #[test]
    fn dynamic_arithmetic_recovers_temporal_and_duration_values() {
        let text = |value: &str| ExtValue::from_text(value.to_owned());

        assert_eq!(
            cypher_add_impl(&[text("1984-10-11"), text("P1M")]).to_text(),
            Some("1984-11-11")
        );
        assert_eq!(
            cypher_sub_impl(&[text("1984-10-11"), text("P1M")]).to_text(),
            Some("1984-09-11")
        );
        assert_eq!(
            cypher_add_impl(&[text("P1Y2D"), text("P2M3D")]).to_text(),
            Some("P1Y2M5D")
        );
        assert_eq!(
            cypher_sub_impl(&[text("P1Y2D"), text("P2M3D")]).to_text(),
            Some("P10M-1D")
        );
    }
}

#[cfg(test)]
mod dispatch_tests {
    use super::*;

    #[test]
    fn dispatch_covers_every_registered_name() {
        for name in FUNCTION_NAMES {
            assert!(
                dispatch(name, &[]).is_some(),
                "{name} must dispatch (even if the empty-args result is an error value)"
            );
        }
        assert!(dispatch("no_such_function", &[]).is_none());
    }

    #[test]
    fn dispatch_matches_scalar_behavior() {
        let args = vec![ExtValue::from_text("P1DT25H".to_string())];
        let out = dispatch("duration_parse", &args).expect("known name");
        // duration_parse normalizes but must not carry fields across:
        // P1DT25H keeps 25 hours (module doc, lib.rs:8).
        assert_eq!(out.to_text(), Some("P1DT25H"));
    }

    #[test]
    fn agtype_concatenation_distinguishes_maps_from_entity_objects() {
        let concat = |left: ExtValue, right: ExtValue, left_kind: i64, right_kind: i64| {
            cypher_concat_impl(&[
                left,
                right,
                ExtValue::from_integer(left_kind),
                ExtValue::from_integer(right_kind),
            ])
            .to_text()
            .expect("concatenation returns JSON text")
            .to_owned()
        };

        assert_eq!(
            concat(
                ExtValue::from_text("[1,2]".to_owned()),
                ExtValue::from_integer(2),
                0,
                0
            ),
            "[1,2,2]"
        );
        assert_eq!(
            concat(ExtValue::from_integer(1), ExtValue::from_integer(0), 0, 0),
            "[1,0]"
        );
        assert_eq!(
            concat(
                ExtValue::from_text(r#"{"a":1}"#.to_owned()),
                ExtValue::from_text(r#"{"a":2,"b":3}"#.to_owned()),
                1,
                1
            ),
            r#"{"a":2,"b":3}"#
        );
        assert_eq!(
            concat(
                ExtValue::from_text(r#"{"properties":{}}"#.to_owned()),
                ExtValue::from_text(r#"{"properties":{}}"#.to_owned()),
                3,
                3
            ),
            r#"[{"properties":{}},{"properties":{}}]"#
        );
    }
}
