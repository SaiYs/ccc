use std::process::Command;

pub(crate) fn assert_exit_code(s: &str, expected: i32) {
    let testcase_id: u32 = rand::random();
    let test_asm_name = format!("./target/tmp/testcase{}.s", testcase_id);
    let test_bin_name = format!("./target/tmp/testcase{}", testcase_id);

    let _ = Command::new("cargo")
        .args(["run", "-q", "--", "-c", s, "-o", test_asm_name.as_str()])
        .spawn()
        .expect("failed to execute sofac")
        .wait()
        .unwrap();

    let _ = Command::new("gcc")
        .args([
            test_asm_name.as_str(),
            "-o",
            test_bin_name.as_str(),
            "-static",
        ])
        .spawn()
        .expect("failed to assemble with gcc")
        .wait()
        .unwrap();

    let status = Command::new(test_bin_name.as_str())
        .status()
        .expect("failed to run binary");

    assert_eq!(status.code(), Some(expected));
    std::fs::remove_file(test_asm_name).unwrap();
    std::fs::remove_file(test_bin_name).unwrap();
}
