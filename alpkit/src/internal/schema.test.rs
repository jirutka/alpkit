use super::*;

#[test]
fn dependencies_schema() {
    // shouldn't panic
    Dependencies::json_schema(&mut Default::default());
}

#[test]
fn secfixes_schema() {
    // shouldn't panic
    Secfixes::json_schema(&mut Default::default());
}
