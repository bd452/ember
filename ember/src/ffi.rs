//! C ABI (`--features capi`).
//!
//! Exposes the subset of Ember needed to build a reactive screen from
//! another compiled language: integer/string signals, labels, buttons, stacks,
//! and the app/event loop. Structural helpers (`slot`/`for_each`) and custom
//! components remain Rust-only for now — the ABI can grow.
//!
//! Conventions:
//! - Handles are opaque pointers created by `ember_*_new` and released by the
//!   matching `ember_*_free`. All calls must happen on the thread that created the
//!   app.
//! - Strings crossing the boundary are UTF-8 `const char*`, copied immediately.
//! - Every entry point is wrapped in [`catch_unwind`] so a Rust panic never
//!   unwinds into the caller's frames (it becomes a no-op / error return).
//!
//! The header (`include/ember.h`) is generated from this module by cbindgen.

// Handle types are named in snake_case to match idiomatic C typedefs in the
// generated header.
#![allow(non_camel_case_types)]

use std::cell::RefCell;
use std::ffi::{c_char, c_int, c_void, CStr};
use std::panic::{catch_unwind, AssertUnwindSafe};

use crate::app::{App, ExitHandle};
use crate::component::{BuildCtx, Component};
use crate::node::{IntoNode, Node};
use crate::render::mock::MockRenderer;
use crate::signal::Signal;
use crate::widget::{Align, AnyWidget, Button, Label, Stack};

// --- opaque handle types (named for cbindgen / the header) ---

/// Opaque handle to an `i64` signal.
pub struct ember_signal_i64 {
    inner: Signal<i64>,
}
/// Opaque handle to a text label widget.
pub struct ember_label {
    inner: Label,
}
/// Opaque handle to a button widget.
pub struct ember_button {
    inner: Button,
}
/// Opaque handle to a stack container.
pub struct ember_stack {
    inner: Stack,
}
/// Opaque handle to the running application.
pub struct ember_app {
    inner: App<MockRenderer>,
}

/// A tap callback and its opaque user data.
struct TapCb {
    cb: extern "C" fn(*mut c_void),
    user: *mut c_void,
}

thread_local! {
    static LAST_ERROR: RefCell<Option<std::ffi::CString>> = const { RefCell::new(None) };
}

fn set_error(msg: &str) {
    LAST_ERROR.with(|e| {
        *e.borrow_mut() = std::ffi::CString::new(msg).ok();
    });
}

/// Runs `f`, converting any panic into `fallback` and recording an error.
fn guard<T>(fallback: T, f: impl FnOnce() -> T) -> T {
    match catch_unwind(AssertUnwindSafe(f)) {
        Ok(v) => v,
        Err(_) => {
            set_error("panic in ember FFI call");
            fallback
        }
    }
}

unsafe fn cstr<'a>(p: *const c_char) -> &'a str {
    if p.is_null() {
        return "";
    }
    CStr::from_ptr(p).to_str().unwrap_or("")
}

/// Returns the library version as a static C string.
#[no_mangle]
pub extern "C" fn ember_version() -> *const c_char {
    // Version is nul-terminated at compile time.
    concat!(env!("CARGO_PKG_VERSION"), "\0").as_ptr() as *const c_char
}

/// Returns the last error message on this thread, or NULL. Valid until the next
/// failing FFI call on the same thread.
#[no_mangle]
pub extern "C" fn ember_last_error() -> *const c_char {
    LAST_ERROR.with(|e| {
        e.borrow()
            .as_ref()
            .map(|s| s.as_ptr())
            .unwrap_or(std::ptr::null())
    })
}

// --- i64 signal ---

#[no_mangle]
pub extern "C" fn ember_signal_i64_new(initial: i64) -> *mut ember_signal_i64 {
    guard(std::ptr::null_mut(), || {
        Box::into_raw(Box::new(ember_signal_i64 {
            inner: Signal::new(initial),
        }))
    })
}

#[no_mangle]
pub extern "C" fn ember_signal_i64_get(sig: *const ember_signal_i64) -> i64 {
    guard(0, || {
        let Some(sig) = (unsafe { sig.as_ref() }) else {
            set_error("ember_signal_i64_get: null handle");
            return 0;
        };
        sig.inner.get()
    })
}

#[no_mangle]
pub extern "C" fn ember_signal_i64_set(sig: *const ember_signal_i64, value: i64) {
    guard((), || {
        let Some(sig) = (unsafe { sig.as_ref() }) else {
            set_error("ember_signal_i64_set: null handle");
            return;
        };
        sig.inner.set(value);
    })
}

#[no_mangle]
pub extern "C" fn ember_signal_i64_free(sig: *mut ember_signal_i64) {
    guard((), || {
        if !sig.is_null() {
            drop(unsafe { Box::from_raw(sig) });
        }
    })
}

// --- label ---

#[no_mangle]
pub extern "C" fn ember_label_new(text: *const c_char) -> *mut ember_label {
    guard(std::ptr::null_mut(), || {
        let text = unsafe { cstr(text) };
        Box::into_raw(Box::new(ember_label {
            inner: Label::new(text),
        }))
    })
}

#[no_mangle]
pub extern "C" fn ember_label_set_text(label: *const ember_label, text: *const c_char) {
    guard((), || {
        let Some(label) = (unsafe { label.as_ref() }) else {
            set_error("ember_label_set_text: null handle");
            return;
        };
        let text = unsafe { cstr(text) };
        label.inner.set_text(text);
    })
}

/// Binds a label's text to an `i64` signal, formatted with `printf`-style `%lld`
/// replaced by the value via a fixed prefix/suffix. To keep the ABI simple the
/// binding renders `<prefix><value><suffix>`.
#[no_mangle]
pub extern "C" fn ember_label_bind_i64(
    label: *const ember_label,
    sig: *const ember_signal_i64,
    prefix: *const c_char,
    suffix: *const c_char,
) {
    guard((), || {
        let (Some(l), Some(s)) = (unsafe { label.as_ref() }, unsafe { sig.as_ref() }) else {
            set_error("ember_label_bind_i64: null handle");
            return;
        };
        let label = l.inner.clone();
        let sig = s.inner.clone();
        let prefix = unsafe { cstr(prefix) }.to_owned();
        let suffix = unsafe { cstr(suffix) }.to_owned();
        // Apply the current value now.
        label.set_text(format!("{prefix}{}{suffix}", sig.get()));
        // Then keep it in sync. Not tied to a component scope: the label owns
        // the subscription for its lifetime by leaking the disposable (typical
        // for C-owned widgets).
        let target = label.clone();
        sig.observe(move |v| target.set_text(format!("{prefix}{v}{suffix}")))
            .forget();
    })
}

#[no_mangle]
pub extern "C" fn ember_label_free(label: *mut ember_label) {
    guard((), || {
        if !label.is_null() {
            drop(unsafe { Box::from_raw(label) });
        }
    })
}

// --- button ---

#[no_mangle]
pub extern "C" fn ember_button_new(title: *const c_char) -> *mut ember_button {
    guard(std::ptr::null_mut(), || {
        let title = unsafe { cstr(title) };
        Box::into_raw(Box::new(ember_button {
            inner: Button::new(title),
        }))
    })
}

/// Registers a tap handler. `user_data` is passed back to `cb` on each tap.
#[no_mangle]
pub extern "C" fn ember_button_on_tap(
    button: *const ember_button,
    cb: extern "C" fn(*mut c_void),
    user_data: *mut c_void,
) {
    guard((), || {
        let Some(button) = (unsafe { button.as_ref() }) else {
            set_error("ember_button_on_tap: null handle");
            return;
        };
        let button = button.inner.clone();
        let tap = TapCb {
            cb,
            user: user_data,
        };
        // Move the callback into the Rust closure. Raw pointer is Send-unsafe but
        // the whole library is single-threaded by contract.
        let _ = button.clone().on_tap(move || (tap.cb)(tap.user));
    })
}

#[no_mangle]
pub extern "C" fn ember_button_free(button: *mut ember_button) {
    guard((), || {
        if !button.is_null() {
            drop(unsafe { Box::from_raw(button) });
        }
    })
}

// --- stack ---

#[no_mangle]
pub extern "C" fn ember_vstack_new(spacing: c_int) -> *mut ember_stack {
    guard(std::ptr::null_mut(), || {
        Box::into_raw(Box::new(ember_stack {
            inner: Stack::vertical(spacing, Align::Fill),
        }))
    })
}

#[no_mangle]
pub extern "C" fn ember_hstack_new(spacing: c_int) -> *mut ember_stack {
    guard(std::ptr::null_mut(), || {
        Box::into_raw(Box::new(ember_stack {
            inner: Stack::horizontal(spacing, Align::Fill),
        }))
    })
}

#[no_mangle]
pub extern "C" fn ember_stack_push_label(stack: *const ember_stack, label: *const ember_label) {
    guard((), || {
        let (Some(stack), Some(label)) = (unsafe { stack.as_ref() }, unsafe { label.as_ref() })
        else {
            set_error("ember_stack_push_label: null handle");
            return;
        };
        stack.inner.push(AnyWidget::Label(label.inner.clone()));
    })
}

#[no_mangle]
pub extern "C" fn ember_stack_push_button(stack: *const ember_stack, button: *const ember_button) {
    guard((), || {
        let (Some(stack), Some(button)) = (unsafe { stack.as_ref() }, unsafe { button.as_ref() })
        else {
            set_error("ember_stack_push_button: null handle");
            return;
        };
        stack.inner.push(AnyWidget::Button(button.inner.clone()));
    })
}

#[no_mangle]
pub extern "C" fn ember_stack_free(stack: *mut ember_stack) {
    guard((), || {
        if !stack.is_null() {
            drop(unsafe { Box::from_raw(stack) });
        }
    })
}

// --- app ---

/// A component that mounts a pre-built stack (the C side assembles the tree,
/// then hands its root here).
struct RootStack(Option<Stack>);
impl Component for RootStack {
    fn build(&mut self, _ctx: &mut BuildCtx) -> Node {
        self.0.take().expect("root built once").into_node()
    }
}

/// Creates the app from a root stack and a screen size, using the mock renderer
/// (the FBInk renderer requires the `fbink` feature; the C ABI ships with it in
/// the cross build, and this constructor is swapped accordingly). Consumes the
/// stack handle.
#[no_mangle]
pub extern "C" fn ember_app_new(
    root: *mut ember_stack,
    width: c_int,
    height: c_int,
) -> *mut ember_app {
    guard(std::ptr::null_mut(), || {
        if root.is_null() {
            set_error("ember_app_new: null root");
            return std::ptr::null_mut();
        }
        let root = unsafe { Box::from_raw(root) };
        let size = crate::geometry::Size::new(width, height);
        let app = App::new(
            Box::new(RootStack(Some(root.inner.clone()))),
            MockRenderer::new(size),
            ExitHandle::new(),
        );
        Box::into_raw(Box::new(ember_app { inner: app }))
    })
}

/// Renders one frame. Returns 0 on success, -1 on error.
#[no_mangle]
pub extern "C" fn ember_app_render_frame(app: *mut ember_app) -> c_int {
    guard(-1, || {
        let Some(app) = (unsafe { app.as_mut() }) else {
            set_error("ember_app_render_frame: null handle");
            return -1;
        };
        match app.inner.render_frame() {
            Ok(()) => 0,
            Err(e) => {
                set_error(&format!("render_frame: {e}"));
                -1
            }
        }
    })
}

/// Dispatches a tap at (`x`, `y`) in screen pixels. Returns 1 if a handler ran.
#[no_mangle]
pub extern "C" fn ember_app_tap_at(app: *mut ember_app, x: c_int, y: c_int) -> c_int {
    guard(0, || {
        let Some(app) = (unsafe { app.as_mut() }) else {
            set_error("ember_app_tap_at: null handle");
            return 0;
        };
        app.inner.tap_at(crate::geometry::Point::new(x, y)) as c_int
    })
}

#[no_mangle]
pub extern "C" fn ember_app_free(app: *mut ember_app) {
    guard((), || {
        if !app.is_null() {
            drop(unsafe { Box::from_raw(app) });
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;
    use std::sync::atomic::{AtomicI32, Ordering};

    static TAP_COUNT: AtomicI32 = AtomicI32::new(0);
    extern "C" fn on_tap(_user: *mut c_void) {
        TAP_COUNT.fetch_add(1, Ordering::SeqCst);
    }

    #[test]
    fn build_tree_and_dispatch_tap_over_ffi() {
        let sig = ember_signal_i64_new(0);
        assert_eq!(ember_signal_i64_get(sig), 0);
        ember_signal_i64_set(sig, 5);
        assert_eq!(ember_signal_i64_get(sig), 5);

        let title = CString::new("Tap").unwrap();
        let button = ember_button_new(title.as_ptr());
        ember_button_on_tap(button, on_tap, std::ptr::null_mut());

        let stack = ember_vstack_new(8);
        ember_stack_push_button(stack, button);

        let app = ember_app_new(stack, 400, 600);
        assert_eq!(ember_app_render_frame(app), 0);

        // Tap the button's area (top of the vstack).
        TAP_COUNT.store(0, Ordering::SeqCst);
        let hit = ember_app_tap_at(app, 20, 15);
        assert_eq!(hit, 1);
        assert_eq!(TAP_COUNT.load(Ordering::SeqCst), 1);

        ember_app_free(app);
        ember_button_free(button);
        ember_signal_i64_free(sig);
    }

    #[test]
    fn null_handles_do_not_crash() {
        assert_eq!(ember_signal_i64_get(std::ptr::null()), 0);
        ember_signal_i64_free(std::ptr::null_mut());
        assert!(!ember_version().is_null());
    }
}
