//! Safe shim directly over the Oculus SDK. More or less directly exposes the Oculus "way" of
//! interfacing with an HMD and handling rendering.

use std::ptr;
use std::default::Default;
use ffi::UnsafeDynamicLibrary;
use std::marker::PhantomData;
use std::rc::Rc;
use std::string::String;
use std::sync::atomic;
use std::vec;

use libc;

use ffi;
use OculusError;
use Eye;
use RenderTarget;

/// Invoke an FFI function with an ovrBool return value, yielding OculusError::SdkError with the
/// supplied message on failure.
macro_rules! ovr_invoke {
    ($x:expr) => {
        if $x == ffi::ovrFalse {
            return Err(OculusError::SdkError("$x failed"));
        }
    }
}

/// Invoke an FFI function with an ovrBool return value, and panic on failure.
macro_rules! ovr_expect {
    ($x:expr) => {
        if $x == ffi::ovrFalse {
            panic!("$x failed");
        }
    }
}

/// RAII wrapper for an Oculus context. Ensures only one Context is active at once in the process.
pub struct Context {
    function_table: ffi::FunctionTable
}

static ACTIVE_CONTEXT: atomic::AtomicBool = atomic::ATOMIC_BOOL_INIT;

const PRODUCT_VERSION: &'static str = "0";
const MAJOR_VERSION: &'static str = "5";

macro_rules! try_load {
    ($x:expr) => {
        match $x {
            Ok(v) => v,
            Err(v) => return Err(OculusError::OculusRuntimeError(v))
        }
    }
}

// Notes from OVR CAPI shim:
//
// Versioned file expectations.
//
// Windows: LibOVRRT<BIT_DEPTH>_<PRODUCT_VERSION>_<MAJOR_VERSION>.dll 
// Example: LibOVRRT64_1_1.dll -- LibOVRRT 64 bit, product 1, major version 1, minor/patch/build
// numbers unspecified in the name.
//
// Mac: LibOVRRT_<PRODUCT_VERSION>.framework/Versions/<MAJOR_VERSION>/LibOVRRT_<PRODUCT_VERSION> 
// We are not presently using the .framework bundle's Current directory to hold the version number.
// This may change.
//
// Linux: libOVRRT<BIT_DEPTH>_<PRODUCT_VERSION>.so.<MAJOR_VERSION> 
// The file on disk may contain a minor version number, but a symlink is used to map this
// major-only version to it.

#[cfg(windows)]
fn load_ovr() -> Result<UnsafeDynamicLibrary, OculusError> { 
    let bits = if cfg!(target_pointer_width = "64") { "64" } else { "32" };
    let lib_name = format!("LibOVRRT{}_{}_{}", bits, PRODUCT_VERSION, MAJOR_VERSION);
    Ok(try_load!(unsafe { UnsafeDynamicLibrary::open(Some(lib_name.as_ref())) }))
}

#[cfg(target_os = "macos")]
fn load_ovr() -> Result<UnsafeDynamicLibrary, OculusError> {
    let lib_name = format!("LibOVRRT_{0}.framework/Versions/{1}/LibOVRRT_{0}", PRODUCT_VERSION, MAJOR_VERSION);
    Ok(try_load!(unsafe { UnsafeDynamicLibrary::open(Some(lib_name.as_ref())) }))
}

#[cfg(target_os = "linux")]
fn load_ovr() -> Result<UnsafeDynamicLibrary, OculusError> {
    let bits = if cfg!(target_pointer_width = "64") { "64" } else { "32" };
    let lib_name = format!("/usr/local/lib/libOVRRT{}_{}.so.{}", bits, PRODUCT_VERSION, MAJOR_VERSION);
    unsafe {
        Ok(try_load!(UnsafeDynamicLibrary::open(Some(lib_name.as_ref()))))
    }
}

impl Context {
    pub fn new() -> Result<Context, OculusError> { 
        let was_active = ACTIVE_CONTEXT.compare_and_swap(false, true, atomic::Ordering::SeqCst);
        if was_active {
            return Err(OculusError::DuplicateContext);
        }

        let lib = try!(load_ovr());
        let function_table = unsafe {
            let function_table = try_load!(ffi::FunctionTable::load(lib));
            let params: ffi::ovrInitParams = Default::default();
            ovr_invoke!(function_table.ovr_Initialize(&params));
            function_table
        };
        Ok(Context {
            function_table: function_table
        })
    }

    pub fn invoker(&self) -> &ffi::FunctionTable {
        &self.function_table
    }
}

impl Drop for Context {
    fn drop(&mut self) {
        unsafe {
            self.invoker().ovr_Shutdown();
            let was_active = ACTIVE_CONTEXT.swap(false, atomic::Ordering::SeqCst);
            assert!(was_active);
        }
    }
}

/// Platform-specific identifier for the OS display representing an Hmd.
#[allow(dead_code)] // Per-platform, only one of these enum values is used.
#[derive(Debug, Eq, PartialEq)]
pub enum HmdDisplayId {
    /// On OS X, this value is the display ID as it would be returned from
    /// `CGGetActiveDisplayList`.
    Numeric(u32),

    /// On Windows, this value is the device name as would be reported by `EnumDisplayDevices`.
    Name(String),

    /// On other platforms, a native identifier for this monitor is not reported by the SDK.
    Unavailable
}

/// Full details about the system display representing this Hmd. These should be used to find the
/// correct monitor on which to prepare a rendering window.
pub struct HmdDisplay {
    /// Identifier for this monitor, if available.
    pub id: HmdDisplayId,

    /// Left edge of the display region.
    pub x: i32,

    /// Top edge of the display region.
    pub y: i32,

    /// Width of the display region.
    pub width: u32,

    /// Height of the display region.
    pub height: u32
}

/// RAII wrapper for an Oculus headset. Provides safe wrappers for access to basic headset
/// metadata and tracking state.
pub struct Hmd {
    native_hmd: *mut ffi::ovrHmdDesc,
    context: Rc<Context>
}

impl Hmd {
    /// Create a new HMD. If `allow_debug` is true and no headset is otherwise detected, a fake
    /// "debug" HMD instance will be returned instead.
    pub fn new(allow_debug: bool, owning_context: Rc<Context>) -> Result<Hmd, OculusError> {
        let hmd = {
            unsafe {
                let h = owning_context.invoker().ovrHmd_Create(0);
                if h.is_null() && allow_debug { 
                    owning_context.invoker().ovrHmd_CreateDebug(ffi::ovrHmd_DK2) 
                } else { 
                    h
                }
            }
        };
        if hmd.is_null() { 
            Err(OculusError::SdkError("ovrHmd_Create failed"))
        } else { 
            Ok(Hmd{ native_hmd: hmd, context: owning_context })
        }
    }

    /// Set HMD caps. Some HMD caps cannot be set using the Oculus SDK; see the Oculus docs for
    /// more details.
    pub fn set_caps(&mut self, caps: ffi::ovrHmdCaps) {
        unsafe {
            self.context.invoker().ovrHmd_SetEnabledCaps(self.native_hmd, caps);
        }
    }

    /// Dismiss the Health and Safety warning automatically displayed by the Oculus runtime. This
    /// should only be dismissed in response to user input; see the Oculus SDK documentation for
    /// details on proper usage.
    pub fn dismiss_hsw(&self) {
        unsafe {
            // Ignore the return value; the underlying implementation is already idempotent, and
            // queues up the dismissal if it isn't ready yet.
            self.context.invoker().ovrHmd_DismissHSWDisplay(self.native_hmd);
        }
    }

    pub fn recenter_pose(&self) {
        unsafe {
            self.context.invoker().ovrHmd_RecenterPose(self.native_hmd);
        }
    }

    /// Enable tracking for this HMD with the specified capabilities.
    pub fn configure_tracking(&mut self, caps: ffi::ovrTrackingCaps) -> Result<(), OculusError> {
        unsafe {
            ovr_invoke!(self.context.invoker().ovrHmd_ConfigureTracking(self.native_hmd, 
                                                                        caps, 
                                                                        ffi::ovrTrackingCaps::empty()));
        }
        Ok(())
    }

    /// Returns true if the HMD is configured to run in Direct mode, or false if it is in Extend
    /// Desktop mode.
    pub fn is_direct(&self) -> bool {
        unsafe {
            let h = &*self.native_hmd;
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
        unsafe {
            let ref native_struct = *self.native_hmd;
            let id = if cfg!(windows) {
                let s = {
                    use std::ffi::CStr;
                    CStr::from_ptr(native_struct.DisplayDeviceName).to_bytes()
                };
                HmdDisplayId::Name(String::from_utf8_lossy(s).into_owned())
            } else if cfg!(target_os = "macos") {
                HmdDisplayId::Numeric(native_struct.DisplayId as u32)
            } else {
                HmdDisplayId::Unavailable
            };
            HmdDisplay {
                id: id,
                x: native_struct.WindowsPos.x,
                y: native_struct.WindowsPos.y,
                width: native_struct.Resolution.w as u32,
                height: native_struct.Resolution.h as u32
            }
        }
    }
}

impl Drop for Hmd {
    fn drop(&mut self) {
        unsafe {
            self.context.invoker().ovrHmd_Destroy(self.native_hmd);
        }
    }
}

/// An active Oculus rendering context associated with an HMD. Only OpenGL is supported. This
/// provides access to the basic metadata necessary to prepare OpenGL framebuffers for drawing.
/// 
/// See `hmd.render_to()` for details on use.
pub struct RenderContext<'a> {
    metadata: RenderMetadata,

    owning_hmd: &'a Hmd,

    // hold on to the render target because we need the window handle to stay alive
    _render_phantom: PhantomData<&'a RenderTarget>
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
    fn native_window<'a>(&'a mut self, native_window: *const libc::c_void) -> &'a mut GlConfigBuilder {
        self.config.Window = native_window;
        self
    }

    #[cfg(not(windows))]
    fn native_window<'a>(&'a mut self, _: *const libc::c_void) -> &'a mut GlConfigBuilder {
        self
    }

    fn build(&self) -> ffi::ovrGLConfig {
        self.config.clone()
    }
}


pub trait CreateRenderContext<'a> {
    fn new(owning_hmd: &'a Hmd,
           target: &'a RenderTarget) -> Result<Self, OculusError>;
}

impl<'a> CreateRenderContext<'a> for RenderContext<'a> {
    /// Create an active Oculus rendering context.
    fn new(owning_hmd: &'a Hmd, 
           target: &'a RenderTarget) -> Result<RenderContext<'a>, OculusError> {
        let (w, h) = owning_hmd.resolution();
        let invoker = owning_hmd.context.invoker();
        let metadata = unsafe {
            let config = GlConfigBuilder::new(w, h, target.get_multisample() as i32)
                .native_window(target.get_native_window())
                .build();

            // TODO: pull in caps as an argument
            let caps = 
                ffi::ovrDistortionCap_TimeWarp |
                ffi::ovrDistortionCap_Overdrive;
            let mut eye_render_desc: [ffi::ovrEyeRenderDesc; 2] = [Default::default(); 2];
            let hmd_data = &*owning_hmd.native_hmd;
            ovr_invoke!(invoker.ovrHmd_ConfigureRendering(owning_hmd.native_hmd,
                                                          &config,
                                                          caps,
                                                          &hmd_data.MaxEyeFov,
                                                          &mut eye_render_desc));
            if owning_hmd.is_direct() {
                ovr_invoke!(invoker.ovrHmd_AttachToWindow(owning_hmd.native_hmd, 
                                                          target.get_native_window(), 
                                                          ptr::null(), 
                                                          ptr::null()));
            }
            RenderMetadata::new(&owning_hmd, &eye_render_desc[0], &eye_render_desc[1])
        };

        Ok(RenderContext {
            owning_hmd: owning_hmd,
            _render_phantom: PhantomData,
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

impl<'a> Drop for RenderContext<'a> {
    fn drop(&mut self) {
        let mut eye_render_desc: [ffi::ovrEyeRenderDesc; 2] = [Default::default(); 2];
        unsafe {
            let invoker = self.owning_hmd.context.invoker();
            let hmd_data = &*self.owning_hmd.native_hmd;
            ovr_expect!(invoker.ovrHmd_ConfigureRendering(self.owning_hmd.native_hmd,
                                                          ptr::null(),
                                                          ffi::ovrDistortionCaps::empty(),
                                                          &hmd_data.MaxEyeFov,
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
            &*hmd.native_hmd
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
            let invoker = hmd.context.invoker();
            PerEyeRenderMetadata {
                texture_size: invoker.ovrHmd_GetFovTextureSize(hmd.native_hmd, 
                                                               eye_index, 
                                                               fov, 
                                                               1f32),
                projection: invoker.ovrMatrix4f_Projection(render_desc.Fov, 
                                                           0.2f32, 
                                                           100f32, 
                                                           ffi::ovrTrue)
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
#[derive(Clone, Copy)]
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
        let invoker = owning_context.owning_hmd.context.invoker();
        unsafe {
            invoker.ovrHmd_BeginFrame(owning_context.owning_hmd.native_hmd, 0);
            invoker.ovrHmd_GetEyePoses(owning_context.owning_hmd.native_hmd,
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

impl<'a> Drop for Frame<'a> {
    fn drop(&mut self) {
        unsafe {
            let invoker = self.owning_context.owning_hmd.context.invoker();
            invoker.ovrHmd_EndFrame(self.owning_context.owning_hmd.native_hmd,
                                    &self.poses,
                                    &self.textures.textures);
        }
    }
}
