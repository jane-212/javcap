use std::process::Command;

fn main() {
    #[cfg(debug_assertions)]
    println!("cargo:rustc-env=VERSION=DEBUG");

    let git_hash = Command::new("git")
        .arg("rev-parse")
        .arg("--short")
        .arg("HEAD")
        .output()
        .map(|output| String::from_utf8(output.stdout))
        .expect("get git hash failed")
        .expect("get git hash failed")
        .trim()
        .to_string();

    println!("cargo:rustc-env=HASH={git_hash}");
}
