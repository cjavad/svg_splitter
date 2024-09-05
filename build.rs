fn main() {
    println!("cargo:rustc-link-lib=static=stdc++");
    println!("cargo:rustc-link-lib=OpenScadLibSvgCFFI");
    println!("cargo:rustc-link-search=.");
}
