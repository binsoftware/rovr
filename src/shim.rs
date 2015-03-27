//! Safe shim directly over the Oculus SDK. More or less directly exposes the Oculus "way" of
//! interfacing with an HMD and handling rendering.

use std::ptr;
use std::sync::atomic;
use std::rc::Rc;
use std::default::Default;
use std::string::String;
#[cfg(windows)]
use std::str;
use std::vec::{self, Vec};

use libc;

#[cfg(feature = "glutin")]
use glutin;

use ffi;
use OculusError;
use Eye;

/// Invoke an FFI function with an ovrBool return value, yielding OculusError::SdkError with the
/// supplied message on failure.
macro_rules! ovr_invoke {
    ($x:expr) => {
        if $x == '\0' {
            return Err(OculusError::SdkError("$x failed"));
        }
    }
}

/// Invoke an FFI function with an ovrBool return value, and panic on failure.
macro_rules! ovr_expect {
    ($x:expr) => {
        if $x == '\0' {
            panic!("$x failed");
        }
    }
}

/// RAII wrapper for an Oculus context. Ensures only one Context is active at once in the process.
pub struct Context;

static ACTIVE_CONTEXT: atomic::AtomicBool = atomic::ATOMIC_BOOL_INIT;

impl Context {
    pub fn new() -> Result<Context, OculusError> { 
        let was_active = ACTIVE_CONTEXT.compare_and_swap(false, true, atomic::Ordering::SeqCst);
        if was_active {
            return Err(OculusError::DuplicateContext);
        }
        unsafe {
            ovr_invoke!(ffi::ovr_Initialize());
        }
        Ok(Context)
    }
}

impl Drop for Context {
    fn drop(&mut self) {
        unsafe {
            ffi::ovr_Shutdown();
            let was_active = ACTIVE_CONTEXT.swap(false, atomic::Ordering::SeqCst);
            assert!(was_active);
        }
    }
}

#[allow(dead_code)] // Per-platform, only one of these enum values is used.
#[derive(Debug, Eq, PartialEq)]
pub enum HmdDisplay {
    Numeric(u32),
    Name(String)
}

#[cfg(feature = "glutin")]
impl PartialEq<glutin::NativeMonitorId> for HmdDisplay {
    fn eq(&self, other: &glutin::NativeMonitorId) -> bool {
        match (self, other) {
            (&HmdDisplay::Numeric(ref s), &glutin::NativeMonitorId::Numeric(ref o)) => s == o,
            (&HmdDisplay::Name(ref s), &glutin::NativeMonitorId::Name(ref o)) => s == o,
            _ => false
        }
    }
}

#[cfg(feature = "glutin")]
impl PartialEq<HmdDisplay> for glutin::NativeMonitorId {
    fn eq(&self, other: &HmdDisplay) -> bool {
        other == self
    }
}

/// RAII wrapper for an Oculus headset. Provides safe wrappers for access to basic headset
/// metadata and tracking state.
pub struct Hmd {
    native_hmd: *mut ffi::ovrHmdDesc,
    _owning_context: Rc<Context>
}

impl Hmd {
    /// Create a new HMD. If `allow_debug` is true and no headset is otherwise detected, a fake
    /// "debug" HMD instance will be returned instead.
    pub fn new(allow_debug: bool, owning_context: Rc<Context>) -> Result<Hmd, OculusError> {
        let hmd = {
            unsafe {
                let h = ffi::ovrHmd_Create(0);
                if h.is_null() && allow_debug { ffi::ovrHmd_CreateDebug() } else { h }
            }
        };
        if hmd.is_null() { 
            Err(OculusError::SdkError("ovrHmd_Create failed"))
        } else { 
            Ok(Hmd{ native_hmd: hmd, _owning_context: owning_context })
        }
    }

    /// Set HMD caps. Some HMD caps cannot be set using the Oculus SDK; see the Oculus docs for
    /// more details.
    pub fn set_caps(&mut self, caps: ffi::ovrHmdCaps) {
        unsafe {
            ffi::ovrHmd_SetEnabledCaps(self.native_hmd, caps);
        }
    }

    /// Dismiss the Health and Safety warning automatically displayed by the Oculus runtime. This
    /// should only be dismissed in response to user input; see the Oculus SDK documentation for
    /// details on proper usage.
    pub fn dismiss_hsw(&self) {
        unsafe {
            // Ignore the return value; the underlying implementation is already idempotent, and
            // queues up the dismissal if it isn't ready yet.
            ffi::ovrHmd_DismissHSWDisplay(self.native_hmd);
        }
    }

    pub fn recenter_pose(&self) {
        unsafe {
            ffi::ovrHmd_RecenterPose(self.native_hmd);
        }
    }

    /// Enable tracking for this HMD with the specified capabilities.
    pub fn configure_tracking(&mut self, caps: ffi::ovrTrackingCaps) -> Result<(), OculusError> {
        unsafe {
            ovr_invoke!(ffi::ovrHmd_ConfigureTracking(self.native_hmd, 
                                                      caps, 
                                                      ffi::ovrTrackingCaps::empty()));
        }
        Ok(())
    }

    /// Returns true if the HMD is configured to run in Direct mode, or false if it is in Extend
    /// Desktop mode.
    pub fn is_direct(&self) -> bool {
        unsafe {
            let h = self.native_hmd.as_ref().unwrap();
            !h.HmdCaps.contains(ffi::ovrHmdCap_ExtendDesktop)
        }
    }

    /// Native resolution of the full HMD display.
    pub fn resolution(&self) -> (u32, u32) {
        unsafe {
            let ref native_struct = *self.native_hmd;
            (native_struct.Resolution.w as u32, native_struct.Resolution.h as u32)
        }
    }

    /// Get the native display identifier for the monitor represented by this HMD.
    pub fn get_display(&self) -> HmdDisplay {
        #[cfg(not(windows))]
        fn get_display_impl(native_hmd: &ffi::ovrHmdDesc) -> HmdDisplay {
            HmdDisplay::Numeric(native_hmd.DisplayId as u32)
        }

        #[cfg(windows)]
        fn get_display_impl(native_hmd: &ffi::ovrHmdDesc) -> HmdDisplay {
            let s = unsafe {
                use std::ffi::CStr;
                str::from_utf8(CStr::from_ptr(native_hmd.DisplayDeviceName).to_bytes())
                    .unwrap_or("")
            };
            HmdDisplay::Name(String::from_str(s))
        }

        unsafe {
            let ref native_struct = *self.native_hmd;
            get_display_impl(native_struct)
        }
    }
}

impl Drop for Hmd {
    fn drop(&mut self) {
        unsafe {
            ffi::ovrHmd_Destroy(self.native_hmd);
        }
    }
}

/// An active Oculus rendering context associated with an HMD. Only OpenGL is supported. This
/// provides access to the basic metadata necessary to prepare OpenGL framebuffers for drawing.
/// 
/// See `hmd.init_render()` for details on use.
pub struct RenderContext<'a> {
    metadata: RenderMetadata,

    owning_hmd: &'a Hmd,
}

struct GlConfigBuilder {
    config: ffi::ovrGLConfig
}

impl GlConfigBuilder {
    fn new(w: u32, h: u32, multisample: i32) -> GlConfigBuilder {
        GlConfigBuilder {
            config: ffi::ovrGLConfig {
                API: ffi::ovrRenderAPI_OpenGL,
                BackBufferSize: ffi::ovrSizei { w: w as i32, h: h as i32 },
                Multisample: multisample,
                .. Default::default()
            }
        }
    }

    #[cfg(windows)]
    fn native_window<'a>(&'a mut self, native_window: *mut libc::c_void) -> &'a mut GlConfigBuilder {
        self.config.Window = native_window;
        self
    }

    #[cfg(not(windows))]
    fn native_window<'a>(&'a mut self, _: *mut libc::c_void) -> &'a mut GlConfigBuilder {
        self
    }

    fn build(&self) -> ffi::ovrGLConfig {
        self.config.clone()
    }
}


pub trait CreateRenderContext<'a> {
    fn new(owning_hmd: &'a Hmd,
           multisample: i32,
           native_window: *mut libc::c_void) -> Result<Self, OculusError>;
}

impl<'a> CreateRenderContext<'a> for RenderContext<'a> {
    /// Create an active Oculus rendering context. **native_window** is only necessary on the
    /// Windows platform, and is ignored otherwise.
    fn new(owning_hmd: &'a Hmd, 
           multisample: i32, 
           native_window: *mut libc::c_void) -> Result<RenderContext<'a>, OculusError> {
        let (w, h) = owning_hmd.resolution();
        let config = GlConfigBuilder::new(w, h, multisample)
            .native_window(native_window)
            .build();

        // TODO: pull in caps as an argument
        let caps = ffi::ovrDistortionCap_Chromatic |
            ffi::ovrDistortionCap_TimeWarp |
            ffi::ovrDistortionCap_Overdrive;
        let mut eye_render_desc: [ffi::ovrEyeRenderDesc; 2] = [Default::default(); 2];
        unsafe {
            ovr_invoke!(ffi::ovrHmd_ConfigureRendering(owning_hmd.native_hmd,
                                                       &config,
                                                       caps,
                                                       &owning_hmd.native_hmd.as_ref().unwrap().MaxEyeFov,
                                                       &mut eye_render_desc));
        }
        if owning_hmd.is_direct() {
            unsafe {
                ovr_invoke!(ffi::ovrHmd_AttachToWindow(owning_hmd.native_hmd, 
                                                       native_window, 
                                                       ptr::null(), 
                                                       ptr::null()));
            }
        }

        let metadata = RenderMetadata::new(&owning_hmd, &eye_render_desc[0], &eye_render_desc[1]);
        Ok(RenderContext {
            owning_hmd: owning_hmd,
            metadata: metadata
        })
    }
}

impl<'a> RenderContext<'a> {
    /// Dismiss the Health and Safety warning automatically displayed by the Oculus runtime. This
    /// should only be dismissed in response to user input; see the Oculus SDK documentation for
    /// details on proper usage.
    pub fn dismiss_hsw(&self) {
        self.owning_hmd.dismiss_hsw();
    }

    /// Recenter the headset, using the current orientation and position as the origin.
    pub fn recenter_pose(&self) {
        self.owning_hmd.recenter_pose();
    }

    /// Return a `(width, height)` tuple containing the suggested size for a render target for the
    /// given eye.
    pub fn target_texture_size(&self, eye: &Eye) -> (u32, u32) {
        match eye {
            &Eye::Left => self.metadata.left.resolution(),
            &Eye::Right => self.metadata.right.resolution()
        }
    }

    /// Create a texture binding given a pair of OpenGL texture IDs for the left and right eye,
    /// respectively. The left and right textures should be of the size suggested by
    /// `target_texture_size`.
    pub fn create_binding(&self, tex_id_left: u32, tex_id_right: u32) -> TextureBinding {
        TextureBinding::new((self.metadata.left.texture_size, tex_id_left),
                            (self.metadata.right.texture_size, tex_id_right))
    }
}

#[unsafe_destructor]
impl<'a> Drop for RenderContext<'a> {
    fn drop(&mut self) {
        let mut eye_render_desc: [ffi::ovrEyeRenderDesc; 2] = [Default::default(); 2];
        unsafe {
            ovr_expect!(ffi::ovrHmd_ConfigureRendering(self.owning_hmd.native_hmd,
                                                       ptr::null(),
                                                       ffi::ovrDistortionCaps::empty(),
                                                       &self.owning_hmd.native_hmd.as_ref().unwrap().MaxEyeFov,
                                                       &mut eye_render_desc));
        }
    }
}

/// Metadata describing the desired rendering parameters for both eyes.
struct RenderMetadata {
    left: PerEyeRenderMetadata,
    right: PerEyeRenderMetadata,
    offsets: [ffi::ovrVector3f; 2]
}

impl RenderMetadata {
    fn new(hmd: &Hmd,
           left: &ffi::ovrEyeRenderDesc,
           right: &ffi::ovrEyeRenderDesc) -> RenderMetadata {
        let h = unsafe {
            hmd.native_hmd.as_ref().unwrap()
        };
        RenderMetadata {
            left: PerEyeRenderMetadata::new(hmd, left, 0, h.MaxEyeFov[0]),
            right: PerEyeRenderMetadata::new(hmd, right, 1, h.MaxEyeFov[1]),
            offsets: [left.HmdToEyeViewOffset, right.HmdToEyeViewOffset]
        }
    }
}

struct PerEyeRenderMetadata {
    texture_size: ffi::ovrSizei,
    projection: ffi::ovrMatrix4f
}

impl PerEyeRenderMetadata {
    fn new(hmd: &Hmd, 
           render_desc: &ffi::ovrEyeRenderDesc, 
           eye_index: i32,
           fov: ffi::ovrFovPort) -> PerEyeRenderMetadata {
        unsafe {
            PerEyeRenderMetadata {
                texture_size: ffi::ovrHmd_GetFovTextureSize(hmd.native_hmd, eye_index, fov, 1f32),
                projection: ffi::ovrMatrix4f_Projection(render_desc.Fov, 0.2f32, 100f32, '1')
            }
        }
    }

    fn resolution(&self) -> (u32, u32) {
        (self.texture_size.w as u32, self.texture_size.h as u32)
    }
}

/// Texture binding, representing a registered pair of OpenGL textures that should serve as render
/// targets for per-eye viewpoints. Create with `RenderContext::create_binding()`
pub struct TextureBinding {
    textures: [ffi::ovrGLTexture; 2]
}

impl TextureBinding {
    fn new(left_pair: (ffi::ovrSizei, u32), right_pair: (ffi::ovrSizei, u32)) -> TextureBinding {
        fn texture_struct(size: ffi::ovrSizei, id: u32) -> ffi::ovrGLTexture {
            let viewport = ffi::ovrRecti {
                Pos: ffi::ovrVector2i { x: 0i32, y: 0i32 },
                Size: size
            };
            ffi::ovrGLTexture {
                API: ffi::ovrRenderAPI_OpenGL,
                TextureSize: size,
                RenderViewport: viewport,
                TexId: id,
                .. Default::default()
            }
        }

        TextureBinding {
            textures: [texture_struct(left_pair.0, left_pair.1),
                       texture_struct(right_pair.0, right_pair.1)]
        }
    }

}

/// A quaternion. The first element of the tuple is the w value, and the array contains x, y, and z
/// values.
pub type Quaternion = (f32, [f32; 3]);

/// A 3-dimensional vector, with (in order) x, y, and z components.
pub type Vector3 = [f32; 3];

/// A 4x4 matrix, by convention in column-major format.
pub type Matrix4 = [[f32; 4]; 4];

/// A single eye's pose for a frame.
#[derive(Copy)]
pub struct FrameEyePose {
    pub eye: Eye,
    pub orientation: Quaternion,
    pub position: Vector3,
    pub projection_matrix: Matrix4
}

/// A single frame. All OpenGL rendering to both eyes' frame buffers should happen while this
/// object is alive. When going out of scope, the Oculus SDK will complete the rendering process,
/// including post-processing and any necessary buffer swapping.
pub struct Frame<'a> {
    owning_context: &'a RenderContext<'a>,
    textures: &'a TextureBinding,
    poses: [ffi::ovrPosef; 2]
}

impl<'a> Frame<'a> {
    /// Start a frame.
    pub fn new(owning_context: &'a RenderContext, 
               texture_binding: &'a TextureBinding) -> Frame<'a> {
        let mut poses: [ffi::ovrPosef; 2] = [Default::default(); 2];
        unsafe {
            ffi::ovrHmd_BeginFrame(owning_context.owning_hmd.native_hmd, 0);
            ffi::ovrHmd_GetEyePoses(owning_context.owning_hmd.native_hmd,
                                    0,
                                    &owning_context.metadata.offsets,
                                    &mut poses,
                                    ptr::null_mut());
        }

        Frame {
            owning_context: owning_context,
            textures: texture_binding,
            poses: poses
        }
    }

    /// Get an iterable list of eye poses that should be drawn for this frame. These are returned
    /// in the suggested rendering order.
    pub fn eye_poses(&self) -> vec::IntoIter<FrameEyePose> {
        unsafe {
            let ref hmd_struct = *self.owning_context.owning_hmd.native_hmd;
            let mut poses = Vec::<FrameEyePose>::with_capacity(2);
            for i in hmd_struct.EyeRenderOrder.iter() {
                let (eye, ref pm) = match i {
                    &0u32 => (Eye::Left, self.owning_context.metadata.left.projection.M),
                    &1u32 => (Eye::Right, self.owning_context.metadata.right.projection.M),
                    _ => panic!("Too many eyes!")
                };
                let position = self.poses[*i as usize].Position;
                let orientation = self.poses[*i as usize].Orientation;

                // note that we must invert projection_matrix to column major
                poses.push(FrameEyePose {
                    eye: eye,
                    orientation: (orientation.w, [orientation.x, orientation.y, orientation.z]),
                    position: [position.x, position.y, position.z],
                    projection_matrix: [[pm[0][0], pm[1][0], pm[2][0], pm[3][0]],
                                        [pm[0][1], pm[1][1], pm[2][1], pm[3][1]],
                                        [pm[0][2], pm[1][2], pm[2][2], pm[3][2]],
                                        [pm[0][3], pm[1][3], pm[2][3], pm[3][3]]]
                });
            }
            poses.into_iter()
        }
    }
}

#[unsafe_destructor]
impl<'a> Drop for Frame<'a> {
    fn drop(&mut self) {
        unsafe {
            ffi::ovrHmd_EndFrame(self.owning_context.owning_hmd.native_hmd,
                                 &self.poses,
                                 &self.textures.textures);
        }
    }
}
