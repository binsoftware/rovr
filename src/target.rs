//! Types to ease integration of windowing libraries with rovr.

#[cfg(feature = "glutin")]
mod glutin_target {
    use glutin;
    use libc;

    use RenderTarget;

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
}

#[cfg(feature = "glutin")]
pub use target::glutin_target::GlutinRenderTarget;

