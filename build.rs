fn main() {
    println!("cargo:rerun-if-changed=windows/icon.ico");
    println!("cargo:rerun-if-changed=windows/icon.rc");
    if std::env::var("CARGO_CFG_TARGET_OS").ok().as_deref() == Some("windows") {
        let _ = embed_resource::compile("windows/icon.rc", embed_resource::NONE);
    }
}
