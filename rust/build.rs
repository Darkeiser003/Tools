use std::env;

fn main() {
    println!("cargo:rerun-if-env-changed=LTOOLS_EMBEDDED_UPDATE_PUBLIC_KEY");
    if let Ok(key) = env::var("LTOOLS_EMBEDDED_UPDATE_PUBLIC_KEY") {
        let canonical: String = key
            .chars()
            .filter(|character| !character.is_whitespace())
            .collect();
        if !canonical.is_empty() {
            println!("cargo:rustc-env=LTOOLS_EMBEDDED_UPDATE_PUBLIC_KEY={canonical}");
        }
    }
}
