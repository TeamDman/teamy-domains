use std::path::Path;
use std::process::Command;

fn assert_source_pin(relative: &str, expected: &str) {
    let checkout = Path::new(env!("CARGO_MANIFEST_DIR")).join(relative);
    let output = Command::new("git")
        .arg("-C")
        .arg(checkout)
        .args(["rev-parse", "HEAD"])
        .output()
        .expect("git is required to verify exact owned source pins");
    assert!(output.status.success(), "owned source checkout unavailable");
    assert_eq!(String::from_utf8(output.stdout).unwrap().trim(), expected);
    let status = Command::new("git")
        .arg("-C")
        .arg(Path::new(env!("CARGO_MANIFEST_DIR")).join(relative))
        .args(["status", "--porcelain"])
        .output()
        .expect("git is required to check owned source worktrees");
    assert!(status.status.success(), "owned source status unavailable");
    assert!(status.stdout.is_empty(), "owned source checkout is dirty");
}

#[test]
fn qualification_uses_reviewed_owned_source_commits() {
    assert_source_pin(
        "../../../facet-format-domains",
        "fe271471dfeed52a049ce2df6aeb72bb9c75a632",
    );
    assert_source_pin("../../../facet", "74b1070dc1c889f429ba79ff458a38dae46209a5");
}
