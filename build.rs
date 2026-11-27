// Embed the app icon as resource #1: Explorer shows it on the exe, and GPUI
// loads resource #1 for the window title bar and taskbar.
fn main() {
    println!("cargo:rerun-if-changed=assets/focuslock.rc");
    println!("cargo:rerun-if-changed=assets/focuslock.ico");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        embed_resource::compile("assets/focuslock.rc", embed_resource::NONE)
            .manifest_optional()
            .expect("embed the app icon");
    }
}
