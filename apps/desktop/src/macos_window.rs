use objc2::{AllocAnyThread, MainThreadMarker};
use objc2_app_kit::{NSApplication, NSImage, NSView};
use objc2_foundation::NSData;
use std::{ffi::c_void, ptr::NonNull};

/// Read the state of GPUI's current native window.
///
/// # Safety
/// `view` must be a live NSView, accessed on the application's main thread.
pub unsafe fn is_minimized(view: NonNull<c_void>) -> bool {
    let view = unsafe { &*view.as_ptr().cast::<NSView>() };
    view.window().is_some_and(|window| window.isMiniaturized())
}

pub fn set_application_icon(png: &[u8]) {
    let Some(main_thread) = MainThreadMarker::new() else {
        return;
    };
    let data = NSData::with_bytes(png);
    if let Some(image) = NSImage::initWithData(NSImage::alloc(), &data) {
        let app = NSApplication::sharedApplication(main_thread);
        unsafe {
            app.setApplicationIconImage(Some(&image));
        }
    }
}
