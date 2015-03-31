//! Idiomatic Rust bindings for the Oculus SDK. Provides access to headset
//! metadata and tracking information, plus helpers for attaching the headset
//! to an OpenGL rendering context.

#![feature(unsafe_destructor, core, collections, std_misc, convert)]

#[macro_use] extern crate bitflags;
extern crate libc;

#[cfg(feature = "glutin")]
extern crate glutin;

use std::rc::Rc;
use std::fmt;

mod ffi;
mod shim;

pub use shim::HmdDisplayId;
pub use shim::HmdDisplay;

pub mod render;
pub mod target;

/// Error produced while interacting with a wrapped Oculus device.
#[derive(Clone, Debug)]
pub enum OculusError {
    /// Error while attempting to find the Oculus runtime. This probably means a supported version
    /// of the runtime is not installed.
    OculusRuntimeError(String),

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
            &OculusError::OculusRuntimeError(ref description) => f.write_str(description),
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

/// A target window to bind headset rendering to.
pub trait RenderTarget {
    /// Number of samples used for MSAA.
    fn get_multisample(&self) -> u32;

    /// The native window handle for this window. This can return null for all platforms except
    /// Windows. The returned handle must be valid with an effective lifetime greater than or equal 
    /// to the lifetime of self.
    unsafe fn get_native_window(&self) -> *const libc::c_void;
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

    /// Create a `RenderContext` for this headset.
    pub fn render_to<'a>(&'a self,
                         target: &'a RenderTarget) -> Result<render::RenderContext, OculusError> {
        use shim::CreateRenderContext;
        render::RenderContext::new(&self.shim_hmd, target)
    }

    /// Returns a `(width, height)` pair representing the native resolution of the HMD.
    pub fn resolution(&self) -> (u32, u32) {
        self.shim_hmd.resolution()
    }

    /// Return details about the display representing this headset.
    pub fn get_display(&self) -> HmdDisplay {
        self.shim_hmd.get_display()
    }
}

