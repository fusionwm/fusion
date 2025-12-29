use std::{
    path::{Path, PathBuf},
    process::Command,
};

fn workspace_dir() -> PathBuf {
    let output = Command::new(env!("CARGO"))
        .arg("locate-project")
        .arg("--workspace")
        .arg("--message-format=plain")
        .output()
        .unwrap()
        .stdout;
    let cargo_path = Path::new(std::str::from_utf8(&output).unwrap().trim());
    cargo_path.parent().unwrap().to_path_buf()
}

fn main() {
    let grammar_dir = workspace_dir().join("lib/tree-sitter-config/src");
    cc::Build::new()
        .include(&grammar_dir)
        .file(grammar_dir.join("parser.c"))
        .compile("tree-sitter-config");

    println!("cargo:rerun-if-changed=tree-sitter-config/src/parser.c");
    println!("cargo:rerun-if-changed=tree-sitter-config/src/scanner.c");
}
