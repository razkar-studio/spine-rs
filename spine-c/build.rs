fn main() {
    cbindgen::Builder::new()
        .with_crate(".")
        .generate()
        .expect("failed to generate header")
        .write_to_file("../include/spine.h");
}
