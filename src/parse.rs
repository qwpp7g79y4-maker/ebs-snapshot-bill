//! Parse `describe-snapshots` JSON.
//! Fields: <https://docs.aws.amazon.com/AWSEC2/latest/APIReference/API_DescribeSnapshots.html>
//! (geraadpleegd 2026-11-09).

use crate::Error;
use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Snapshot {
    pub snapshot_id: String,
    pub volume_id: Option<String>,
    pub volume_size_gb: u64,
    pub start_time: String,
    pub archive: bool,
    pub description: String,
}

/// Walks a JSON document and fails on an object that repeats a key. The values are dropped.
struct NoDup;

impl<'de> serde::Deserialize<'de> for NoDup {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        struct V;
        impl<'de> serde::de::Visitor<'de> for V {
            type Value = NoDup;
            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                f.write_str("any JSON value")
            }
            fn visit_bool<E>(self, _: bool) -> Result<NoDup, E> {
                Ok(NoDup)
            }
            fn visit_i64<E>(self, _: i64) -> Result<NoDup, E> {
                Ok(NoDup)
            }
            fn visit_u64<E>(self, _: u64) -> Result<NoDup, E> {
                Ok(NoDup)
            }
            fn visit_f64<E>(self, _: f64) -> Result<NoDup, E> {
                Ok(NoDup)
            }
            fn visit_str<E>(self, _: &str) -> Result<NoDup, E> {
                Ok(NoDup)
            }
            fn visit_unit<E>(self) -> Result<NoDup, E> {
                Ok(NoDup)
            }
            fn visit_seq<A: serde::de::SeqAccess<'de>>(self, mut s: A) -> Result<NoDup, A::Error> {
                while s.next_element::<NoDup>()?.is_some() {}
                Ok(NoDup)
            }
            fn visit_map<A: serde::de::MapAccess<'de>>(self, mut m: A) -> Result<NoDup, A::Error> {
                let mut seen = std::collections::HashSet::new();
                while let Some(key) = m.next_key::<String>()? {
                    if !seen.insert(key.clone()) {
                        return Err(serde::de::Error::custom(format!("duplicate key {key}")));
                    }
                    m.next_value::<NoDup>()?;
                }
                Ok(NoDup)
            }
        }
        d.deserialize_any(V)
    }
}

/// Parse JSON like `serde_json::from_slice`, but refuse an object with a repeated key.
pub(crate) fn parse_strict(bytes: &[u8]) -> Result<Value, serde_json::Error> {
    serde_json::from_slice::<NoDup>(bytes)?;
    serde_json::from_slice(bytes)
}

/// Fail when two snapshots share a SnapshotId (also: the same file given twice).
pub fn check_unique(snaps: &[Snapshot]) -> Result<(), Error> {
    let mut seen = std::collections::HashSet::new();
    for s in snaps {
        if !seen.insert(s.snapshot_id.as_str()) {
            return Err(Error::msg(format!(
                "duplicate SnapshotId {}",
                s.snapshot_id
            )));
        }
    }
    Ok(())
}

pub fn parse_snapshots(bytes: &[u8]) -> Result<Vec<Snapshot>, Error> {
    let v: Value =
        parse_strict(bytes).map_err(|e| Error::msg(format!("input is not JSON: {e}")))?;
    if let Some(arr) = v.get("Snapshots").and_then(Value::as_array) {
        return arr.iter().map(parse_one).collect();
    }
    if v.get("SnapshotId").is_some() {
        return Ok(vec![parse_one(&v)?]);
    }
    Err(Error::msg("JSON mist Snapshots[] of een Snapshot-object"))
}

fn parse_one(v: &Value) -> Result<Snapshot, Error> {
    let snapshot_id = v
        .get("SnapshotId")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();
    if snapshot_id.is_empty() {
        return Err(Error::msg("snapshot mist SnapshotId"));
    }
    let volume_id = v.get("VolumeId").and_then(Value::as_str).and_then(|s| {
        if s.is_empty() || s == "vol-ffffffff" || s == "None" {
            None
        } else {
            Some(s.to_string())
        }
    });
    let volume_size_gb = match v.get("VolumeSize") {
        Some(Value::Number(n)) => n
            .as_u64()
            .ok_or_else(|| Error::msg("VolumeSize ongeldig"))?,
        Some(Value::String(s)) => s
            .parse()
            .map_err(|_| Error::msg(format!("VolumeSize ongeldig: {s}")))?,
        None => {
            return Err(Error::msg(format!(
                "snapshot {snapshot_id} mist VolumeSize"
            )))
        }
        _ => return Err(Error::msg("VolumeSize heeft een ongeldig type")),
    };
    let start_time = v
        .get("StartTime")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();
    if start_time.is_empty() {
        return Err(Error::msg(format!("snapshot {snapshot_id} mist StartTime")));
    }
    let archive = match v.get("StorageTier") {
        Some(Value::String(s)) => s.eq_ignore_ascii_case("archive"),
        Some(Value::Array(a)) => a.iter().any(|x| {
            x.as_str()
                .map(|s| s.eq_ignore_ascii_case("archive"))
                .unwrap_or(false)
        }),
        _ => false,
    };
    let description = v
        .get("Description")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();
    Ok(Snapshot {
        snapshot_id,
        volume_id,
        volume_size_gb,
        start_time,
        archive,
        description,
    })
}

/// One element of a volumes list: a bare id string, or an object with a `VolumeId` string.
struct VolumeElem(Option<String>);

impl<'de> serde::Deserialize<'de> for VolumeElem {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        struct V;
        impl<'de> serde::de::Visitor<'de> for V {
            type Value = VolumeElem;
            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                f.write_str("a volume id string or an object with a VolumeId string")
            }
            fn visit_str<E>(self, s: &str) -> Result<VolumeElem, E> {
                Ok(VolumeElem(Some(s.to_string())))
            }
            fn visit_map<A: serde::de::MapAccess<'de>>(
                self,
                mut m: A,
            ) -> Result<VolumeElem, A::Error> {
                let mut id = None;
                let mut seen = false;
                while let Some(key) = m.next_key::<String>()? {
                    if key == "VolumeId" {
                        if seen {
                            return Err(serde::de::Error::custom("duplicate VolumeId key"));
                        }
                        seen = true;
                        if let Value::String(s) = m.next_value::<Value>()? {
                            id = Some(s);
                        }
                    } else {
                        m.next_value::<serde::de::IgnoredAny>()?;
                    }
                }
                Ok(VolumeElem(id))
            }
        }
        d.deserialize_any(V)
    }
}

/// A volumes file: a list of elements, or an object whose `Volumes` is such a list.
struct VolumeFile(Vec<VolumeElem>);

impl<'de> serde::Deserialize<'de> for VolumeFile {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        struct V;
        impl<'de> serde::de::Visitor<'de> for V {
            type Value = VolumeFile;
            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                f.write_str("a list of volumes or an object with Volumes[]")
            }
            fn visit_seq<A: serde::de::SeqAccess<'de>>(
                self,
                mut s: A,
            ) -> Result<VolumeFile, A::Error> {
                let mut v = Vec::new();
                while let Some(e) = s.next_element::<VolumeElem>()? {
                    v.push(e);
                }
                Ok(VolumeFile(v))
            }
            fn visit_map<A: serde::de::MapAccess<'de>>(
                self,
                mut m: A,
            ) -> Result<VolumeFile, A::Error> {
                let mut list = None;
                while let Some(key) = m.next_key::<String>()? {
                    if key == "Volumes" {
                        if list.is_some() {
                            return Err(serde::de::Error::custom("duplicate Volumes key"));
                        }
                        list = Some(m.next_value::<VolumeFile>()?.0);
                    } else {
                        m.next_value::<serde::de::IgnoredAny>()?;
                    }
                }
                list.map(VolumeFile)
                    .ok_or_else(|| serde::de::Error::custom("missing Volumes[]"))
            }
        }
        d.deserialize_any(V)
    }
}

/// Parse the optional `--volumes` file. Accepted: `{"Volumes":[{"VolumeId":"vol-..."}]}`, a top-level
/// list `[{"VolumeId":"vol-..."}]` (what `describe-volumes --query 'Volumes[]'` prints), or a list
/// of id strings. Anything else is an error: an empty list, an element without a string `VolumeId`,
/// a repeated `VolumeId` or `Volumes` key. A list that is not understood must not make every
/// snapshot an orphan.
pub fn parse_volume_ids(bytes: &[u8]) -> Result<std::collections::BTreeSet<String>, Error> {
    let file: VolumeFile = serde_json::from_slice(bytes)
        .map_err(|e| Error::msg(format!("volumes file is not a volume list: {e}")))?;
    if file.0.is_empty() {
        return Err(Error::msg("volumes file lists no volumes"));
    }
    let mut set = std::collections::BTreeSet::new();
    for (i, e) in file.0.into_iter().enumerate() {
        match e.0 {
            Some(id) => {
                set.insert(id);
            }
            None => {
                return Err(Error::msg(format!(
                    "volumes file element {} has no string VolumeId",
                    i + 1
                )))
            }
        }
    }
    Ok(set)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parses_list() {
        let s = parse_snapshots(br#"{"Snapshots":[{"SnapshotId":"snap-1","VolumeId":"vol-1","VolumeSize":8,"StartTime":"2026-01-01T00:00:00.000Z"}]}"#).unwrap();
        assert_eq!(s[0].snapshot_id, "snap-1");
        assert_eq!(s[0].volume_size_gb, 8);
        assert!(!s[0].archive);
    }
    #[test]
    fn orphan_volume() {
        let s = parse_snapshots(br#"{"Snapshots":[{"SnapshotId":"snap-1","VolumeId":"vol-ffffffff","VolumeSize":1,"StartTime":"2026-01-01T00:00:00Z"}]}"#).unwrap();
        assert!(s[0].volume_id.is_none());
    }
    #[test]
    fn archive_tier() {
        let s = parse_snapshots(br#"{"Snapshots":[{"SnapshotId":"snap-1","VolumeId":"vol-1","VolumeSize":1,"StartTime":"2026-01-01T00:00:00Z","StorageTier":"archive"}]}"#).unwrap();
        assert!(s[0].archive);
    }
    #[test]
    fn broken_json() {
        assert!(parse_snapshots(b"{").is_err());
    }
    #[test]
    fn missing_size() {
        assert!(
            parse_snapshots(br#"{"Snapshots":[{"SnapshotId":"snap-1","StartTime":"t"}]}"#).is_err()
        );
    }
    #[test]
    fn volume_ids() {
        let ids = parse_volume_ids(br#"{"Volumes":[{"VolumeId":"vol-abc"}]}"#).unwrap();
        assert!(ids.contains("vol-abc"));
    }
    #[test]
    fn volume_id_array() {
        let ids = parse_volume_ids(br#"["vol-a","vol-b"]"#).unwrap();
        assert_eq!(ids.len(), 2);
    }
}
