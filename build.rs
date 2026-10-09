fn main() {
    // Only compile Windows resources when building for Windows targets
    if std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default() == "windows" {
        let mut res = winres::WindowsResource::new();
        res.set_icon("art/icon.ico");
        let _ = res.compile();
    }
}
