//! Puts the app icon into the Windows executable (GPUI takes the window and taskbar icon from
//! resource 1).
fn main() {
    #[cfg(windows)]
    {
        println!("cargo:rerun-if-changed=app.rc");
        println!("cargo:rerun-if-changed=icon.ico");
        let _ = embed_resource::compile("app.rc", embed_resource::NONE);
    }
}
