#![allow(dead_code, non_upper_case_globals, non_camel_case_types, non_snake_case)]

mod dynamic_lib;

use libc;
use std::default::Default;
use std::mem;
use std::ptr;

pub use ffi::dynamic_lib::UnsafeDynamicLibrary;

pub type ovrBool = u8;
pub const ovrFalse: ovrBool = 0;
pub const ovrTrue: ovrBool = 1;

pub type ovrResult = i32;
pub const ovrSuccess: ovrResult = 0;
pub const ovrSuccess_NotVisible: ovrResult = 1000;
pub const ovrSuccess_HMDFirmwareMismatch: ovrResult = 4100;
pub const ovrSuccess_TrackerFirmwareMismatch: ovrResult = 4101;
pub const ovrSuccess_ControllerFirmwareMismatch: ovrResult = 4104;
pub fn ovrSuccess(r: ovrResult) -> bool {
    return r > 0;
}
pub fn ovrUnqualifiedSuccess(r: ovrResult) -> bool {
    return r == ovrSuccess;
}
pub fn ovrFailure(r: ovrResult) -> bool {
    return !ovrSuccess(r);
}

#[repr(C)]
pub struct ovrHmdStruct;
pub type ovrSession = *mut ovrHmdStruct;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct ovrPoseStatef {
    pub ThePose: ovrPosef,
    pub AngularVelocity: ovrVector3f,
    pub LinearVelocity: ovrVector3f,
    pub AngularAcceleartion: ovrVector3f,
    pub LinearAcceleration: ovrVector3f,

    pub _PAD0_: [u8; 4],

    pub TimeInSeconds: f64
}

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
        const ovrInit_RequestVersion = 0x00000004
    }
);

#[repr(C)]
pub struct ovrInitParams {
    Flags: u32,
    RequestedMinorVersion: u32,
    LogCallback: *const libc::c_void,
    UserData: usize,
    ConnectionTimeoutMS: u32,

    #[cfg(target_pointer_width = "64")]
    pub _PAD0_: [u8; 4]
}

impl Default for ovrInitParams {
    fn default() -> ovrInitParams {
        ovrInitParams {
            Flags: Default::default(),
            RequestedMinorVersion: Default::default(),
            LogCallback: ptr::null(),
            UserData: Default::default(),
            ConnectionTimeoutMS: Default::default()
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct ovrDetectResult {
    pub IsOculusServiceRunning: ovrBool,
    pub IsOculusHMDConnected: ovrBool
}

pub type ovrHmdType = i32;
pub const ovrHmd_None: ovrHmdType = 0;
pub const ovrHmd_DK1: ovrHmdType = 3;
pub const ovrHmd_DKHD: ovrHmdType = 4;
pub const ovrHmd_DK2: ovrHmdType = 6;
pub const ovrHmd_CB: ovrHmdType = 8;
pub const ovrHmd_Other: ovrHmdType = 9;
pub const ovrHmd_E3_2015: ovrHmdType = 10;
pub const ovrHmd_ES06: ovrHmdType = 11;
pub const ovrHmd_ES09: ovrHmdType = 12;

bitflags!(
    #[repr(C)]
    #[derive(Default)]
    flags ovrHmdCaps: u32 {
        const ovrHmdCap_DebugDevice = 0x0010
        const ovrHmdCap_Writable_Mask = 0x0000,
        const ovrHmdCap_Service_Mask = 0x0000
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

#[repr(C)]
pub struct ovrHmdDesc {
    pub Type: ovrHmdType,

    #[cfg(target_pointer_width = "64")]
    pub _PAD0_: [u8; 4],

    pub ProductName: [u8; 64],
    pub Manufacturer: [u8; 64],
    pub VendorId: i16,
    pub ProductId: i16,
    pub SerialNumber: [u8; 24],
    pub FirmwareMajor: i16,
    pub FirmwareMinor: i16,
    pub CameraFrustumHFovInRadians: f32,
    pub CameraFrustumVFovInRadians: f32,
    pub CameraFrustumNearZInMeters: f32,
    pub CameraFrustumFarZInMeters: f32,
    pub AvailableHmdCaps: ovrHmdCaps,
    pub DefaultHmdCaps: ovrHmdCaps,
    pub AvailableTrackingCaps: ovrTrackingCaps,
    pub DefaultTrackingCaps: ovrTrackingCaps,
    pub DefaultEyeFov: [ovrFovPort; 2],
    pub MaxEyeFov: [ovrFovPort; 2],
    pub Resolution: ovrSizei,
    pub DisplayRefreshRate: f32,

    #[cfg(target_pointer_width = "64")]
    pub _PAD1_: [u8; 4]
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct ovrGraphicsLuid {
    Reserved: [u8; 8]
}

pub type ovrRenderAPIType = u32;
pub const ovrRenderAPI_None: ovrRenderAPIType = 0;
pub const ovrRenderAPI_OpenGL: ovrRenderAPIType = 1;
pub const ovrRenderAPI_Android_GLES: ovrRenderAPIType = 2;
pub const ovrRenderAPI_D3D11: ovrRenderAPIType = 5;
pub const ovrRenderAPI_Count: ovrRenderAPIType = 4;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct ovrTextureHeader {
    API: ovrRenderAPIType,
    TextureSize: ovrSizei
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
    pub Header: ovrTextureHeader,
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
#[derive(Clone, Copy)]
pub struct ovrSwapTextureSet {
    pub Textures: *const ovrTexture,
    pub TextureCount: i32,
    pub CurrentIndex: i32
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
pub struct ovrTrackingState {
    pub HeadPose: ovrPoseStatef,
    pub CameraPose: ovrPosef,
    pub LeveledCamearPose: ovrPosef,
    pub HandPoses: [ovrPoseStatef; 2],
    pub RawSensorData: ovrSensorData,
    pub StatusFlags: u32,
    pub HandStatusFlags: [u32; 2],
    pub LastCameraFrameCounter: u32,

    pub _PAD0_: [u8; 4]
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct ovrViewScaleDesc {
    pub HmdToEyeViewOffset: [ovrVector3f; 2],
    pub HmdSpaceToWorldScaleInMeters: f32
}

pub type ovrLayerType = i32;
pub const ovrLayerType_Disabled: ovrLayerType = 0;
pub const ovrLayerType_EyeFov: ovrLayerType = 1;
pub const ovrLayerType_EyeFovDepth: ovrLayerType = 2;
pub const ovrLayerType_Quad: ovrLayerType = 3;
pub const ovrLayerType_EyeMatrix: ovrLayerType = 5;
pub const ovrLayerType_Direct: ovrLayerType = 6;

bitflags!(
    #[repr(C)]
    #[derive(Default)]
    flags ovrLayerFlags: u32 {
        ovrLayerFlag_HighQuality = 0x01,
        ovrLayerFlag_TextureOriginAtBottomLeft = 0x02,
        ovrLayerFlag_HeadLocked = 0x04
    }
);

#[repr(C)]
#[derive(Clone, Copy)]
pub struct ovrLayerHeader {
    pub Type: ovrLayerType,
    pub Flags: ovrLayerFlags
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct ovrLayerEyeFov {
    pub Header: ovrLayerHeader,
    pub ColorTexture: [*const ovrSwapTextureSet; 2],
    pub Viewport: [ovrRecti; 2],
    pub Fov: [ovrFovPort; 2],
    pub RenderPose: [overPosef; 2],
    pub SensorSampleTime: f64
}

bitflags!(
    #[repr(C)]
    #[derive(Default)]
    flags ovrProjectionModifier: u32 {
        const ovrProjection_None = 0x00,
        const ovrProjection_RightHanded = 0x01,
        const ovrProjection_FarLessTHanNear = 0x02,
        const ovrProjection_FarClipAtInfinity = 0x04,
        const ovrProjection_ClipRangeOpenGL = 0x08
    }
);

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

function_table!(
    fn ovr_Detect(timeoutMsec: i32) -> ovrDetectResult,

    fn ovr_Initialize(params: *const ovrInitParams) -> ovrResult,
    fn ovr_Shutdown() -> (),

    fn ovr_Create(pSession: *mut ovrSession, pLuid: *mut ovrGraphicsLuid) -> ovrResult,
    fn ovr_Destroy(session: ovrSession) -> (),

    fn ovr_GetHmdDesc(session: ovrSession) -> ovrHmdDesc,

    fn ovr_ConfigureTracking(session: ovrSession, 
                             supportedTrackingCaps: ovrTrackingCaps, 
                             requiredTrackingCaps: ovrTrackingCaps) -> ovrResult,
    fn ovr_RecenterPose(session: ovrSession) -> (),

    fn ovr_GetFovTextureSize(session: ovrSession, 
                             eye: i32, 
                             fov: ovrFovPort, 
                             pixelsPerDisplayPixel: f32) -> ovrSizei,
    fn ovr_GetRenderDesc(session: ovrSession,
                         eye: i32,
                         fov: ovrFovPort) -> ovrEyeRenderDesc,

    fn ovr_CreateMirrorTextureGL(session: ovrSession,
                                 format: GLuint,
                                 width: i32,
                                 height: i32,
                                 outMirrorTexture: *mut *mut ovrTexture) -> ovrResult,
    fn ovr_DestroyMirrorTexture(session: ovrSession,
                                mirrorTexture: *ovrTexture) -> (),

    fn ovr_CreateSwapTextureSetGL(session: ovrSession,
                                  format: GLuint,
                                  width: i32,
                                  height: i32,
                                  outTextureSet: *mut *mut ovrSwapTextureSet) -> ovrResult,
    fn ovr_DestroySwapTextureSet(session: ovrSession,
                                 textureSet: *const ovrSwapTextureSet) -> (),

    fn ovr_GetPredictedDisplayTime(session: ovrSession,
                                   frameIndex: i64) -> f64,
    fn ovr_GetTrackingState(session: ovrSession,
                            absTime: f64,
                            latencyMarker: ovrBool) -> ovrTrackingState,
    fn ovr_CalcEyePoses(headPose: ovrPosef,
                        hmdToEyeViewOffset: [ovrVector3f; 2],
                        outEyePoses: mut [ovrPosef; 2]) -> (),
    fn ovr_SubmitFrame(session: ovrSession,
                       frameIndex: i64,
                       viewScaleDesc: *const ovrViewScaleDesc,
                       layerPtrList: *const *const ovrLayerHeader,
                       layerCount: u32) -> ovrResult,

    fn ovrMatrix4f_Projection(fov: ovrFovPort, 
                              znear: f32, 
                              zfar: f32, 
                              projectionModFlags: ovrProjectionModifier) -> ovrMatrix4f
);

