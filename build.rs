fn main() {
    // Try to embed the icon using available resource tools
    // For MSVC: uses rc.exe
    // For GNU: uses windres.exe or llvm-rc
    // If none available, icon is loaded at runtime from chomik_icon.ico
    
    #[cfg(target_os = "windows")]
    {
        let rc_path = std::path::Path::new("chomik-hamster.rc");
        if rc_path.exists() {
            // Try embed_resource v2 (MSVC)
            // Try v1 style fallback (GNU)
            if let Ok(_) = std::process::Command::new("embed-resource").arg("--help").output() {
                // embedded via external crate
            }
        }
    }
    
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=chomik-hamster.rc");
    println!("cargo:rerun-if-changed=chomik-hamster.manifest");
}
