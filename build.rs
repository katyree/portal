fn main() {
    println!("cargo:rerun-if-changed=assets/portal.rc");
    println!("cargo:rerun-if-changed=assets/portal.ico");
    embed_resource::compile("assets/portal.rc", embed_resource::NONE)
        .manifest_required()
        .expect("Could not embed the Portal Windows icon");
}
