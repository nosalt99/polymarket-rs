use chrono::{DateTime, Utc};
use rust_decimal::prelude::FromPrimitive;
use rust_decimal::Decimal;
use serde::{Deserialize, Deserializer};
use std::fmt::Display;
use std::str::FromStr;

/// Deserialize a number from either a string or number
pub fn deserialize_number_from_string<'de, T, D>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: FromStr + serde::Deserialize<'de>,
    <T as FromStr>::Err: Display,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum StringOrInt<T> {
        String(String),
        Number(T),
    }

    match StringOrInt::<T>::deserialize(deserializer)? {
        StringOrInt::String(s) => s.parse::<T>().map_err(serde::de::Error::custom),
        StringOrInt::Number(i) => Ok(i),
    }
}

/// Deserialize Decimal from JSON number (f64/int) or string
pub fn deserialize_decimal<'de, D>(deserializer: D) -> Result<Decimal, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum Repr {
        Str(String),
        F64(f64),
        U64(u64),
        I64(i64),
    }

    match Repr::deserialize(deserializer)? {
        Repr::Str(s) => Decimal::from_str(&s).map_err(serde::de::Error::custom),
        Repr::F64(f) => {
            Decimal::from_f64(f).ok_or_else(|| serde::de::Error::custom("invalid f64 for Decimal"))
        }
        Repr::U64(u) => Ok(Decimal::from(u)),
        Repr::I64(i) => Ok(Decimal::from(i)),
    }
}

/// Deserialize Option<DateTime<Utc>> from an optional datetime string
/// Supports multiple formats:
/// - RFC3339: "2022-07-27T14:41:12.085+00:00" or "2022-07-27T14:41:12.085Z"
/// - PostgreSQL: "2022-07-27 14:41:12.085+00"
/// - Date only: "2022-07-27" (assumes 00:00:00 UTC)
/// - Empty strings are treated as None
pub fn deserialize_optional_datetime<'de, D>(
    deserializer: D,
) -> Result<Option<DateTime<Utc>>, D::Error>
where
    D: Deserializer<'de>,
{
    use chrono::NaiveDate;

    let opt: Option<String> = Option::deserialize(deserializer)?;
    match opt {
        Some(s) => {
            let s = s.trim();

            // Treat empty strings as None
            if s.is_empty() {
                return Ok(None);
            }

            // Try RFC3339 first
            if let Ok(dt) = DateTime::parse_from_rfc3339(&s) {
                return Ok(Some(dt.with_timezone(&Utc)));
            }

            // Try PostgreSQL format: "2022-07-27 14:41:12.085+00"
            // Convert to RFC3339 by replacing space with T and fixing timezone
            let mut rfc3339_attempt = s.replace(" ", "T");

            // Fix timezone format: +00 -> +00:00, -00 -> -00:00
            if rfc3339_attempt.ends_with("+00") {
                rfc3339_attempt = rfc3339_attempt.replace("+00", "+00:00");
            } else if rfc3339_attempt.ends_with("-00") {
                rfc3339_attempt = rfc3339_attempt.replace("-00", "-00:00");
            }

            if let Ok(dt) = DateTime::parse_from_rfc3339(&rfc3339_attempt) {
                return Ok(Some(dt.with_timezone(&Utc)));
            }

            // Try date-only format: "2022-07-27"
            if let Ok(date) = NaiveDate::parse_from_str(&s, "%Y-%m-%d") {
                let dt = date
                    .and_hms_opt(0, 0, 0)
                    .ok_or_else(|| serde::de::Error::custom("invalid date"))?
                    .and_utc();
                return Ok(Some(dt));
            }

            Err(serde::de::Error::custom(format!(
                "failed to parse datetime: {}",
                s
            )))
        }
        None => Ok(None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;

    #[derive(Deserialize)]
    struct TestStruct {
        #[serde(deserialize_with = "deserialize_optional_datetime")]
        date: Option<DateTime<Utc>>,
    }

    #[test]
    fn test_deserialize_optional_datetime_z_suffix() {
        let json = r#"{"date": "2025-12-09T00:30:00Z"}"#;
        let result: TestStruct = serde_json::from_str(json).unwrap();
        assert!(result.date.is_some());
        assert_eq!(
            result.date.unwrap().to_rfc3339(),
            "2025-12-09T00:30:00+00:00"
        );
    }

    #[test]
    fn test_deserialize_optional_datetime_z_suffix_microseconds() {
        let json = r#"{"date": "2025-12-08T00:34:34.047523Z"}"#;
        let result: TestStruct = serde_json::from_str(json).unwrap();
        assert!(result.date.is_some());
        assert_eq!(
            result.date.unwrap().to_rfc3339(),
            "2025-12-08T00:34:34.047523+00:00"
        );
    }

    #[test]
    fn test_deserialize_optional_datetime_postgres_format() {
        let json = r#"{"date": "2025-10-23 05:00:35+00"}"#;
        let result: TestStruct = serde_json::from_str(json).unwrap();
        assert!(result.date.is_some());
        assert_eq!(
            result.date.unwrap().to_rfc3339(),
            "2025-10-23T05:00:35+00:00"
        );
    }

    #[test]
    fn test_deserialize_optional_datetime_date_only() {
        let json = r#"{"date": "2025-10-23"}"#;
        let result: TestStruct = serde_json::from_str(json).unwrap();
        assert!(result.date.is_some());
        assert_eq!(
            result.date.unwrap().to_rfc3339(),
            "2025-10-23T00:00:00+00:00"
        );
    }
}
