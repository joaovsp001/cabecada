//! Small things that work differently on the phone and on the computer.

/// A line in the Android log (logcat, tag "SAPP"), to follow what the game does on a
/// phone. Does nothing on the computer.
pub fn log(msg: &str) {
    #[cfg(target_os = "android")]
    {
        if let Ok(c) = std::ffi::CString::new(msg.replace('\0', " ")) {
            unsafe { macroquad::miniquad::native::android::console_info(c.as_ptr()) };
        }
    }
    #[cfg(not(target_os = "android"))]
    let _ = msg;
}

/// Closes the game. On Android the activity is finished first, so the system does
/// not start it again when the process ends.
pub fn quit() -> ! {
    #[cfg(target_os = "android")]
    unsafe {
        use macroquad::miniquad::native::android as a;
        let env = a::attach_jni_env();
        macroquad::miniquad::call_void_method!(env, a::ACTIVITY, "finish", "()V");
        std::thread::sleep(std::time::Duration::from_millis(150));
    }
    std::process::exit(0)
}

/// Remembers whether the app went to the background (home button, a call, the screen
/// turned off) since the last frame, so a running match can be paused.
#[derive(Default)]
pub struct LifeEvents {
    pub went_away: bool,
}

impl macroquad::miniquad::EventHandler for LifeEvents {
    fn update(&mut self) {}
    fn draw(&mut self) {}
    fn window_minimized_event(&mut self) {
        self.went_away = true;
    }
}
