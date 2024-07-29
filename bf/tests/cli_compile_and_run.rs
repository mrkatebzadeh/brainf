use assert_cmd::cargo::cargo_bin;
use std::fs;
use std::process::Command;

#[test]
fn compile_and_run_hello() {
    // Hello World!
    let src = "++++++++++[>+++++++>++++++++++>+++>+<<<<-]>++.>+.+++++++..+++.>++.<<+++++++++++++++.>.+++.------.--------.>+.>.";
    let dir = tempfile::tempdir().unwrap();
    let src_path = dir.path().join("hello.bf");
    fs::write(&src_path, src).unwrap();

    let bf = cargo_bin("bf");
    let out_path = dir.path().join("hello");

    let status = Command::new(&bf)
        .args(["-c", "-O", "2", "-f"])
        .arg(&src_path)
        .args(["-o"])
        .arg(&out_path)
        .status()
        .unwrap();
    assert!(status.success());

    let out = Command::new(&out_path).output().unwrap();
    assert!(out.status.success());
    assert_eq!(String::from_utf8_lossy(&out.stdout), "Hello World!\n");
}
