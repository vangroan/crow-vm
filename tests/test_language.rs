#[test]
fn test_core_hello_world() {
    crow::compile_file("tests/language/core/hello-world.crow").unwrap();
}

#[test]
fn test_local_arithmetic() {
    crow::compile_file("tests/language/local/arithmetic.crow").unwrap();
}
