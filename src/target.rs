//! Types to ease integration of windowing libraries with rovr.

#[cfg(feature = "glutin")]
mod glutin_target {
    use glutin;
    use libc;

    use RenderTarget;
    use HmdDisplay;
    use HmdDisplayId;

    /// Wrapper to use a glutin window as a render target.
    pub struct GlutinRenderTarget<'a> {
        window: &'a glutin::Window,
        multisample: u32
    }

    impl<'a> GlutinRenderTarget<'a> {
        /// Create a glutin render target from the specified window. `multisample` should match the
        /// multisampling level used when creating the window.
        pub fn new(window: &'a glutin::Window,
                   multisample: u32) -> GlutinRenderTarget<'a> {
            // wish we didn't need to do this, but currently, glutin won't tell us what multisampling
            // was set to on creation
            GlutinRenderTarget {
                window: window,
                multisample: multisample
            }
        }
    }

    impl<'a> RenderTarget for GlutinRenderTarget<'a> {
        fn get_multisample(&self) -> u32 {
            self.multisample
        }

        #[cfg(windows)]
        unsafe fn get_native_window(&self) -> *const libc::c_void {
            self.window.platform_window()
        }

        // glutin currently panics for non-windows platforms if we even ask for the native window, so
        // don't!
        #[cfg(not(windows))]
        fn get_native_window(&self) -> *const libc::c_void {
            ptr::null()
        }
    }

    impl PartialEq<glutin::NativeMonitorId> for HmdDisplayId {
        fn eq(&self, other: &glutin::NativeMonitorId) -> bool {
            match (self, other) {
                (&HmdDisplayId::Numeric(ref s), &glutin::NativeMonitorId::Numeric(ref o)) => s == o,
                (&HmdDisplayId::Name(ref s), &glutin::NativeMonitorId::Name(ref o)) => s == o,
                _ => false
            }
        }
    }

    impl PartialEq<HmdDisplayId> for glutin::NativeMonitorId {
        fn eq(&self, other: &HmdDisplayId) -> bool {
            other == self
        }
    }

    /// Find the glutin monitor that matches the HmdDisplay details.
    pub fn find_glutin_monitor(display: &HmdDisplay) -> Option<glutin::MonitorID> {
        // TODO: this needs to also compare window position if the id type is Unavailable, but
        // glutin doesn't currently expose this information
        for mon in glutin::get_available_monitors() {
            if mon.get_native_identifier() == display.id {
                return Some(mon);
            }
        }
        None
    }
}

#[cfg(feature = "glutin")]
pub use target::glutin_target::{GlutinRenderTarget, find_glutin_monitor};

