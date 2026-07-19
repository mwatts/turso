//! Cypher duration values as a statically registered scalar extension.
//!
//! Durations are month/day/second/nanosecond component vectors encoded as
//! canonical ISO-8601 text (`P1Y2M3DT4H5M6.000000007S`). The arithmetic is
//! implemented in Rust — `chrono` calendar types for month/day steps and
//! plain integer seconds/nanoseconds for the time part — and registered on
//! a connection through the engine's static-extension mechanism.

use chrono::{DateTime, Days, Months, Utc};
use turso_core::Connection;
use turso_ext::{scalar, ExtensionApi, Value as ExtValue};

/// Registers the duration functions on a connection. Safe to call more
/// than once; later registrations replace the earlier entries.
pub fn install_duration_extension(connection: &Connection) {
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
        register(c"datetime_add_duration".as_ptr(), datetime_add_duration);
        register(c"datetime_sub_duration".as_ptr(), datetime_sub_duration);
    });
}

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
            let hours = normalized.seconds / 3600;
            let minutes = (normalized.seconds % 3600) / 60;
            let seconds = normalized.seconds % 60;
            if hours != 0 {
                out.push_str(&format!("{hours}H"));
            }
            if minutes != 0 {
                out.push_str(&format!("{minutes}M"));
            }
            if seconds != 0 || normalized.nanos != 0 || (hours == 0 && minutes == 0) {
                if normalized.nanos == 0 {
                    out.push_str(&format!("{seconds}S"));
                } else {
                    out.push_str(&format!("{seconds}.{:09}S", normalized.nanos));
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
            Some((seconds, if seconds < 0 { -nanos } else { nanos }))
        }
    }
}

fn integer(value: &ExtValue) -> Option<i64> {
    value.to_integer()
}

fn text(value: &ExtValue) -> Option<String> {
    value.to_text().map(|text| text.to_owned())
}

fn duration_value(value: &ExtValue) -> Option<Dur> {
    Dur::parse(&text(value)?)
}

#[scalar(name = "duration_make")]
fn duration_make(args: &[ExtValue]) -> ExtValue {
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
    match args.first().and_then(duration_value) {
        Some(value) => ExtValue::from_text(value.render()),
        None => ExtValue::null(),
    }
}

#[scalar(name = "duration_get")]
fn duration_get(args: &[ExtValue]) -> ExtValue {
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

fn parse_instant(text: &str) -> Option<DateTime<Utc>> {
    let trimmed = text.trim();
    if let Ok(parsed) = DateTime::parse_from_rfc3339(trimmed) {
        return Some(parsed.with_timezone(&Utc));
    }
    let with_zone = format!("{trimmed}Z");
    if let Ok(parsed) = DateTime::parse_from_rfc3339(&with_zone) {
        return Some(parsed.with_timezone(&Utc));
    }
    let with_time = format!("{trimmed}T00:00:00Z");
    DateTime::parse_from_rfc3339(&with_time)
        .ok()
        .map(|parsed| parsed.with_timezone(&Utc))
}

fn shift(instant: DateTime<Utc>, duration: Dur, sign: i64) -> Option<DateTime<Utc>> {
    let months = duration.months * sign;
    let instant = if months >= 0 {
        instant.checked_add_months(Months::new(u32::try_from(months).ok()?))?
    } else {
        instant.checked_sub_months(Months::new(u32::try_from(-months).ok()?))?
    };
    let days = duration.days * sign;
    let instant = if days >= 0 {
        instant.checked_add_days(Days::new(u64::try_from(days).ok()?))?
    } else {
        instant.checked_sub_days(Days::new(u64::try_from(-days).ok()?))?
    };
    let seconds = chrono::Duration::seconds(duration.seconds * sign)
        + chrono::Duration::nanoseconds(duration.nanos * sign);
    instant.checked_add_signed(seconds)
}

fn render_instant(instant: DateTime<Utc>) -> String {
    if instant.timestamp_subsec_nanos() == 0 {
        instant.format("%Y-%m-%dT%H:%M:%SZ").to_string()
    } else {
        instant.format("%Y-%m-%dT%H:%M:%S%.9fZ").to_string()
    }
}

fn shift_datetime(args: &[ExtValue], sign: i64) -> ExtValue {
    let (Some(instant), Some(duration)) = (
        args.first()
            .and_then(text)
            .as_deref()
            .and_then(parse_instant),
        args.get(1).and_then(duration_value),
    ) else {
        return ExtValue::null();
    };
    match shift(instant, duration, sign) {
        Some(shifted) => ExtValue::from_text(render_instant(shifted)),
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

#[scalar(name = "duration_between")]
fn duration_between(args: &[ExtValue]) -> ExtValue {
    let (Some(start), Some(end)) = (
        args.first()
            .and_then(text)
            .as_deref()
            .and_then(parse_instant),
        args.get(1)
            .and_then(text)
            .as_deref()
            .and_then(parse_instant),
    ) else {
        return ExtValue::null();
    };
    let difference = end.signed_duration_since(start);
    let total_nanos = difference.num_nanoseconds().unwrap_or_default();
    let seconds = total_nanos.div_euclid(1_000_000_000);
    let nanos = total_nanos.rem_euclid(1_000_000_000);
    let days = seconds.div_euclid(86_400);
    let seconds = seconds.rem_euclid(86_400);
    ExtValue::from_text(
        Dur {
            months: 0,
            days,
            seconds,
            nanos,
        }
        .render(),
    )
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
    fn calendar_shift_respects_month_lengths() {
        let instant = parse_instant("2024-01-31T00:00:00Z").expect("parses");
        let shifted = shift(
            instant,
            Dur {
                months: 1,
                ..Dur::default()
            },
            1,
        )
        .expect("shifts");
        // chrono clamps to the end of February.
        assert_eq!(render_instant(shifted), "2024-02-29T00:00:00Z");
    }
}
