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

/// The folder where the game may keep its files (settings): the app's private folder on
/// Android (asked to Android through JNI, once), the application-data folder on Windows,
/// and ~/.config elsewhere.
pub fn data_dir() -> Option<std::path::PathBuf> {
    static DIR: std::sync::OnceLock<Option<std::path::PathBuf>> = std::sync::OnceLock::new();
    DIR.get_or_init(find_data_dir).clone()
}

fn find_data_dir() -> Option<std::path::PathBuf> {
    #[cfg(target_os = "android")]
    unsafe {
        use macroquad::miniquad::native::android as a;
        let env = a::attach_jni_env();
        let file = macroquad::miniquad::call_object_method!(env, a::ACTIVITY, "getFilesDir", "()Ljava/io/File;");
        if file.is_null() {
            return None;
        }
        let path = macroquad::miniquad::call_object_method!(env, file, "getAbsolutePath", "()Ljava/lang/String;");
        if path.is_null() {
            return None;
        }
        let s = macroquad::miniquad::get_utf_str!(env, path);
        return Some(std::path::PathBuf::from(s));
    }
    #[cfg(not(target_os = "android"))]
    {
        if let Ok(appdata) = std::env::var("APPDATA") {
            return Some(std::path::PathBuf::from(appdata).join("Cabecada"));
        }
        let home = std::env::var("HOME").ok()?;
        Some(std::path::PathBuf::from(home).join(".config").join("cabecada"))
    }
}

/// Copies text to the clipboard (the room code, for example). On Android it calls
/// `cabecadaCopyText`, added to the activity by android/MainActivityInject.java.
pub fn copy_text(s: &str) {
    #[cfg(target_os = "android")]
    unsafe {
        use macroquad::miniquad::native::android as a;
        let Ok(c) = std::ffi::CString::new(s.replace('\0', "")) else { return };
        let env = a::attach_jni_env();
        let jtext = ((**env).NewStringUTF.unwrap())(env, c.as_ptr());
        if jtext.is_null() {
            return;
        }
        macroquad::miniquad::call_void_method!(env, a::ACTIVITY, "cabecadaCopyText", "(Ljava/lang/String;)V", jtext);
        ((**env).DeleteLocalRef.unwrap())(env, jtext);
    }
    #[cfg(not(target_os = "android"))]
    macroquad::miniquad::window::clipboard_set(s);
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
