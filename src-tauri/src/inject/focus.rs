use std::sync::Mutex;

static SAVED_TARGET: Mutex<Option<isize>> = Mutex::new(None);

pub fn capture_target_window() {
    #[cfg(target_os = "windows")]
    {
        use winapi::um::winuser::GetForegroundWindow;
        unsafe {
            let hwnd = GetForegroundWindow();
            if !hwnd.is_null() {
                if let Ok(mut guard) = SAVED_TARGET.lock() {
                    *guard = Some(hwnd as isize);
                }
            }
        }
    }

    #[cfg(target_os = "macos")]
    {
        let _guard = SAVED_TARGET.lock();
    }
}

pub fn restore_target_window() {
    #[cfg(target_os = "windows")]
    {
        use winapi::shared::windef::HWND;
        use winapi::um::winuser::{
            AttachThreadInput, GetForegroundWindow, GetWindowThreadProcessId, SetForegroundWindow,
            ShowWindow, SW_SHOW,
        };

        let target = SAVED_TARGET.lock().ok().and_then(|g| *g);
        let Some(hwnd) = target else {
            return;
        };

        unsafe {
            let hwnd = hwnd as HWND;
            let foreground = GetForegroundWindow();
            if foreground.is_null() || foreground == hwnd {
                let _ = SetForegroundWindow(hwnd);
                let _ = ShowWindow(hwnd, SW_SHOW);
                return;
            }

            let mut foreground_pid = 0;
            let mut target_pid = 0;
            let fg_thread = GetWindowThreadProcessId(foreground, &mut foreground_pid);
            let target_thread = GetWindowThreadProcessId(hwnd, &mut target_pid);

            if fg_thread != 0 && target_thread != 0 {
                AttachThreadInput(fg_thread, target_thread, 1);
            }
            let _ = SetForegroundWindow(hwnd);
            let _ = ShowWindow(hwnd, SW_SHOW);
            if fg_thread != 0 && target_thread != 0 {
                AttachThreadInput(fg_thread, target_thread, 0);
            }
        }
    }
}

pub fn clear_target_window() {
    if let Ok(mut guard) = SAVED_TARGET.lock() {
        *guard = None;
    }
}
