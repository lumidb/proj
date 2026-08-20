use proj_sys::{PJ_LOG_LEVEL, PJ_LOG_LEVEL_PJ_LOG_DEBUG, PJ_LOG_LEVEL_PJ_LOG_ERROR, proj_log_func};
use std::ffi::{CStr, c_char, c_int, c_void};
use std::sync::OnceLock;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    Error,
    Debug,
    Trace,
}

static HANDLER: OnceLock<fn(LogLevel, &str)> = OnceLock::new();

/// Send libproj's own messages to `handler`. PROJ's default logger writes them to stderr, which a
/// host application with its own logging has no way to intercept.
///
/// Call once, before creating any [`Proj`](crate::Proj): this replaces the logger on PROJ's default
/// context, and each context created afterwards copies it, but a context that already exists keeps
/// writing to stderr. The first call wins, later ones do nothing.
pub fn set_log_handler(handler: fn(LogLevel, &str)) {
    if HANDLER.set(handler).is_err() {
        return;
    }

    // A null context is PROJ's default context.
    unsafe {
        proj_log_func(
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            Some(log_to_handler),
        )
    };
}

/// Called by libproj on whichever thread produced the message, with a NUL-terminated string that
/// lives only for the duration of the call.
unsafe extern "C" fn log_to_handler(_app_data: *mut c_void, level: c_int, msg: *const c_char) {
    let (Some(handler), false) = (HANDLER.get(), msg.is_null()) else {
        return;
    };

    let level = match level as PJ_LOG_LEVEL {
        PJ_LOG_LEVEL_PJ_LOG_ERROR => LogLevel::Error,
        PJ_LOG_LEVEL_PJ_LOG_DEBUG => LogLevel::Debug,
        _ => LogLevel::Trace,
    };
    handler(level, &unsafe { CStr::from_ptr(msg) }.to_string_lossy());
}
