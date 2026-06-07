fn main() {
    if std::env::var("SPINE_GENERATE_HEADER").is_err() {
        return;
    }
    let header = cbindgen::Builder::new()
        .with_crate(".")
        .generate()
        .expect("failed to generate header");
    let out_dir = std::path::PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap())
        .parent()
        .unwrap()
        .join("include")
        .join("spine.h");
    header.write_to_file(out_dir);
}
