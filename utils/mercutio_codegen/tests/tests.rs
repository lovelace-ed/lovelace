#[test]
fn test() {
    let t = trybuild::TestCases::new();
    t.pass("tests/examples/pass.rs");
    t.compile_fail("tests/examples/fail.rs");
}
