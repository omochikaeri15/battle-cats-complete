pub fn suffix() -> &'static str {
    match () {
        _ if cfg!(target_os = "windows") => "windows_x64",
        _ if cfg!(all(target_os = "macos", target_arch = "aarch64")) => "mac_arm",
        _ if cfg!(target_os = "macos") => "mac_x64",
        _ if cfg!(target_arch = "aarch64") => "linux_arm",
        _ => "linux_x64",
    }
}
