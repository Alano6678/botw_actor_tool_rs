//! Build script: embed the Windows application icon into the release exe.

fn main() {
    #[cfg(windows)]
    {
        winresource::WindowsResource::new()
            .set_icon("assets/app.ico")
            .compile()
            .expect("failed to embed app icon (assets/app.ico)");
    }
    println!("cargo:rerun-if-changed=assets/app.ico");
}
