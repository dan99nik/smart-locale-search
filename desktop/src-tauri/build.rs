fn main() {
    println!("cargo:rustc-link-arg=/NODEFAULTLIB:libcpmt.lib");
    println!("cargo:rustc-link-arg=/NODEFAULTLIB:libcpmtd.lib");
    tauri_build::build()
}
