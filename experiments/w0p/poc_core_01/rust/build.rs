// RESEARCH ONLY / W0-P / NON-PRODUCTION
fn main() {
    let lib = std::env::var("UFBX_LIB_DIR").expect("UFBX_LIB_DIR must point at prebuilt ufbx.lib + ufbx_access.lib");
    println!("cargo:rustc-link-search=native={lib}");
    println!("cargo:rustc-link-lib=static=ufbx");
    println!("cargo:rustc-link-lib=static=ufbx_access");
    println!("cargo:rerun-if-env-changed=UFBX_LIB_DIR");
}
