#[test]
fn ui() {
    let t = trybuild::TestCases::new();
    t.pass("tests/ui/derive_test.rs");
    t.compile_fail("tests/ui/non_trait.rs");
}