#[test]
fn normalize_macro_contract() {
    let tests = trybuild::TestCases::new();

    tests.pass("tests/ui/input_policy/pass.rs");

    tests.compile_fail("tests/ui/input_policy/fail_unknown_operation.rs");

    tests.compile_fail("tests/ui/input_policy/fail_wrong_type.rs");

    tests.compile_fail("tests/ui/input_policy/fail_empty.rs");

    tests.compile_fail("tests/ui/input_policy/fail_duplicate.rs");

    tests.compile_fail("tests/ui/input_policy/fail_enum.rs");

    tests.compile_fail("tests/ui/input_policy/fail_assignment.rs");

    tests.compile_fail("tests/ui/input_policy/fail_arguments.rs");
}
