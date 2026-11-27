//! Engine logs and panics to logcat under the `VelmoraEngine` tag
//! (`adb logcat -s VelmoraEngine`). Does nothing off Android, where the only
//! build is the one the bindings are generated from.

#[cfg(target_os = "android")]
mod imp {
    use std::ffi::CString;
    use std::io::Write;
    use std::os::raw::{c_char, c_int};

    const INFO: c_int = 4;
    const ERROR: c_int = 6;

    #[link(name = "log")]
    extern "C" {
        fn __android_log_write(prio: c_int, tag: *const c_char, text: *const c_char) -> c_int;
    }

    fn write(prio: c_int, text: &str) {
        let text = text.trim_end();
        if text.is_empty() {
            return;
        }
        if let Ok(msg) = CString::new(text.replace('\0', " ")) {
            // SAFETY: both pointers are valid NUL-terminated strings for the call.
            unsafe {
                __android_log_write(prio, c"VelmoraEngine".as_ptr(), msg.as_ptr());
            }
        }
    }

    /// Collects one formatted event and sends it to logcat when dropped.
    struct Line(Vec<u8>);

    impl Write for Line {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            self.0.extend_from_slice(buf);
            Ok(buf.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    impl Drop for Line {
        fn drop(&mut self) {
            write(INFO, &String::from_utf8_lossy(&self.0));
        }
    }

    pub fn init() {
        static ONCE: std::sync::Once = std::sync::Once::new();
        ONCE.call_once(|| {
            let filter = tracing_subscriber::EnvFilter::new("info,librqbit=warn,librqbit_dht=warn");
            let _ = tracing_subscriber::fmt()
                .with_env_filter(filter)
                .with_ansi(false)
                .without_time()
                .with_writer(|| Line(Vec::new()))
                .try_init();
            let previous = std::panic::take_hook();
            std::panic::set_hook(Box::new(move |info| {
                write(ERROR, &format!("panic: {info}"));
                previous(info);
            }));
        });
    }
}

#[cfg(not(target_os = "android"))]
mod imp {
    pub fn init() {}
}

pub use imp::init;
