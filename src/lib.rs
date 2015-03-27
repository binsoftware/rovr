//! Idiomatic Rust bindings for the Oculus SDK. Provides access to headset
//! metadata and tracking information, plus helpers for attaching the headset
//! to an OpenGL rendering context.

#![feature(unsafe_destructor, core, collections)]

#[macro_use] extern crate bitflags;
extern crate libc;

#[cfg(feature = "glutin")]
extern crate glutin;

use std::rc::Rc;
use std::fmt;

mod ffi;
mod shim;
pub mod render;

/// Error produced while interacting with a wrapped Oculus device.
#[derive(Clone, Debug)]
pub enum OculusError {
    /// Error while interacting directly with the Oculus SDK. The SDK doesn't provide more detailed
    /// error information, but the included string provides some basic context about what was
    /// happening at the time of failure.
    SdkError(&'static str),

    /// Only one `Context` can be active at a time per process. This error occurs when attempting to
    /// create a second `Context` while a `Context` is already active.
    DuplicateContext
}

impl fmt::Display for OculusError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &OculusError::SdkError(ref description) => f.write_str(description),
            &OculusError::DuplicateContext => f.write_str(
                "Context creation failed because another Context is already active in this process")
        }
    }
}

#[derive(Copy, Clone)]
pub enum Eye {
    Left,
    Right
}

/// Oculus SDK context. Ensures the Oculus SDK has been initialized properly, and serves as a
/// factory for builders that give access to the HMD.
pub struct Context {
    shim_context: Rc<shim::Context>
}

impl Context {
    /// Create a new Oculus SDK context.
    ///
    /// # Failure
    ///
    /// Only one `Context` can be active per process. If a `Context` is already active, this will
    /// fail with `Err(OculsuError::DuplicateContext)`. Note that `Hmd`s hold an internal reference
    /// to their associated context.
    pub fn new() -> Result<Context, OculusError> {
        let shim_context = Rc::new(try!(shim::Context::new()));
        Ok(Context {
            shim_context: shim_context
        })
    }

    /// Create a builder for an HMD.
    pub fn build_hmd(&self) -> HmdBuilder {
        HmdBuilder::new(self.shim_context.clone())
    }
}

/// Options for specifying the enabled tracking capabilities of a headset.
pub struct TrackingOptions {
    track_caps: ffi::ovrTrackingCaps
}

impl TrackingOptions {
    /// `TrackingOptions` with no tracking options enabled.
    pub fn new() -> TrackingOptions {
        TrackingOptions {
            track_caps: ffi::ovrTrackingCaps::empty()
        }
    }

    /// `TrackingOptions` with all supported tracking options enabled.
    pub fn with_all() -> TrackingOptions {
        TrackingOptions {
            track_caps: ffi::ovrTrackingCap_Orientation |
                ffi::ovrTrackingCap_MagYawCorrection |
                ffi::ovrTrackingCap_Position
        }
    }

    /// Enable tracking of head position.
    pub fn position<'f>(&'f mut self) -> &'f mut TrackingOptions {
        self.track_caps.insert(ffi::ovrTrackingCap_Position);
        self
    }

    /// Enable tracking of head orientation.
    pub fn orientation<'f>(&'f mut self) -> &'f mut TrackingOptions {
        self.track_caps.insert(ffi::ovrTrackingCap_Orientation);
        self
    }

    /// Enable yaw drift correction.
    pub fn mag_yaw_correct<'f>(&'f mut self) -> &'f mut TrackingOptions {
        self.track_caps.insert(ffi::ovrTrackingCap_MagYawCorrection);
        self
    }
    
}

/// Builder to construct an HMD. Allows the configuration of HMD settings and tracking
/// capabilities.
pub struct HmdBuilder {
    caps: ffi::ovrHmdCaps,
    track_caps: ffi::ovrTrackingCaps,
    allow_debug: bool,
    owning_context: Rc<shim::Context> 
}

impl HmdBuilder {
    fn new(owning_context: Rc<shim::Context>) -> HmdBuilder {
        let default_caps = ffi::ovrHmdCap_LowPersistence | ffi::ovrHmdCap_DynamicPrediction;
        HmdBuilder { 
            caps: default_caps, 
            track_caps: ffi::ovrTrackingCaps::empty(), 
            allow_debug: false,
            owning_context: owning_context
        }
    }

    /// Disables mirroring of HMD output to the attached window. This may improve
    /// rendering performance slightly.
    pub fn no_mirror<'f>(&'f mut self) -> &'f mut HmdBuilder {
        self.caps.insert(ffi::ovrHmdCap_NoMirrorToWindow);
        self
    }

    /// Turns off HMD screen and output (only if the HMD is not in Direct display
    /// mode).
    pub fn no_display<'f>(&'f mut self) -> &'f mut HmdBuilder {
        self.caps.insert(ffi::ovrHmdCap_DisplayOff);
        self
    }

    /// Disable low persistence.
    pub fn no_low_persistence<'f>(&'f mut self) -> &'f mut HmdBuilder {
        self.caps.remove(ffi::ovrHmdCap_LowPersistence);
        self
    }

    /// Disable dynamic adjustment of tracking prediction based on internally
    /// measured latency.
    pub fn no_dynamic_prediction<'f>(&'f mut self) -> &'f mut HmdBuilder {
        self.caps.remove(ffi::ovrHmdCap_DynamicPrediction);
        self
    }
    
    /// Write directly in pentile color mapping format.
    pub fn direct_pentile<'f>(&'f mut self) -> &'f mut HmdBuilder {
        self.caps.insert(ffi::ovrHmdCap_DirectPentile);
        self
    }

    /// Disable VSync.
    pub fn no_vsync<'f>(&'f mut self) -> &'f mut HmdBuilder {
        self.caps.insert(ffi::ovrHmdCap_NoVSync);
        self
    }

    /// Enable tracking with the specified tracking options.
    pub fn track<'f>(&'f mut self, tracking_options: &TrackingOptions) -> &'f mut HmdBuilder {
        self.track_caps = tracking_options.track_caps;
        self
    }

    /// Allow creation of a dummy "debug" HMD if no other HMD is found.
    pub fn allow_debug<'f>(&'f mut self) -> &'f mut HmdBuilder {
        self.allow_debug = true;
        self
    }

    /// Build the HMD instance. This will begin tracking if tracking is enabled.
    pub fn build(&self) -> Result<Hmd, OculusError> {
        Hmd::new(self.caps, self.track_caps, self.allow_debug, self.owning_context.clone())
    }
}

/// An initialized HMD.
pub struct Hmd {
    shim_hmd: shim::Hmd
}

impl Hmd {
    fn new(caps: ffi::ovrHmdCaps, 
           track_caps: ffi::ovrTrackingCaps,
           allow_debug: bool,
           owning_context: Rc<shim::Context>) -> Result<Hmd, OculusError> {
        let mut shim_hmd = try!(shim::Hmd::new(allow_debug, owning_context));
        shim_hmd.set_caps(caps);
        if !track_caps.is_empty() {
            try!(shim_hmd.configure_tracking(track_caps));
        }
        Ok(Hmd{ shim_hmd: shim_hmd })
    }

    /// Create a `RenderContext` to manually set up rendering for this headset.
    pub unsafe fn init_render(&self, 
                              native_window: *mut libc::c_void) -> Result<render::RenderContext, OculusError> {
        use shim::CreateRenderContext;
        render::RenderContext::new(&self.shim_hmd, 0, native_window)
    }

    /// Create a `RenderContext` to manually set up rendering for this headset, bound to the
    /// specified glutin window.
    #[cfg(feature = "glutin")]
    pub unsafe fn init_render_glutin(&self,
                                     window: &glutin::Window) 
        -> Result<render::RenderContext, OculusError> {
        #[cfg(windows)]
        fn native_window(window: &glutin::Window) -> Option<*mut libc::c_void> {
            unsafe {
                Some(window.platform_window())
            }
        }

        #[cfg(not(windows))]
        fn native_window(_: &glutin::Window) -> Option<*mut libc::c_void> {
            None
        }

        let native_window = native_window(window).unwrap_or(std::ptr::null_mut());
        self.init_render(native_window)
    }

    /// Returns a `(width, height)` pair representing the native resolution of the HMD.
    pub fn resolution(&self) -> (u32, u32) {
        self.shim_hmd.resolution()
    }

    /// Find the glutin monitor for this HMD.
    #[cfg(feature = "glutin")]
    pub fn find_glutin_monitor(&self) -> Option<glutin::MonitorID> {
        let hmd_display_id = self.shim_hmd.get_display();
        for mon in glutin::get_available_monitors() {
            if mon.get_native_identifier() == hmd_display_id {
                return Some(mon);
            }
        }
        None
    }
}

