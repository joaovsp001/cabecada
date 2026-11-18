//! For automatic checks: keep the game window out of the person's way.
//! On Windows the window goes behind the others and focus goes back to
//! whatever was in front before. Elsewhere nothing is done.

#[cfg(windows)]
pub fn foreground() -> isize {
    unsafe { windows_sys::Win32::UI::WindowsAndMessaging::GetForegroundWindow() as isize }
}

#[cfg(not(windows))]
pub fn foreground() -> isize {
    0
}

#[cfg(windows)]
pub fn lower_window(prev: isize) {
    use windows_sys::Win32::Foundation::{BOOL, HWND, LPARAM};
    use windows_sys::Win32::System::Threading::GetCurrentProcessId;
    use windows_sys::Win32::UI::WindowsAndMessaging::*;

    unsafe extern "system" fn each(hwnd: HWND, lp: LPARAM) -> BOOL {
        let mut pid = 0u32;
        GetWindowThreadProcessId(hwnd, &mut pid);
        if pid == GetCurrentProcessId() && IsWindowVisible(hwnd) != 0 {
            let out = lp as *mut Vec<isize>;
            (*out).push(hwnd as isize);
        }
        1
    }
    unsafe {
        let mut mine: Vec<isize> = vec![];
        EnumWindows(Some(each), &mut mine as *mut Vec<isize> as LPARAM);
        for h in &mine {
            SetWindowPos(*h as HWND, HWND_BOTTOM, 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE);
        }
        if prev != 0 && !mine.contains(&prev) {
            SetForegroundWindow(prev as HWND);
        }
    }
}

#[cfg(not(windows))]
pub fn lower_window(_prev: isize) {}
