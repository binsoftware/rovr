//! Methods for directly working with rendering.
//!
//! # Example
//!
//! End-to-end rendering looks something like this:
//!
//! ```no_run
//! # extern crate rovr;
//! # extern crate libc;
//! # use rovr::{Context, TrackingOptions, Eye};
//! # use rovr::render::Frame;
//! # fn main() {
//! let hmd = Context::new().unwrap()
//!     .build_hmd()
//!     .track(&TrackingOptions::with_all())
//!     .build().unwrap();
//! let (w, h) = hmd.resolution();
//!
//! // <create a window with an OpenGL context based on resolution>
//! // <get native window handle>
//!
//! # let native_window: *mut libc::c_void = std::ptr::null_mut();
//! // This is unsafe because of the lifetime of native_window. If the window is closed before this
//! // render context is destroyed, bad things may happen!
//! let rc = unsafe { hmd.init_render(native_window).unwrap() };
//! let (w_left, h_left) = rc.target_texture_size(&Eye::Left);
//! let (w_right, h_right) = rc.target_texture_size(&Eye::Right);
//!
//! // <create framebuffers with backing textures with these dimensions>
//! // <grab their OpenGL ids>
//!
//! # let (left_tex_id, right_tex_id) = (0, 0);
//! let binding = rc.create_binding(left_tex_id, right_tex_id);
//! loop {
//!     let frame = Frame::new(&rc, &binding);
//!     // draw to framebuffers; frame will finish at end of loop body
//! }
//! # }
//! ```

pub use shim::RenderContext;
pub use shim::TextureBinding;
pub use shim::Quaternion;
pub use shim::Vector3;
pub use shim::Matrix4;
pub use shim::FrameEyePose;
pub use shim::Frame;
