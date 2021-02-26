use std::fs::read_to_string;

#[test]
fn test_direct_apply() {
    let t = trybuild::TestCases::new();
    t.pass("tests/examples/pass.rs");
}

#[test]
fn test_classes_to_file() {
    let t = trybuild::TestCases::new();
    t.pass("tests/write_file/pass.rs");
    std::mem::drop(t);
    let file = read_to_string(&format!(
        "{}/../../target/tests/mercutio_codegen/styles.css",
        std::env::var("CARGO_MANIFEST_DIR").unwrap()
    ))
    .unwrap();
    assert!(file.contains("{font-size:24px;font-family:sans-serif;}"));
}
