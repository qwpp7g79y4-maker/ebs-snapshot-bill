use assert_cmd::Command;
use predicates::prelude::*;
use std::path::PathBuf;
fn bin() -> Command {
    Command::cargo_bin("ebs-snapshot-bill").unwrap()
}
fn fix(n: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(n)
}

#[test]
fn help_ok() {
    let a = bin().arg("--help").assert().success();
    let o = String::from_utf8_lossy(&a.get_output().stdout);
    assert!(o.contains("--max-age-days"));
    assert!(o.contains("--volumes"));
}
#[test]
fn version_ok() {
    bin()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("ebs-snapshot-bill"));
}
#[test]
fn normal_costs() {
    bin()
        .arg(fix("normal.json"))
        .assert()
        .success()
        .stdout(predicate::str::contains("0.40"));
}
fn json_rows(args: &[&std::path::Path], extra: &[&str]) -> serde_json::Value {
    let o = bin().args(args).args(extra).arg("--json").output().unwrap();
    assert!(o.status.success());
    serde_json::from_slice(&o.stdout).unwrap()
}
fn row<'a>(v: &'a serde_json::Value, id: &str) -> &'a serde_json::Value {
    v["snapshots"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["snapshot_id"] == id)
        .unwrap_or_else(|| panic!("no row {id}"))
}
fn ids(v: &serde_json::Value, key: &str) -> Vec<String> {
    v[key]
        .as_array()
        .unwrap()
        .iter()
        .map(|x| x.as_str().unwrap().to_string())
        .collect()
}

#[test]
fn old_flagged() {
    let old = fix("old.json");
    let v = json_rows(&[old.as_path()], &["--max-age-days", "30"]);
    assert_eq!(row(&v, "snap-old1")["old"], true);
    assert_eq!(ids(&v, "old"), vec!["snap-old1"]);
    // Within the limit the same row is not old.
    let v = json_rows(&[old.as_path()], &["--max-age-days", "10000"]);
    assert_eq!(row(&v, "snap-old1")["old"], false);
    assert!(ids(&v, "old").is_empty());
    // Text output: the row itself carries the flag, not just the header.
    let a = bin()
        .args([old.to_str().unwrap(), "--max-age-days", "30"])
        .assert()
        .success();
    let out = String::from_utf8_lossy(&a.get_output().stdout).to_string();
    let line = out.lines().find(|l| l.starts_with("snap-old1\t")).unwrap();
    let cols: Vec<&str> = line.split('\t').collect();
    assert_eq!(cols[5], "false");
    assert_eq!(cols[6], "true");
}
#[test]
fn orphan_flagged() {
    let orphan = fix("orphan.json");
    let vols = fix("volumes.json");
    let v = json_rows(&[orphan.as_path()], &["--volumes", vols.to_str().unwrap()]);
    assert_eq!(row(&v, "snap-orphan1")["orphan"], true);
    assert_eq!(ids(&v, "orphans"), vec!["snap-orphan1"]);
    let a = bin()
        .arg(&orphan)
        .arg("--volumes")
        .arg(&vols)
        .assert()
        .success();
    let out = String::from_utf8_lossy(&a.get_output().stdout).to_string();
    let line = out
        .lines()
        .find(|l| l.starts_with("snap-orphan1\t"))
        .unwrap();
    let cols: Vec<&str> = line.split('\t').collect();
    assert_eq!(cols[5], "true");
    // vol-ffffffff is AWS's placeholder id: an orphan even without a volumes file.
    let v = json_rows(&[orphan.as_path()], &[]);
    assert_eq!(row(&v, "snap-orphan1")["orphan"], true);
}
#[test]
fn volumes_marks_missing() {
    let n = fix("normal.json");
    let o = fix("old.json");
    let vols = fix("volumes.json");
    let v = json_rows(
        &[n.as_path(), o.as_path()],
        &["--volumes", vols.to_str().unwrap()],
    );
    assert_eq!(row(&v, "snap-normal1")["orphan"], false);
    assert_eq!(row(&v, "snap-old1")["orphan"], true);
    let orphans = ids(&v, "orphans");
    assert!(!orphans.contains(&"snap-normal1".to_string()));
    assert!(orphans.contains(&"snap-old1".to_string()));
}
#[test]
fn no_volume_id_is_orphan() {
    let n = fix("novolume.json");
    let v = json_rows(&[n.as_path()], &[]);
    assert_eq!(row(&v, "snap-novol1")["orphan"], true);
    assert_eq!(ids(&v, "orphans"), vec!["snap-novol1"]);
}
#[test]
fn negative_price_exit_2() {
    bin()
        .arg(fix("normal.json"))
        .arg("--prices")
        .arg(fix("prices-negative.json"))
        .assert()
        .failure()
        .code(2);
}
#[test]
fn overflow_price_exit_2() {
    bin()
        .arg(fix("normal.json"))
        .arg("--prices")
        .arg(fix("prices-overflow.json"))
        .assert()
        .failure()
        .code(2)
        .stderr(predicate::str::contains("does not fit"));
}
#[test]
fn json_schema() {
    let o = bin()
        .arg(fix("normal.json"))
        .arg("--json")
        .output()
        .unwrap();
    assert!(o.status.success());
    let v: serde_json::Value = serde_json::from_slice(&o.stdout).unwrap();
    assert_eq!(v["schema_version"], 1);
    assert_eq!(v["tool"], "ebs-snapshot-bill");
}
#[test]
fn broken_exit_2() {
    bin().arg(fix("broken.json")).assert().failure().code(2);
}
#[test]
fn missing_exit_2() {
    bin().arg("/no/snap.json").assert().failure().code(2);
}
#[test]
fn prices_file() {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("prices/us-east-1.json");
    bin()
        .arg(fix("normal.json"))
        .args(["--prices"])
        .arg(p)
        .assert()
        .success();
}
#[test]
fn unknown_flag() {
    bin().arg("--nope").assert().failure().code(2);
}

// ---- eindreview 21462f7: B1 (text escaping), B2 (--volumes shapes), B3 (GB overflow) ----

fn tmp(name: &str, content: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("ebssnap-{}-{name}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let p = dir.join("f.json");
    std::fs::write(&p, content).unwrap();
    p
}
fn snap_json(id: &str, vol: &str, size: &str) -> String {
    format!(
        r#"{{"Snapshots":[{{"SnapshotId":"{id}","VolumeId":"{vol}","VolumeSize":{size},"StartTime":"2026-10-01T00:00:00Z"}}]}}"#
    )
}
fn text_of(args: &[&std::path::Path], extra: &[&str]) -> String {
    let o = bin().args(args).args(extra).output().unwrap();
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    String::from_utf8(o.stdout).unwrap()
}

#[test]
fn text_esc_json_case() {
    // esc.json: a line break in SnapshotId and an ESC in VolumeId.
    let f = tmp(
        "esc",
        &snap_json(r"snap-1\ntotal_cost\t0 USD\nfake", r"vol-\u001b[2J\tx", "8"),
    );
    let out = text_of(&[f.as_path()], &[]);
    assert!(!out.contains('\u{1b}'), "raw ESC in {out:?}");
    assert_eq!(
        out.lines()
            .filter(|l| l.starts_with("total_cost\t"))
            .count(),
        1,
        "{out:?}"
    );
    assert!(!out.lines().any(|l| l.starts_with("fake")), "{out:?}");
    // Header + 1 row + total_gb + total_cost + by_volume + 1 volume row.
    assert_eq!(out.lines().count(), 6, "{out:?}");
    let row = out.lines().nth(1).unwrap();
    assert_eq!(row.split('\t').count(), 8, "{row:?}");
    assert!(
        row.starts_with("snap-1?total_cost?0 USD?fake\tvol-?[2J?x\t"),
        "{row:?}"
    );
    // JSON stays escaped and round-trips the original text.
    let o = bin().arg(&f).arg("--json").output().unwrap();
    assert!(o.status.success());
    let raw = String::from_utf8(o.stdout).unwrap();
    assert!(!raw.contains('\u{1b}'));
    assert!(raw.contains(r"\u001b"));
    let v: serde_json::Value = serde_json::from_str(&raw).unwrap();
    assert_eq!(
        v["snapshots"][0]["snapshot_id"],
        "snap-1\ntotal_cost\t0 USD\nfake"
    );
    assert_eq!(v["snapshots"][0]["volume_id"], "vol-\u{1b}[2J\tx");
}

#[test]
fn text_currency_case() {
    let f = tmp("cur-snap", &snap_json("snap-1", "vol-1", "8"));
    let p = tmp(
        "cur-prices",
        r#"{"currency":"USD\nowned\u001b]0;x\u0007","region":"r","source":"s","date":"d","snapshot_standard_per_gb_month":"0.05","snapshot_archive_per_gb_month":"0.0125"}"#,
    );
    let out = text_of(&[f.as_path()], &["--prices", p.to_str().unwrap()]);
    assert!(!out.contains('\u{1b}') && !out.contains('\u{7}'), "{out:?}");
    assert!(!out.lines().any(|l| l.starts_with("owned")), "{out:?}");
    assert!(out.contains("0.40 USD?owned?]0;x?"), "{out:?}");
    // JSON keeps the original currency text, escaped.
    let o = bin()
        .arg(&f)
        .args(["--json", "--prices"])
        .arg(&p)
        .output()
        .unwrap();
    let v: serde_json::Value = serde_json::from_slice(&o.stdout).unwrap();
    assert_eq!(v["currency"], "USD\nowned\u{1b}]0;x\u{7}");
}

#[test]
fn text_nul_and_formula_prefix() {
    let f = tmp("nul", &snap_json(r"a\u0000b", "vol-1", "8"));
    let out = text_of(&[f.as_path()], &[]);
    assert!(!out.contains('\0'));
    assert!(out.contains("a?b\t"), "{out:?}");
    for lead in ["=", "+", "-", "@"] {
        let f = tmp(
            "formula",
            &snap_json(&format!("{lead}1+1"), &format!("{lead}SUM(A1)"), "8"),
        );
        let out = text_of(&[f.as_path()], &[]);
        let row = out.lines().nth(1).unwrap();
        let cols: Vec<&str> = row.split('\t').collect();
        assert_eq!(cols[0], format!("'{lead}1+1"), "{row:?}");
        assert_eq!(cols[1], format!("'{lead}SUM(A1)"), "{row:?}");
        // The by_volume and orphans/old lines carry the same protection.
        assert!(
            out.lines()
                .any(|l| l.starts_with(&format!("'{lead}SUM(A1)\t"))),
            "{out:?}"
        );
        let v = json_rows(&[f.as_path()], &[]);
        assert_eq!(v["snapshots"][0]["snapshot_id"], format!("{lead}1+1"));
    }
    // Plain ids are not changed.
    let f = tmp("plain", &snap_json("snap-1", "vol-1", "8"));
    let out = text_of(&[f.as_path()], &[]);
    assert!(out.lines().nth(1).unwrap().starts_with("snap-1\tvol-1\t"));
}

#[test]
fn volumes_accepted_shapes() {
    let s = tmp("shape-snap", &snap_json("snap-1", "vol-aaaa1111", "8"));
    for body in [
        r#"{"Volumes":[{"VolumeId":"vol-aaaa1111"}]}"#,
        r#"[{"VolumeId":"vol-aaaa1111"}]"#,
        r#"[{"VolumeId":"vol-aaaa1111","Size":8},{"VolumeId":"vol-bbbb2222"}]"#,
        r#"["vol-aaaa1111"]"#,
    ] {
        let vf = tmp("shape-vol", body);
        let v = json_rows(&[s.as_path()], &["--volumes", vf.to_str().unwrap()]);
        assert_eq!(row(&v, "snap-1")["orphan"], false, "{body}");
        assert!(ids(&v, "orphans").is_empty(), "{body}");
    }
    // A known list that lacks the volume still marks the orphan.
    let vf = tmp("shape-other", r#"[{"VolumeId":"vol-zzzz9999"}]"#);
    let v = json_rows(&[s.as_path()], &["--volumes", vf.to_str().unwrap()]);
    assert_eq!(row(&v, "snap-1")["orphan"], true);
}

#[test]
fn volumes_bad_shapes_exit_2() {
    let s = tmp("bad-snap", &snap_json("snap-1", "vol-aaaa1111", "8"));
    for body in [
        "[]",
        r#"{"Volumes":[]}"#,
        r#"{"Volumes":[{"vol":"x"}]}"#,
        r#"{"Volumes":[{"VolumeId":1}]}"#,
        r#"{"Volumes":[{"VolumeId":null}]}"#,
        r#"{"Volumes":[{"VolumeId":"a","VolumeId":"b"}]}"#,
        r#"[{"VolumeId":"a","VolumeId":"b"}]"#,
        r#"[{"VolumeId":"vol-aaaa1111"},{"vol":"x"}]"#,
        r#"{"Volumes":[{"VolumeId":"vol-aaaa1111"},5]}"#,
        r#"[{"VolumeId":"vol-aaaa1111"},7]"#,
        r#"{"Volumes":[{"VolumeId":"vol-aaaa1111"}],"Volumes":[{"VolumeId":"vol-bbbb2222"}]}"#,
        r#"{"Other":[{"VolumeId":"vol-aaaa1111"}]}"#,
        r#"{"Volumes":"vol-aaaa1111"}"#,
        r#""vol-aaaa1111""#,
    ] {
        let vf = tmp("bad-vol", body);
        let o = bin().arg(&s).args(["--volumes"]).arg(&vf).output().unwrap();
        assert_eq!(o.status.code(), Some(2), "{body}");
        assert!(o.stdout.is_empty(), "{body}");
        assert!(!o.stderr.is_empty(), "{body}");
    }
}

fn two_snaps(vol_a: &str, vol_b: &str, size: &str) -> String {
    format!(
        r#"{{"Snapshots":[{{"SnapshotId":"a","VolumeId":"{vol_a}","VolumeSize":{size},"StartTime":"2026-10-01"}},{{"SnapshotId":"b","VolumeId":"{vol_b}","VolumeSize":{size},"StartTime":"2026-10-01"}}]}}"#
    )
}

#[test]
fn gb_overflow_exit_2() {
    let max = u64::MAX.to_string();
    // One snapshot of u64::MAX GB is still fine and exact.
    let f = tmp("gb-one", &snap_json("a", "vol-1", &max));
    let out = text_of(&[f.as_path()], &[]);
    assert!(out.contains(&format!("total_gb\t{max}\n")), "{out:?}");
    // Two of them overflow the GB total (the money still fits): exit 2, no output.
    for (a, b) in [("vol-1", "vol-1"), ("vol-1", "vol-2")] {
        let f = tmp("gb-two", &two_snaps(a, b, &max));
        for extra in [&[][..], &["--json"][..]] {
            let o = bin().arg(&f).args(extra).output().unwrap();
            assert_eq!(o.status.code(), Some(2), "{a} {b}");
            assert!(o.stdout.is_empty());
            assert!(String::from_utf8_lossy(&o.stderr).contains("does not fit"));
        }
    }
    // Just below the limit adds up exactly.
    let half = (u64::MAX / 2).to_string();
    let f = tmp("gb-half", &two_snaps("vol-1", "vol-2", &half));
    let out = text_of(&[f.as_path()], &[]);
    assert!(
        out.contains(&format!("total_gb\t{}\n", u64::MAX / 2 * 2)),
        "{out:?}"
    );
}

// ---- eindreview d84fb31: B1- (stderr), B2- (duplicate keys and ids), B3- (real flags) ----

/// Run with args that must fail with exit 2; returns stderr. Stdout must stay empty.
fn fail2(args: &[&std::path::Path], extra: &[&str]) -> String {
    let o = bin().args(args).args(extra).output().unwrap();
    assert_eq!(o.status.code(), Some(2), "{extra:?}");
    assert!(o.stdout.is_empty());
    String::from_utf8(o.stderr).unwrap()
}
fn assert_clean_stderr(err: &str) {
    assert!(!err.contains('\u{1b}') && !err.contains('\u{7}'), "{err:?}");
    assert_eq!(err.lines().count(), 1, "{err:?}");
    assert!(err.starts_with("ebs-snapshot-bill: "), "{err:?}");
}

#[test]
fn stderr_snapshot_id_clean() {
    // SnapshotId with ESC and a line break, and no VolumeSize.
    let f = tmp(
        "err-id",
        r#"{"Snapshots":[{"SnapshotId":"x\u001b[31mRED\nebs-snapshot-bill: fake ok","VolumeId":"v","StartTime":"2026-10-01"}]}"#,
    );
    let err = fail2(&[f.as_path()], &[]);
    assert_clean_stderr(&err);
    assert!(
        err.contains("x?[31mRED?ebs-snapshot-bill: fake ok"),
        "{err:?}"
    );
}

#[test]
fn stderr_volume_size_clean() {
    let f = tmp("err-size", &snap_json("s", "v", r#""8\u001b]0;pwn\u0007""#));
    let err = fail2(&[f.as_path()], &[]);
    assert_clean_stderr(&err);
    assert!(err.contains("VolumeSize"), "{err:?}");
    assert!(err.contains("8?]0;pwn?"), "{err:?}");
}

#[test]
fn stderr_start_time_clean() {
    let f = tmp(
        "err-time",
        r#"{"Snapshots":[{"SnapshotId":"s","VolumeId":"v","VolumeSize":8,"StartTime":"2026-1\u001b[2Jx\nfake"}]}"#,
    );
    let err = fail2(&[f.as_path()], &[]);
    assert_clean_stderr(&err);
    assert!(err.contains("2026-1?[2Jx"), "{err:?}");
}

#[test]
fn stderr_path_clean() {
    let p = PathBuf::from("nonexist\u{1b}[31m\nfake.json");
    let err = fail2(&[p.as_path()], &[]);
    assert_clean_stderr(&err);
    assert!(err.contains("nonexist?[31m?fake.json"), "{err:?}");
    // The same for the other path options.
    let good = tmp("err-path-ok", &snap_json("s", "v", "8"));
    for opt in ["--prices", "--volumes"] {
        let err = fail2(&[good.as_path()], &[opt, "no\u{1b}]0;x\u{7}.json"]);
        assert_clean_stderr(&err);
    }
}

#[test]
fn duplicate_keys_exit_2() {
    let snap = |extra: &str| {
        format!(
            r#"{{"Snapshots":[{{"SnapshotId":"s1","VolumeId":"v","VolumeSize":8,"StartTime":"2026-10-01"{extra}}}]}}"#
        )
    };
    let cases = [
        ("VolumeSize", snap(r#","VolumeSize":1000"#)),
        ("StorageTier", snap(r#","StorageTier":"archive","StorageTier":"standard""#)),
        ("SnapshotId", snap(r#","SnapshotId":"s2""#)),
        (
            "single-object",
            r#"{"SnapshotId":"s1","VolumeId":"v","VolumeSize":1,"VolumeSize":2,"StartTime":"2026-10-01"}"#
                .to_string(),
        ),
    ];
    for (name, body) in &cases {
        let f = tmp(&format!("dup-{name}"), body);
        let err = fail2(&[f.as_path()], &[]);
        assert!(err.contains("duplicate key"), "{name}: {err:?}");
    }
    // One of each key once is still fine.
    let f = tmp("dup-ok", &snap(""));
    text_of(&[f.as_path()], &[]);
}

#[test]
fn duplicate_price_keys_exit_2() {
    let good = tmp("dupp-snap", &snap_json("s", "v", "8"));
    let base = |a: &str, b: &str| {
        format!(
            r#"{{"currency":"USD"{a},"region":"r","source":"s","date":"d","snapshot_standard_per_gb_month":"0.05"{b},"snapshot_archive_per_gb_month":"0.0125"}}"#
        )
    };
    for (name, body) in [
        ("currency", base(r#","currency":"EUR""#, "")),
        ("rate", base("", r#","snapshot_standard_per_gb_month":"9""#)),
    ] {
        let p = tmp(&format!("dupp-{name}"), &body);
        let err = fail2(&[good.as_path()], &["--prices", p.to_str().unwrap()]);
        assert!(err.contains("duplicate key"), "{name}: {err:?}");
    }
    let p = tmp("dupp-ok", &base("", ""));
    text_of(&[good.as_path()], &["--prices", p.to_str().unwrap()]);
}

#[test]
fn duplicate_snapshot_id_exit_2() {
    // The same file twice.
    let f = tmp("dupid-file", &snap_json("s1", "vol-1", "8"));
    let err = fail2(&[f.as_path(), f.as_path()], &[]);
    assert!(err.contains("duplicate SnapshotId s1"), "{err:?}");
    // Two entries with the same SnapshotId in one file, on different volumes.
    let body = r#"{"Snapshots":[{"SnapshotId":"s1","VolumeId":"vol-1","VolumeSize":8,"StartTime":"2026-10-01"},{"SnapshotId":"s1","VolumeId":"vol-2","VolumeSize":9,"StartTime":"2026-10-01"}]}"#;
    let g = tmp("dupid-two", body);
    let err = fail2(&[g.as_path()], &["--json"]);
    assert!(err.contains("duplicate SnapshotId s1"), "{err:?}");
    // Different ids over two files are fine.
    let h = tmp("dupid-other", &snap_json("s2", "vol-1", "8"));
    let out = text_of(&[f.as_path(), h.as_path()], &[]);
    assert!(out.contains("total_gb\t16\n"), "{out:?}");
}

fn age_old(f: &std::path::Path, extra: &[&str]) -> (String, String) {
    let out = text_of(&[f], extra);
    let line = out.lines().find(|l| l.starts_with("s\t")).unwrap();
    let c: Vec<&str> = line.split('\t').collect();
    (c[3].to_string(), c[6].to_string())
}

#[test]
fn as_of_changes_age_and_old() {
    // StartTime 2026-10-01: 30 days to 2026-10-31, 90 days to 2026-12-30.
    let f = tmp("asof", &snap_json("s", "v", "8"));
    assert_eq!(
        age_old(&f, &["--as-of", "2026-10-31", "--max-age-days", "29"]),
        ("30".into(), "true".into())
    );
    assert_eq!(
        age_old(&f, &["--as-of", "2026-10-31", "--max-age-days", "30"]),
        ("30".into(), "false".into())
    );
    let (age, _) = age_old(&f, &["--as-of", "2026-12-30"]);
    assert_eq!(age, "90");
    // The flag is on the row, not only a word in the header, and the JSON shows it too.
    let v = json_rows(
        &[f.as_path()],
        &["--as-of", "2026-12-30", "--max-age-days", "89"],
    );
    assert_eq!(row(&v, "s")["old"], true);
    assert_eq!(ids(&v, "old"), vec!["s"]);
}

#[test]
fn age_equal_to_max_is_not_old() {
    // Age 90 with --max-age-days 90 is not old; with 89 it is.
    let f = tmp("edge", &snap_json("s", "v", "8"));
    let base = ["--as-of", "2026-12-30"];
    let at = age_old(&f, &[base[0], base[1], "--max-age-days", "90"]);
    assert_eq!(at, ("90".into(), "false".into()));
    let over = age_old(&f, &[base[0], base[1], "--max-age-days", "89"]);
    assert_eq!(over, ("90".into(), "true".into()));
}

#[test]
fn directory_input_is_read() {
    let f = tmp("dir", &snap_json("s1", "vol-1", "8"));
    let dir = f.parent().unwrap().to_path_buf();
    std::fs::write(dir.join("b.json"), snap_json("s2", "vol-2", "10")).unwrap();
    // Not read: a file with another extension.
    std::fs::write(dir.join("c.txt"), "not json").unwrap();
    let v = json_rows(&[dir.as_path()], &[]);
    assert_eq!(v["total_gb"], 18);
    let _ = row(&v, "s1");
    let _ = row(&v, "s2");
    assert_eq!(v["snapshots"].as_array().unwrap().len(), 2);
}

/// Run the binary through `sh` with standard output closed (`>&-`) or sent to /dev/null.
#[cfg(target_os = "linux")]
fn via_sh(redirect: &str, args: &[&str]) -> std::process::Output {
    let exe = assert_cmd::cargo::cargo_bin("ebs-snapshot-bill");
    std::process::Command::new("sh")
        .arg("-c")
        .arg(format!("exec \"$0\" \"$@\" {redirect}"))
        .arg(exe)
        .args(args)
        .output()
        .unwrap()
}

#[cfg(target_os = "linux")]
#[test]
fn closed_stdout_is_exit_2() {
    let n = fix("normal.json");
    for args in [vec![n.to_str().unwrap()], vec!["--version"], vec!["--help"]] {
        let o = via_sh(">&-", &args);
        assert_eq!(o.status.code(), Some(2), "args {args:?}");
        assert!(
            String::from_utf8_lossy(&o.stderr).contains("could not write output"),
            "args {args:?}"
        );
    }
}

#[cfg(target_os = "linux")]
#[test]
fn dev_null_stdout_is_exit_0() {
    let n = fix("normal.json");
    let o = via_sh(">/dev/null", &[n.to_str().unwrap()]);
    assert_eq!(o.status.code(), Some(0));
}
