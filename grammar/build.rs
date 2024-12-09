use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=src");

    // println!(
    //     "{:#?}",
    //     rust_sitter_tool::generate_grammars(&PathBuf::from("src/lib.rs"))
    // );

    rust_sitter_tool::build_parsers(&PathBuf::from("src/lib.rs"));
}
