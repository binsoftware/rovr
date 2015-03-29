#![allow(dead_code)]
#![allow(non_upper_case_globals)]

#[allow(non_camel_case_types)]
type ovrBool = char;

use libc;
use std::default::Default;
use std::ptr;

#[repr(C)]
#[derive(Default, Clone, Copy)]
#[allow(non_snake_case)]
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
#[allow(non_snake_case)]
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
#[allow(non_snake_case)]
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
#[allow(non_snake_case)]
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
    flags ovrHmdCaps: u32 {
        const ovrHmdCap_Present = 0x0001,
        const ovrHmdCap_Available = 0x0002,
        const ovrHmdCap_Captured = 0x0004,
        const ovrHmdCap_ExtendDesktop = 0x0008,
        const ovrHmdCap_NoMirrorToWindow = 0x2000,
        const ovrHmdCap_DisplayOff = 0x0040,
        const ovrHmdCap_LowPersistence = 0x0080,
        const ovrHmdCap_DynamicPrediction = 0x0200,
        const ovrHmdCap_DirectPentile = 0x0400,
        const ovrHmdCap_NoVSync = 0x1000,
        const ovrHmdCap_Writable_Mask = 0x32F0,
        const ovrHmdCap_Service_Mask = 0x22F0
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
        const ovrDistortionCap_Chromatic = 0x01,
        const ovrDistortionCap_TimeWarp = 0x02,
        const ovrDistortionCap_Vignette = 0x08,
        const ovrDistortionCap_NoRestore = 0x10,
        const ovrDistortionCap_FlipInput = 0x20,
        const ovrDistortionCap_SRGB = 0x40,
        const ovrDistortionCap_Overdrive = 0x80,
        const ovrDistortionCap_HqDistortion = 0x100,
        const ovrDistortionCap_LinuxDevFullscreen = 0x200,
        const ovrDistortionCap_ComputeShader = 0x400,
        const ovrDistortionCap_ProfileNoTimewarpSpinWaits = 0x10000
    }
);

#[repr(C)] 
pub struct ovrHmdStruct;

#[repr(C)]
#[allow(non_snake_case)]
pub struct ovrHmdDesc {
    pub Handle: *mut ovrHmdStruct,
    pub Type: u32,
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

bitflags!(
    #[repr(C)]
    #[derive(Default)]
    flags ovrRenderAPIType: u32 {
        const ovrRenderAPI_None = 0,
        const ovrRenderAPI_OpenGL = 1,
        const ovrRenderAPI_Android_GLES = 2,
        const ovrRenderAPI_D3D9 = 3,
        const ovrRenderAPI_D3D10 = 4,
        const ovrRenderAPI_D3D11 = 5,
        const ovrRenderAPI_Count = 6
    }
);

#[repr(C)]
#[cfg(target_os = "linux")]
pub struct _XDisplay;

#[repr(C)]
#[derive(Clone, Copy)]
#[allow(non_snake_case, raw_pointer_derive)]
pub struct ovrGLConfig {
    pub API: ovrRenderAPIType,
    pub BackBufferSize: ovrSizei,
    pub Multisample: i32,

    #[cfg(windows)]
    pub Window: *const libc::c_void,

    #[cfg(windows)]
    pub HDC: *const libc::c_void,

    #[cfg(target_os = "linux")]
    pub Disp: *const _XDisplay,
}

impl Default for ovrGLConfig {
    #[cfg(windows)]
    fn default() -> ovrGLConfig {
        ovrGLConfig {
            API: Default::default(),
            BackBufferSize: Default::default(),
            Multisample: Default::default(),
            Window: ptr::null_mut::<libc::c_void>(),
            HDC: ptr::null_mut::<libc::c_void>()
        }
    }

    #[cfg(target_os = "linux")]
    fn default() -> ovrGLConfig {
        ovrGLConfig {
            API: Default::default(),
            BackBufferSize: Default::default(),
            Multisample: Default::default(),
            Disp: ptr::null_mut::<_XDisplay>()
        }
    }

    #[cfg(all(not(windows), not(target_os = "linux")))]
    fn default() -> ovrGLConfig {
        ovrGLConfig {
            API: Default::default(),
            BackBufferSize: Default::default(),
            Multisample: Default::default(),
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
#[allow(non_snake_case)]
pub struct ovrGLTexture {
    pub API: ovrRenderAPIType,
    pub TextureSize: ovrSizei,
    pub RenderViewport: ovrRecti,
    pub _PAD0_: u32,

    pub TexId: u32,

    #[cfg(target_pointer_width = "64")]
    pub _PAD1_: [u32; 15], // 8 * ptr_size total padding, -32bits for the texId

    #[cfg(target_pointer_width = "32")]
    pub _PAD1_: [u32; 7], 
}

// Because there's no Default impl for arrays, we need to fill this whole thing in ourselves
impl Default for ovrGLTexture {
    #[cfg(target_pointer_width = "64")]
    fn default() -> ovrGLTexture {
        ovrGLTexture {
            API: Default::default(),
            TextureSize: Default::default(),
            RenderViewport: Default::default(),
            _PAD0_: Default::default(),
            TexId: Default::default(),
            _PAD1_: [0u32; 15]
        }
    }

    #[cfg(target_pointer_width = "32")]
    fn default() -> ovrGLTexture {
        ovrGLTexture {
            API: Default::default(),
            TextureSize: Default::default(),
            RenderViewport: Default::default(),
            _PAD0_: Default::default(),
            TexId: Default::default(),
            _PAD1_: [0u32; 7]
        }
    }
}

#[repr(C)]
#[derive(Default, Clone, Copy)]
#[allow(non_snake_case)]
pub struct ovrEyeRenderDesc {
    pub Eye: u32,
    pub Fov: ovrFovPort,
    pub DistortedViewpoint: ovrRecti,
    pub PixelsPerTanAngleAtCenter: ovrVector2f,
    pub HmdToEyeViewOffset: ovrVector3f
}

#[repr(C)]
#[derive(Default, Clone, Copy)]
pub struct ovrTrackingState;

#[repr(C)]
#[derive(Clone, Copy)]
#[allow(non_snake_case)]
pub struct ovrFrameTiming {
    pub DeltaSeconds: f32,
    pub ThisFrameSeconds: f64,
    pub TimewarpPointSeconds: f64,
    pub NextFrameSeconds: f64,
    pub ScanoutMidpointSeconds: f64,
    pub EyeScanoutSeconds: [f64; 2]
}

#[link(name="ovr")]
extern "C" {
    pub fn ovr_Initialize() -> ovrBool;
    pub fn ovr_Shutdown();

    pub fn ovrHmd_Create(index: i32) -> *mut ovrHmdDesc;
    pub fn ovrHmd_CreateDebug() -> *mut ovrHmdDesc;
    pub fn ovrHmd_Destroy(hmd: *mut ovrHmdDesc);

    pub fn ovrHmd_SetEnabledCaps(hmd: *mut ovrHmdDesc, hmdCaps: ovrHmdCaps);
    pub fn ovrHmd_DismissHSWDisplay(hmd: *mut ovrHmdDesc) -> ovrBool;
    pub fn ovrHmd_RecenterPose(hmd: *mut ovrHmdDesc);
    pub fn ovrHmd_ConfigureTracking(hmd: *mut ovrHmdDesc, supportedTrackingCaps: ovrTrackingCaps, requiredTrackingCaps: ovrTrackingCaps) -> ovrBool;
    pub fn ovrHmd_ConfigureRendering(hmd: *mut ovrHmdDesc, 
                                     apiConfig: *const ovrGLConfig, 
                                     distortionCaps: ovrDistortionCaps, 
                                     eyeFovIn: *const [ovrFovPort; 2], 
                                     eyeRenderDescOut: *mut [ovrEyeRenderDesc; 2]) -> ovrBool;
    pub fn ovrHmd_AttachToWindow(hmd: *mut ovrHmdDesc,
                                 window: *const libc::c_void,
                                 destMirrorRect: *const ovrRecti,
                                 sourceRenderTargetRect: *const ovrRecti) -> ovrBool;
    pub fn ovrHmd_GetFovTextureSize(hmd: *mut ovrHmdDesc, eye: i32, fov: ovrFovPort, pixelsPerDisplayPixel: f32) -> ovrSizei;

    pub fn ovrHmd_BeginFrame(hmd: *mut ovrHmdDesc, frameIndex: u32) -> ovrFrameTiming;
    pub fn ovrHmd_GetEyePoses(hmd: *mut ovrHmdDesc, 
                              frameIndex: u32, 
                              hmdToEyeViewOffset: *const [ovrVector3f; 2], 
                              outEyePoses: *mut [ovrPosef; 2], 
                              outHmdTrackingState: *mut ovrTrackingState);
    pub fn ovrHmd_EndFrame(hmd: *mut ovrHmdDesc, renderPose: *const [ovrPosef; 2], eyeTexture: *const [ovrGLTexture; 2]);

    pub fn ovrMatrix4f_Projection(fov: ovrFovPort, znear: f32, zfar: f32, rightHanded: ovrBool) -> ovrMatrix4f;
}

