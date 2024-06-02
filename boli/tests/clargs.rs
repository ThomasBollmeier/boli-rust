use assert_cmd::Command;

#[test]
fn test_command_line_args() {
    let mut cmd = Command::cargo_bin("boli").unwrap();

    let result = cmd
        .arg("tests/input/clargs.boli")
        .arg("arg1")
        .arg("-v2")
        .arg("--answer=42")
        .ok();

    assert!(result.is_ok());

    let output = String::from_utf8(result.unwrap().stdout).unwrap();
    let expected_output = std::fs::read_to_string("tests/output/clargs.out").unwrap();
    assert_eq!(output, expected_output);
}
