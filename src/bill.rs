use crate::age::age_days;
use crate::parse::Snapshot;
use crate::price::Prices;
use crate::Error;
use rust_decimal::Decimal;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Row {
    pub snapshot_id: String,
    pub volume_id: String,
    pub size_gb: u64,
    pub age_days: u64,
    pub archive: bool,
    pub orphan: bool,
    pub old: bool,
    pub cost: Decimal,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Bill {
    pub rows: Vec<Row>,
    pub total_gb: u64,
    pub total_cost: Decimal,
    pub currency: String,
    pub orphans: Vec<String>,
    pub old: Vec<String>,
}

pub fn bill_snapshots(
    snaps: &[Snapshot],
    prices: &Prices,
    as_of: &str,
    max_age_days: u64,
    known_volumes: Option<&BTreeSet<String>>,
) -> Result<Bill, Error> {
    let mut rows = Vec::new();
    let mut total_gb = 0u64;
    let mut total_cost = Decimal::ZERO;
    let mut orphans = Vec::new();
    let mut old = Vec::new();
    for s in snaps {
        let age = age_days(&s.start_time, as_of)?;
        let orphan = match (&s.volume_id, known_volumes) {
            (None, _) => true,
            (Some(id), Some(set)) => !set.contains(id),
            (Some(_), None) => false,
        };
        let is_old = age > max_age_days;
        let rate = prices.rate(s.archive);
        let cost = Decimal::from(s.volume_size_gb)
            .checked_mul(rate)
            .ok_or_else(|| {
                Error::msg(format!(
                    "cost of {} does not fit: price too large",
                    s.snapshot_id
                ))
            })?
            .round_dp(6);
        total_gb = total_gb
            .checked_add(s.volume_size_gb)
            .ok_or_else(|| Error::msg("total GB does not fit in 64 bits"))?;
        total_cost = total_cost
            .checked_add(cost)
            .ok_or_else(|| Error::msg("total cost does not fit: price too large"))?;
        if orphan {
            orphans.push(s.snapshot_id.clone());
        }
        if is_old {
            old.push(s.snapshot_id.clone());
        }
        rows.push(Row {
            snapshot_id: s.snapshot_id.clone(),
            volume_id: s.volume_id.clone().unwrap_or_else(|| "(none)".into()),
            size_gb: s.volume_size_gb,
            age_days: age,
            archive: s.archive,
            orphan,
            old: is_old,
            cost,
        });
    }
    rows.sort_by(|a, b| a.snapshot_id.cmp(&b.snapshot_id));
    let bill = Bill {
        rows,
        total_gb,
        total_cost: total_cost.round_dp(6),
        currency: prices.currency.clone(),
        orphans,
        old,
    };
    // The per-volume sums are checked too, so a Bill that came out of here can always be rendered.
    by_volume(&bill)?;
    Ok(bill)
}

pub fn by_volume(bill: &Bill) -> Result<BTreeMap<String, (u64, Decimal)>, Error> {
    let mut map = BTreeMap::new();
    for row in &bill.rows {
        let e = map
            .entry(row.volume_id.clone())
            .or_insert((0u64, Decimal::ZERO));
        e.0 =
            e.0.checked_add(row.size_gb)
                .ok_or_else(|| Error::msg("GB of a volume does not fit in 64 bits"))?;
        e.1 =
            e.1.checked_add(row.cost)
                .ok_or_else(|| Error::msg("cost of a volume does not fit: price too large"))?;
    }
    Ok(map)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse::Snapshot;
    use rust_decimal::Decimal;
    use std::str::FromStr;

    fn snap(id: &str, vol: Option<&str>, gb: u64, start: &str, archive: bool) -> Snapshot {
        Snapshot {
            snapshot_id: id.into(),
            volume_id: vol.map(str::to_string),
            volume_size_gb: gb,
            start_time: start.into(),
            archive,
            description: String::new(),
        }
    }

    #[test]
    fn eight_gb_standard_costs_forty_cents() {
        let bill = bill_snapshots(
            &[snap("snap-a", Some("vol-1"), 8, "2026-01-01", false)],
            &Prices::builtin(),
            "2026-11-09",
            365,
            None,
        )
        .unwrap();
        assert_eq!(bill.total_cost, Decimal::from_str("0.40").unwrap());
    }

    #[test]
    fn archive_cheaper() {
        let bill = bill_snapshots(
            &[snap("snap-a", Some("vol-1"), 8, "2026-01-01", true)],
            &Prices::builtin(),
            "2026-11-09",
            365,
            None,
        )
        .unwrap();
        assert_eq!(bill.total_cost, Decimal::from_str("0.100000").unwrap());
    }

    #[test]
    fn marks_old() {
        let bill = bill_snapshots(
            &[snap("snap-a", Some("vol-1"), 1, "2020-01-01", false)],
            &Prices::builtin(),
            "2026-11-09",
            30,
            None,
        )
        .unwrap();
        assert!(bill.rows[0].old);
        assert_eq!(bill.old, vec!["snap-a"]);
    }

    #[test]
    fn marks_orphan_without_volume() {
        let bill = bill_snapshots(
            &[snap("snap-a", None, 1, "2026-01-01", false)],
            &Prices::builtin(),
            "2026-11-09",
            365,
            None,
        )
        .unwrap();
        assert!(bill.rows[0].orphan);
    }

    #[test]
    fn marks_orphan_missing_from_set() {
        let mut set = BTreeSet::new();
        set.insert("vol-alive".into());
        let bill = bill_snapshots(
            &[snap("snap-a", Some("vol-gone"), 1, "2026-01-01", false)],
            &Prices::builtin(),
            "2026-11-09",
            365,
            Some(&set),
        )
        .unwrap();
        assert!(bill.rows[0].orphan);
    }

    #[test]
    fn by_volume_groups() {
        let bill = bill_snapshots(
            &[
                snap("snap-a", Some("vol-1"), 8, "2026-01-01", false),
                snap("snap-b", Some("vol-1"), 8, "2026-02-01", false),
            ],
            &Prices::builtin(),
            "2026-11-09",
            365,
            None,
        )
        .unwrap();
        let map = by_volume(&bill).unwrap();
        assert_eq!(map["vol-1"].0, 16);
    }

    fn big_prices(rate: &str) -> Prices {
        let mut p = Prices::builtin();
        p.standard_per_gb_month = Decimal::from_str(rate).unwrap();
        p
    }

    #[test]
    fn multiplication_overflow_is_error() {
        let r = bill_snapshots(
            &[snap("snap-a", Some("vol-1"), 8, "2026-01-01", false)],
            &big_prices("79228162514264337593543950335"),
            "2026-11-09",
            365,
            None,
        );
        assert!(r.is_err());
    }

    #[test]
    fn addition_overflow_is_error() {
        let r = bill_snapshots(
            &[
                snap("snap-a", Some("vol-1"), 1, "2026-01-01", false),
                snap("snap-b", Some("vol-1"), 1, "2026-01-01", false),
            ],
            &big_prices("50000000000000000000000000000"),
            "2026-11-09",
            365,
            None,
        );
        assert!(r.is_err());
    }

    #[test]
    fn empty() {
        let bill = bill_snapshots(&[], &Prices::builtin(), "2026-11-09", 30, None).unwrap();
        assert!(bill.rows.is_empty());
    }
}
