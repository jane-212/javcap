use std::process::Command;

fn main() {
    #[cfg(debug_assertions)]
    println!("cargo:rustc-env=VERSION=DEBUG");

    let out = Command::new("git")
        .arg("rev-parse")
        .arg("--short")
        .arg("HEAD")
        .output()
        .expect("execute git failed")
        .stdout;
    let git_hash = String::from_utf8(out).expect("parse output to string failed");

    println!("cargo:rustc-env=HASH={git_hash}");
}
