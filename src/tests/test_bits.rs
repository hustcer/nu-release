use crate::tests::{fail_test, run_test, TestResult};

#[test]
fn bits_and() -> TestResult {
    run_test("2 | bits and 4", "0")
}

#[test]
fn bits_and_list() -> TestResult {
    run_test("[1 2 3 8 9 10] | bits and 2 | str collect", "022002")
}
