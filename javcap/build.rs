fn main() {
    #[cfg(debug_assertions)]
    println!("cargo:rustc-env=VERSION=DEBUG");
}
