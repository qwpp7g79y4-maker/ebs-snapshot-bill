use crate::Error;
use rust_decimal::Decimal;
use serde_json::Value;
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Prices {
    pub currency: String,
    pub region: String,
    pub source: String,
    pub date: String,
    pub standard_per_gb_month: Decimal,
    pub archive_per_gb_month: Decimal,
}

impl Prices {
    pub fn builtin() -> Self {
        Self {
            currency: "USD".into(),
            region: "us-east-1".into(),
            source: "https://aws.amazon.com/ebs/pricing/".into(),
            date: "2026-11-09".into(),
            standard_per_gb_month: Decimal::from_str("0.05").unwrap(),
            archive_per_gb_month: Decimal::from_str("0.0125").unwrap(),
        }
    }
    pub fn from_json(bytes: &[u8]) -> Result<Self, Error> {
        let v: Value = crate::parse::parse_strict(bytes)
            .map_err(|e| Error::msg(format!("price file is not JSON: {e}")))?;
        let p = Self {
            currency: s(&v, "currency")?,
            region: s(&v, "region")?,
            source: s(&v, "source")?,
            date: s(&v, "date")?,
            standard_per_gb_month: d(&v, "snapshot_standard_per_gb_month")?,
            archive_per_gb_month: d(&v, "snapshot_archive_per_gb_month")?,
        };
        if p.standard_per_gb_month.is_sign_negative() || p.archive_per_gb_month.is_sign_negative() {
            return Err(Error::msg("price file rate is below 0"));
        }
        Ok(p)
    }
    pub fn rate(&self, archive: bool) -> Decimal {
        if archive {
            self.archive_per_gb_month
        } else {
            self.standard_per_gb_month
        }
    }
}
fn s(v: &Value, k: &str) -> Result<String, Error> {
    v.get(k)
        .and_then(Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| Error::msg(format!("price file mist {k}")))
}
fn d(v: &Value, k: &str) -> Result<Decimal, Error> {
    Decimal::from_str(&s(v, k)?).map_err(|_| Error::msg(format!("price file {k} is not a decimal")))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn builtin() {
        let p = Prices::builtin();
        assert_eq!(p.standard_per_gb_month.to_string(), "0.05");
        assert_eq!(p.archive_per_gb_month.to_string(), "0.0125");
        assert!(p.source.contains("ebs/pricing"));
        assert_eq!(p.date, "2026-11-09");
    }
    #[test]
    fn file() {
        let p = Prices::from_json(include_bytes!("../prices/us-east-1.json")).unwrap();
        assert_eq!(p.region, "us-east-1");
    }
    #[test]
    fn bad_json() {
        assert!(Prices::from_json(b"{").is_err());
    }
    fn json_with(std: &str, arc: &str) -> String {
        format!(
            r#"{{"currency":"USD","region":"r","source":"s","date":"d","snapshot_standard_per_gb_month":"{std}","snapshot_archive_per_gb_month":"{arc}"}}"#
        )
    }
    #[test]
    fn negative_standard_rejected() {
        assert!(Prices::from_json(json_with("-0.05", "0.0125").as_bytes()).is_err());
    }
    #[test]
    fn negative_archive_rejected() {
        assert!(Prices::from_json(json_with("0.05", "-0.01").as_bytes()).is_err());
    }
    #[test]
    fn zero_rate_ok() {
        assert!(Prices::from_json(json_with("0", "0").as_bytes()).is_ok());
    }
    #[test]
    fn missing() {
        assert!(Prices::from_json(br#"{"currency":"USD"}"#).is_err());
    }
}
