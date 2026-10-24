use std::process::Command;

fn run(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_teamy-domains"))
        .args(args)
        .output()
        .expect("run offline CLI")
}

#[test]
fn nested_help_exposes_offline_input_and_no_authentication() {
    let root_help = run(&["--help"]);
    assert!(root_help.status.success());
    assert!(
        String::from_utf8(root_help.stdout)
            .unwrap()
            .contains("decoding is not implemented")
    );
    let output = run(&["record", "add", "--help"]);
    assert!(output.status.success());
    let help = String::from_utf8(output.stdout).unwrap();
    assert!(help.contains("snapshot"));
    assert!(help.contains("record-type"));
    assert!(!help.contains("api-key"));
}

#[test]
fn record_add_outputs_full_plan_without_touching_fixture() {
    let fixture = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/example_snapshot.json"
    );
    let before = std::fs::read(fixture).unwrap();
    let output = run(&[
        "--output-format",
        "json",
        "record",
        "add",
        "--snapshot",
        fixture,
        "--host",
        "docs",
        "--record-type",
        "CNAME",
        "--address",
        "pages.example.test",
        "--ttl",
        "300",
    ]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let report = String::from_utf8(output.stdout).unwrap();
    assert!(report.contains("verification=keep"));
    assert!(report.contains("FUTURE"));
    assert!(report.contains("pages.example.test"));
    assert_eq!(std::fs::read(fixture).unwrap(), before);
}
