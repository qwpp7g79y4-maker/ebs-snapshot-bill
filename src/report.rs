use crate::bill::{by_volume, Bill};
use crate::error::clean;
use crate::Error;
use serde_json::json;

/// A text-table field: `clean`, and a leading `=`, `+`, `-` or `@` gets an apostrophe so a
/// spreadsheet does not read it as a formula.
fn field(s: &str) -> String {
    let c = clean(s);
    if c.starts_with(['=', '+', '-', '@']) {
        format!("'{c}")
    } else {
        c
    }
}

pub fn render_text(bill: &Bill) -> Result<String, Error> {
    let currency = clean(&bill.currency);
    let mut out =
        String::from("snapshot_id\tvolume_id\tsize_gb\tage_days\ttier\torphan\told\tcost\n");
    for r in &bill.rows {
        let tier = if r.archive { "archive" } else { "standard" };
        out.push_str(&format!(
            "{}\t{}\t{}\t{}\t{tier}\t{}\t{}\t{} {}\n",
            field(&r.snapshot_id),
            field(&r.volume_id),
            r.size_gb,
            r.age_days,
            r.orphan,
            r.old,
            r.cost,
            currency
        ));
    }
    out.push_str(&format!("total_gb\t{}\n", bill.total_gb));
    out.push_str(&format!("total_cost\t{} {}\n", bill.total_cost, currency));
    if !bill.orphans.is_empty() {
        out.push_str(&format!("orphans\t{}\n", join(&bill.orphans)));
    }
    if !bill.old.is_empty() {
        out.push_str(&format!("old\t{}\n", join(&bill.old)));
    }
    out.push_str("by_volume\n");
    for (vol, (gb, cost)) in by_volume(bill)? {
        out.push_str(&format!("{}\t{gb}\t{} {}\n", field(&vol), cost, currency));
    }
    Ok(out)
}

fn join(ids: &[String]) -> String {
    ids.iter().map(|i| field(i)).collect::<Vec<_>>().join(",")
}

pub fn render_json(bill: &Bill) -> Result<String, Error> {
    let rows: Vec<_> = bill
        .rows
        .iter()
        .map(|r| {
            json!({
                "snapshot_id": r.snapshot_id,
                "volume_id": r.volume_id,
                "size_gb": r.size_gb,
                "age_days": r.age_days,
                "archive": r.archive,
                "orphan": r.orphan,
                "old": r.old,
                "cost": r.cost.to_string(),
            })
        })
        .collect();
    let vols: Vec<_> = by_volume(bill)?
        .into_iter()
        .map(|(id, (gb, cost))| {
            json!({
                "volume_id": id, "size_gb": gb, "cost": cost.to_string()
            })
        })
        .collect();
    let doc = json!({
        "schema_version": 1,
        "tool": "ebs-snapshot-bill",
        "currency": bill.currency,
        "total_gb": bill.total_gb,
        "total_cost": bill.total_cost.to_string(),
        "orphans": bill.orphans,
        "old": bill.old,
        "snapshots": rows,
        "by_volume": vols,
    });
    Ok(format!(
        "{}\n",
        serde_json::to_string_pretty(&doc).unwrap_or_default()
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bill::bill_snapshots;
    use crate::parse::Snapshot;
    use crate::price::Prices;
    #[test]
    fn text_and_json() {
        let bill = bill_snapshots(
            &[Snapshot {
                snapshot_id: "snap-1".into(),
                volume_id: Some("vol-1".into()),
                volume_size_gb: 8,
                start_time: "2026-01-01".into(),
                archive: false,
                description: String::new(),
            }],
            &Prices::builtin(),
            "2026-11-09",
            365,
            None,
        )
        .unwrap();
        let t = render_text(&bill).unwrap();
        assert!(t.contains("snap-1"));
        assert!(t.contains("0.40"));
        let v: serde_json::Value = serde_json::from_str(&render_json(&bill).unwrap()).unwrap();
        assert_eq!(v["schema_version"], 1);
        assert_eq!(v["tool"], "ebs-snapshot-bill");
    }
}
