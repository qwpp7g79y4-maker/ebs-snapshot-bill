use assert_cmd::Command;
use std::path::PathBuf;
fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}
#[test]
fn readme_sections() {
    let t = std::fs::read_to_string(root().join("README.md")).unwrap();
    for n in [
        "Why",
        "Install",
        "Quick start",
        "How it counts",
        "Limitations",
        "https://aws.amazon.com/ebs/pricing/",
        "2026-11-09",
        "--max-age-days",
        "orphan",
        "GB-month",
    ] {
        assert!(t.contains(n), "missing {n}");
    }
    assert!(!t.contains("PLACEHOLDER"));
}
#[test]
fn changelog() {
    assert!(std::fs::read_to_string(root().join("CHANGELOG.md"))
        .unwrap()
        .contains("0.1.0"));
}
#[test]
fn licenses() {
    assert!(root().join("LICENSE-MIT").is_file());
    assert!(root().join("LICENSE-APACHE").is_file());
}
#[test]
fn examples() {
    Command::cargo_bin("ebs-snapshot-bill")
        .unwrap()
        .arg(root().join("examples/normal.json"))
        .assert()
        .success();
    Command::cargo_bin("ebs-snapshot-bill")
        .unwrap()
        .arg(root().join("examples/old.json"))
        .args(["--max-age-days", "30"])
        .assert()
        .success();
}
#[test]
fn quality() {
    let t = std::fs::read_to_string(root().join("QUALITY.md")).unwrap();
    assert!(t.contains("Tests:"));
}
#[test]
fn not_ebs_wees() {
    // Behaviour, not README wording: a snapshot without VolumeId is an orphan.
    let o = Command::cargo_bin("ebs-snapshot-bill")
        .unwrap()
        .arg(root().join("tests/fixtures/novolume.json"))
        .arg("--json")
        .output()
        .unwrap();
    assert!(o.status.success());
    let v: serde_json::Value = serde_json::from_slice(&o.stdout).unwrap();
    assert_eq!(v["orphans"][0], "snap-novol1");
    assert_eq!(v["snapshots"][0]["orphan"], true);
}
