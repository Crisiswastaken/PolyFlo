use crate::config::PlatformInfo;

pub fn injection_reliable() -> bool {
    if cfg!(target_os = "windows") {
        return true;
    }
    if cfg!(target_os = "macos") {
        return macos_accessibility_granted();
    }
    false
}

#[cfg(target_os = "macos")]
fn macos_accessibility_granted() -> bool {
    macos_accessibility::accessibility::application_is_trusted()
}

#[cfg(not(target_os = "macos"))]
fn macos_accessibility_granted() -> bool {
    true
}

pub fn paste_modifier_label() -> &'static str {
    if cfg!(target_os = "macos") {
        "Cmd"
    } else {
        "Ctrl"
    }
}

pub fn get_platform_info() -> PlatformInfo {
    PlatformInfo {
        os: std::env::consts::OS.to_string(),
        injection_reliable: injection_reliable(),
        paste_modifier: paste_modifier_label().to_string(),
    }
}

#[cfg(target_os = "macos")]
mod macos_accessibility {
    #[link(name = "ApplicationServices", kind = "framework")]
    extern "C" {
        fn AXIsProcessTrusted() -> bool;
    }

    pub mod accessibility {
        pub fn application_is_trusted() -> bool {
            unsafe { super::AXIsProcessTrusted() }
        }
    }
}
