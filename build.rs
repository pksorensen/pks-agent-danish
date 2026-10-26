fn main() {
    // include_dir embeds existing files, but Cargo must also notice newly added
    // rule pairs so a contributor's next build/test uses the updated catalog.
    println!("cargo:rerun-if-changed=rules");
    println!("cargo:rerun-if-changed=styles");
}
