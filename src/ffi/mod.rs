#![allow(dead_code, non_upper_case_globals, non_camel_case_types, non_snake_case)]

type ovrBool = u8;
pub const ovrTrue: u8 = 1;
pub const ovrFalse: u8 = 0;

mod dynamic_lib;

use libc;
use std::default::Default;
use std::mem;
use std::ptr;

pub use ffi::dynamic_lib::UnsafeDynamicLibrary;

#[repr(C)]
#[derive(Default, Clone, Copy)]
pub struct ovrFovPort {
    pub UpTan: f32,
    pub DownTan: f32,
    pub LeftTan: f32,
    pub RightTan: f32
}

#[repr(C)]
#[derive(Default, Clone, Copy)]
pub struct ovrSizei {
    pub w: i32,
    pub h: i32
}

#[repr(C)]
#[derive(Default, Clone, Copy)]
pub struct ovrVector2i {
    pub x: i32,
    pub y: i32
}

#[repr(C)]
#[derive(Default, Clone, Copy)]
pub struct ovrRecti {
    pub Pos: ovrVector2i,
    pub Size: ovrSizei
}

#[repr(C)]
#[derive(Default, Clone, Copy)]
pub struct ovrVector2f {
    pub x: f32,
    pub y: f32
}

#[repr(C)]
#[derive(Default, Clone, Copy)]
pub struct ovrVector3f {
    pub x: f32,
    pub y: f32,
    pub z: f32
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct ovrMatrix4f {
    pub M: [[f32; 4]; 4]
}

#[repr(C)]
#[derive(Default, Clone, Copy)]
pub struct ovrQuatf {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32
}

#[repr(C)]
#[derive(Default, Clone, Copy)]
pub struct ovrPosef {
    pub Orientation: ovrQuatf,
    pub Position: ovrVector3f
}

impl Default for ovrMatrix4f {
    fn default() -> ovrMatrix4f {
        ovrMatrix4f {
            M: [[1f32, 0f32, 0f32, 0f32],
                [0f32, 1f32, 0f32, 0f32],
                [0f32, 0f32, 1f32, 0f32],
                [0f32, 0f32, 0f32, 1f32]]
        }
    }
}

bitflags!(
    #[repr(C)]
    #[derive(Default)]
    flags ovrInitFlags: u32 {
        const ovrInit_Debug = 0x00000001,
        const ovrInit_ServerOptional = 0x00000002,
        const ovrInit_RequestVersion = 0x00000004,
        const ovrInit_ForceNoDebug = 0x00000008
    }
);

#[repr(C)]
pub struct ovrInitParams {
    Flags: u32,
    RequestedMinorVersion: u32,
    LogCallback: *const libc::c_void,
    ConnectionTimeoutMS: u32
}

impl Default for ovrInitParams {
    fn default() -> ovrInitParams {
        ovrInitParams {
            Flags: Default::default(),
            RequestedMinorVersion: Default::default(),
            LogCallback: ptr::null(),
            ConnectionTimeoutMS: Default::default()
        }
    }
}

pub type ovrHmdType = u32;
pub const ovrHmd_None: ovrHmdType = 0;
pub const ovrHmd_DK1: ovrHmdType = 3;
pub const ovrHmd_DKHD: ovrHmdType = 4;
pub const ovrHmd_DK2: ovrHmdType = 6;
pub const ovrHmd_BlackStar: ovrHmdType = 7;
pub const ovrHmd_CB: ovrHmdType = 8;
pub const ovrHmd_Other: ovrHmdType = 9;

bitflags!(
    #[repr(C)]
    #[derive(Default)]
    flags ovrHmdCaps: u32 {
        const ovrHmdCap_Present = 0x0001,
        const ovrHmdCap_Available = 0x0002,
        const ovrHmdCap_Captured = 0x0004,
        const ovrHmdCap_ExtendDesktop = 0x0008,
        const ovrHmdCap_NoMirrorToWindow = 0x2000,
        const ovrHmdCap_DisplayOff = 0x0040,
        const ovrHmdCap_LowPersistence = 0x0080,
        const ovrHmdCap_DynamicPrediction = 0x0200,
        const ovrHmdCap_NoVSync = 0x1000,
        const ovrHmdCap_Writable_Mask = 0x32C0,
        const ovrHmdCap_Service_Mask = 0x22C0
    }
);

bitflags!(
    #[repr(C)]
    #[derive(Default)]
    flags ovrTrackingCaps: u32 {
        const ovrTrackingCap_Orientation = 0x0010,
        const ovrTrackingCap_MagYawCorrection = 0x0020,
        const ovrTrackingCap_Position = 0x0040,
        const ovrTrackingCap_Idle = 0x0100
    }
);

bitflags!(
    #[repr(C)]
    #[derive(Default)]
    flags ovrDistortionCaps: u32 {
        const ovrDistortionCap_TimeWarp = 0x02,
        const ovrDistortionCap_Vignette = 0x08,
        const ovrDistortionCap_NoRestore = 0x10,
        const ovrDistortionCap_FlipInput = 0x20,
        const ovrDistortionCap_SRGB = 0x40,
        const ovrDistortionCap_Overdrive = 0x80,
        const ovrDistortionCap_HqDistortion = 0x100,
        const ovrDistortionCap_LinuxDevFullscreen = 0x200,
        const ovrDistortionCap_ComputeShader = 0x400,
        const ovrDistortionCap_TimewarpJitDelay = 0x1000,
        const ovrDistortionCap_ProfileNoSpinWaits = 0x10000
    }
);

#[repr(C)] 
pub struct ovrHmdStruct;

#[repr(C)]
pub struct ovrHmdDesc {
    pub Handle: *mut ovrHmdStruct,
    pub Type: ovrHmdType,
    pub ProductName: *const u8,
    pub Manufacturer: *const u8,
    pub VendorId: i16,
    pub ProductId: i16,
    pub SerialNumber: [u8; 24],
    pub FirmwareMajor: i16,
    pub FirmwareMinor: i16,
    pub CameraFrustumHFovInRadians: f32,
    pub CameraFrustumVFovInRadians: f32,
    pub CameraFrustumNearZInMeters: f32,
    pub CameraFrustumFarZInMeters: f32,
    pub HmdCaps: ovrHmdCaps,
    pub TrackingCaps: ovrTrackingCaps,
    pub DistortionCaps: ovrDistortionCaps,
    pub DefaultEyeFov: [ovrFovPort; 2],
    pub MaxEyeFov: [ovrFovPort; 2],
    pub EyeRenderOrder: [u32; 2],
    pub Resolution: ovrSizei,
    pub WindowsPos: ovrVector2i,
    pub DisplayDeviceName: *const i8,
    pub DisplayId: i32
}

pub type ovrRenderAPIType = u32;
pub const ovrRenderAPI_None: ovrRenderAPIType = 0;
pub const ovrRenderAPI_OpenGL: ovrRenderAPIType = 1;
pub const ovrRenderAPI_Android_GLES: ovrRenderAPIType = 2;
pub const ovrRenderAPI_D3D9: ovrRenderAPIType = 3;
pub const ovrRenderAPI_D3D10: ovrRenderAPIType = 4;
pub const ovrRenderAPI_D3D11: ovrRenderAPIType = 5;
pub const ovrRenderAPI_Count: ovrRenderAPIType = 6;

#[repr(C)]
#[cfg(target_os = "linux")]
pub struct _XDisplay;

#[repr(C)]
#[derive(Clone, Copy)]
#[allow(raw_pointer_derive)]
pub struct ovrGLConfig {
    pub API: ovrRenderAPIType,
    pub BackBufferSize: ovrSizei,
    pub Multisample: i32,

    #[cfg(windows)]
    pub Window: *const libc::c_void,
    #[cfg(windows)]
    pub HDC: *const libc::c_void,
    #[cfg(windows)]
    pub _PAD_: [usize; 6],

    #[cfg(target_os = "linux")]
    pub Disp: *const _XDisplay,
    #[cfg(target_os = "linux")]
    pub _PAD_: [usize; 7],

    #[cfg(all(not(windows), not(target_os = "linux")))]
    pub _PAD_: [usize; 8],
}

impl Default for ovrGLConfig {
    fn default() -> ovrGLConfig {
        unsafe {
            mem::zeroed()
        }
    }
}

// We're representing the GL-specific half of the union ovrGLTexture (specifically,
// ovrGLTextureData), whose size is defined by the OVR type ovrTexture.  ovrTexture contains API +
// TextureSize + RenderViewport in its header, plus a ptr-sized 8-element array to pad out the rest
// of the struct for rendering system-specific values. The OpenGL struct contains just one u32, so
// for 32-bit builds we need to pad out the remaining 7 * 4 bytes. The 64-bit version of the native
// struct ends up inheriting additional padding due to alignment. offsetof(TexId) is 28, so the
// "on-books" 92 byte struct gets padded by VC to 96 bytes. If we just add 60 bytes--that is, the
// 8 * 8 - 4 bytes remaining in the platform-specific data region ovr ovrTexture--Rust doesn't pad
// the way VC does. So we manually add the additional 4 bytes by promoting _PAD1_ to a u64.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct ovrGLTexture {
    pub API: ovrRenderAPIType,
    pub TextureSize: ovrSizei,
    pub RenderViewport: ovrRecti,

    pub TexId: u32,

    // See above notes about alignment.
    #[cfg(target_pointer_width = "64")]
    pub _PAD1_: u64,

    pub _PAD2_: [usize; 7],
}

impl Default for ovrGLTexture {
    fn default() -> ovrGLTexture {
        unsafe {
            mem::zeroed()
        }
    }
}

#[repr(C)]
#[derive(Default, Clone, Copy)]
pub struct ovrEyeRenderDesc {
    pub Eye: u32,
    pub Fov: ovrFovPort,
    pub DistortedViewpoint: ovrRecti,
    pub PixelsPerTanAngleAtCenter: ovrVector2f,
    pub HmdToEyeViewOffset: ovrVector3f
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct ovrFrameTiming {
    pub DeltaSeconds: f32,
    pub Pad: f32,
    pub ThisFrameSeconds: f64,
    pub TimewarpPointSeconds: f64,
    pub NextFrameSeconds: f64,
    pub ScanoutMidpointSeconds: f64,
    pub EyeScanoutSeconds: [f64; 2]
}

macro_rules! function_table {
    ( $( fn $func_name:ident( $( $param_name:ident: $param_type:ty ),* ) -> $ret_type:ty ),+ ) => {
        #[allow(non_snake_case)]
        struct FunctionTablePtrs {
            $(
                $func_name: unsafe extern "C" fn($( $param_type, )*) -> $ret_type,
            )*
        }

        pub struct FunctionTable {
            ptrs: FunctionTablePtrs,
            lib: UnsafeDynamicLibrary
        }

        #[allow(non_snake_case)]
        impl FunctionTable {
            pub unsafe fn load(lib: UnsafeDynamicLibrary) -> Result<FunctionTable, String> {
                let ptrs = FunctionTablePtrs {
                    $(
                        $func_name: mem::transmute(
                            try!(lib.symbol::<*const libc::c_void>(stringify!($func_name)))
                        ),
                    )*
                };
                Ok(FunctionTable {
                    ptrs: ptrs,
                    lib: lib
                })
            }

            $(
                #[inline]
                pub unsafe fn $func_name(&self, $( $param_name: $param_type),*) -> $ret_type {
                    (self.ptrs.$func_name)($( $param_name, )*)
                }
            )*
        }
    };
}

bitflags!(
    #[repr(C)]
    #[derive(Default)]
    flags ovrProjectionModifier: u32 {
        const ovrProjection_None = 0x00,
        const ovrProjection_RightHanded = 0x01,
        const ovrProjection_FarLessThanNear = 0x02,
        const ovrProjection_FarClipAtInfinity = 0x04,
        const ovrProjection_ClipRangeOpenGL = 0x08
    }
);

function_table!(
    fn ovr_Initialize(params: *const ovrInitParams) -> ovrBool,
    fn ovr_Shutdown() -> (),

    fn ovrHmd_Create(index: i32) -> *mut ovrHmdDesc,
    fn ovrHmd_CreateDebug(the_type: ovrHmdType) -> *mut ovrHmdDesc,
    fn ovrHmd_Destroy(hmd: *mut ovrHmdDesc) -> (),

    fn ovrHmd_SetEnabledCaps(hmd: *mut ovrHmdDesc, hmdCaps: ovrHmdCaps) -> (),
    fn ovrHmd_DismissHSWDisplay(hmd: *mut ovrHmdDesc) -> ovrBool,
    fn ovrHmd_RecenterPose(hmd: *mut ovrHmdDesc) -> (),
    fn ovrHmd_ConfigureTracking(hmd: *mut ovrHmdDesc, 
                                supportedTrackingCaps: ovrTrackingCaps, 
                                requiredTrackingCaps: ovrTrackingCaps) -> ovrBool,
    fn ovrHmd_ConfigureRendering(hmd: *mut ovrHmdDesc, 
                                 apiConfig: *const ovrGLConfig, 
                                 distortionCaps: ovrDistortionCaps, 
                                 eyeFovIn: *const [ovrFovPort; 2], 
                                 eyeRenderDescOut: *mut [ovrEyeRenderDesc; 2]) -> ovrBool,
    fn ovrHmd_AttachToWindow(hmd: *mut ovrHmdDesc,
                             window: *const libc::c_void,
                             destMirrorRect: *const ovrRecti,
                             sourceRenderTargetRect: *const ovrRecti) -> ovrBool,
    fn ovrHmd_GetFovTextureSize(hmd: *mut ovrHmdDesc, 
                                eye: i32, 
                                fov: ovrFovPort, 
                                pixelsPerDisplayPixel: f32) -> ovrSizei,

    fn ovrHmd_BeginFrame(hmd: *mut ovrHmdDesc, frameIndex: u32) -> ovrFrameTiming,
    fn ovrHmd_GetEyePoses(hmd: *mut ovrHmdDesc, 
                          frameIndex: u32, 
                          hmdToEyeViewOffset: *const [ovrVector3f; 2], 
                          outEyePoses: *mut [ovrPosef; 2], 
                          outHmdTrackingState: *mut libc::c_void) -> (),
    fn ovrHmd_EndFrame(hmd: *mut ovrHmdDesc, 
                       renderPose: *const [ovrPosef; 2], 
                       eyeTexture: *const [ovrGLTexture; 2]) -> (),

    fn ovrMatrix4f_Projection(fov: ovrFovPort, 
                              znear: f32, 
                              zfar: f32, 
                              projectionModFlags: ovrProjectionModifier) -> ovrMatrix4f
);

