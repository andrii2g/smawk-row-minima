use std::process::Command;

fn run(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_smawk-row-minima"))
        .args(args)
        .output()
        .unwrap()
}

#[test]
fn verify_and_benchmark_commands() {
    let output = run(&["verify", "--rows", "8", "--cols", "12"]);
    assert!(output.status.success());
    assert!(
        String::from_utf8(output.stdout)
            .unwrap()
            .contains("Monge check: passed")
    );
    let output = run(&["bench", "--sizes", "8", "--repeats", "2"]);
    assert!(output.status.success());
    let csv = String::from_utf8(output.stdout).unwrap();
    assert_eq!(csv.lines().count(), 7);
    assert!(csv.lines().all(|line| line.split(',').count() == 11));
}

#[test]
fn invalid_arguments_fail_cleanly() {
    for args in [
        vec!["oops"],
        vec!["verify", "--rows"],
        vec!["verify", "--rows", "-1"],
        vec!["verify", "--cols", "0"],
        vec!["visualize", "--rows", "33"],
        vec!["bench", "--repeats", "0"],
        vec!["verify", "--wat", "1"],
        vec!["verify", "--seed", "1", "--seed", "2"],
    ] {
        let output = run(&args);
        assert!(!output.status.success(), "{args:?}");
        assert!(
            String::from_utf8(output.stderr)
                .unwrap()
                .starts_with("error:")
        );
    }
}

#[test]
fn svg_file_contains_search_explanation() {
    let path = std::env::temp_dir().join(format!("smawk-{}-test.svg", std::process::id()));
    let output = run(&[
        "visualize",
        "--rows",
        "8",
        "--cols",
        "12",
        "--out",
        path.to_str().unwrap(),
    ]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let svg = std::fs::read_to_string(&path).unwrap();
    std::fs::remove_file(path).unwrap();
    assert!(svg.contains("<svg xmlns="));
    assert!(svg.contains("Depth 1"));
    assert!(svg.contains("Depth 3"));
    assert!(svg.contains("<polyline"));
    assert!(svg.contains("evaluations "));
    assert!(svg.ends_with("</svg>\n"));
}
