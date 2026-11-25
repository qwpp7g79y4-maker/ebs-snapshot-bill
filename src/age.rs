//! Very small RFC3339 / AWS date parser for age in whole days.
use crate::Error;

/// Parse `YYYY-MM-DD` or `YYYY-MM-DDTHH:MM:SS...` into days since unix epoch (UTC, floor).
pub fn days_since_epoch(text: &str) -> Result<i64, Error> {
    let date = text
        .get(..10)
        .ok_or_else(|| Error::msg(format!("StartTime te kort: {text}")))?;
    let mut parts = date.split('-');
    let y: i64 = parts
        .next()
        .and_then(|s| s.parse().ok())
        .ok_or_else(|| Error::msg(format!("ongeldige StartTime: {text}")))?;
    let m: i64 = parts
        .next()
        .and_then(|s| s.parse().ok())
        .ok_or_else(|| Error::msg(format!("ongeldige StartTime: {text}")))?;
    let d: i64 = parts
        .next()
        .and_then(|s| s.parse().ok())
        .ok_or_else(|| Error::msg(format!("ongeldige StartTime: {text}")))?;
    if !(1..=12).contains(&m) || !(1..=31).contains(&d) {
        return Err(Error::msg(format!("ongeldige StartTime: {text}")));
    }
    Ok(days_from_civil(y, m, d))
}

/// Howard Hinnant civil_from_days inverse (days from 1970-01-01).
fn days_from_civil(y: i64, m: i64, d: i64) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = y - era * 400;
    let mp = if m > 2 { m - 3 } else { m + 9 };
    let doy = (153 * mp + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146097 + doe - 719468
}

pub fn age_days(start: &str, as_of: &str) -> Result<u64, Error> {
    let a = days_since_epoch(start)?;
    let b = days_since_epoch(as_of)?;
    Ok((b - a).max(0) as u64)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn epoch() {
        assert_eq!(days_since_epoch("1970-01-01").unwrap(), 0);
    }
    #[test]
    fn known() {
        assert_eq!(
            days_since_epoch("2026-11-09").unwrap(),
            days_since_epoch("2026-11-09T12:00:00Z").unwrap()
        );
    }
    #[test]
    fn age() {
        assert_eq!(age_days("2026-01-01", "2026-01-31").unwrap(), 30);
    }
    #[test]
    fn bad() {
        assert!(days_since_epoch("nope").is_err());
    }
    #[test]
    fn future_age_zero() {
        assert_eq!(age_days("2026-12-01", "2026-11-01").unwrap(), 0);
    }
}
