#![feature(plugin, collections)]

extern crate glutin;
#[macro_use] extern crate glium;
extern crate cgmath;
extern crate libc;
extern crate rovr;

use std::string;
use cgmath::{ToMatrix4, Matrix, Point, FixedArray, Vector};

fn main() {
    use glium::DisplayBuild;

    let context = rovr::Context::new().unwrap();
    let hmd = context.build_hmd()
        .allow_debug()
        .track(&rovr::TrackingOptions::with_all())
        .build()
        .ok().expect("Unable to build HMD");

    let monitor = rovr::target::find_glutin_monitor(&hmd.get_display());
    let builder = match monitor {
        Some(id) => glutin::WindowBuilder::new().with_fullscreen(id),
        None => {
            let (w, h) = hmd.resolution();
            glutin::WindowBuilder::new().with_dimensions(w, h)
        }
    };
    let display = builder
        .with_title(string::String::from_str("Cube"))
        .with_vsync()
        .with_gl_version((4, 1))
        .build_glium()
        .ok().expect("Unable to build Window");

    // NOTE: keeping this window around will cause rebuild to panic; not sure there's a way around
    // this with the current glium mutability/rebuild design
    let window = display.get_window().unwrap();
    let target = rovr::target::GlutinRenderTarget::new(&window, 1);
    let render = hmd.render_to(&target).unwrap();

    let program = basic_shader::compile(&display);
    let (vertex_buffer, index_buffer) = basic_shader::cube(&display);

    let attachments = glium_oculus::Attachments::new(&display, &render);
    let mut surfaces = glium_oculus::Surfaces::new(&display, &attachments);

    display_loop(&display, &attachments, &mut surfaces, |m, surface| {
        use glium::Surface;
        use cgmath::FixedArray;

        let uniforms = uniform! {
            uViewProjectionMatrix: *m.as_fixed()
        };

        let params = glium::DrawParameters {
            backface_culling: glium::BackfaceCullingMode::CullClockWise,
            depth_test: glium::DepthTest::IfLess,
            depth_write: true,
            .. std::default::Default::default()
        };

        surface.clear_color(0f32, 0f32, 0f32, 1f32);
        surface.clear_depth(1f32);
        surface.draw(&vertex_buffer, &index_buffer, &program, &uniforms, &params).unwrap();
    });
}

fn display_loop<'a, F: Fn(&cgmath::Matrix4<f32>, &mut glium::framebuffer::SimpleFrameBuffer)>(
    display: &glium::Display, 
    attachments: &'a glium_oculus::Attachments,
    surfaces: &'a mut glium_oculus::Surfaces<'a>,
    draw: F) {
    use cgmath::Matrix;

    let mut frame_index = 0u32;
    loop {
        {
            let frame = attachments.start_frame();
            for pose in frame.eye_poses() {
                let fixed = cgmath::Vector3::new(0f32, 1f32, 2f32);
                let center = cgmath::Point3::new(0f32, 0f32, 0f32);
                let up = cgmath::Vector3::new(0f32, 1f32, 0f32);

                let camera_position = fixed.add_v(cgmath::Vector3::from_fixed_ref(&pose.position));

                let orientation_mat = {
                    let (orientation_s, ref orientation_v) = pose.orientation;
                    cgmath::Quaternion::from_sv(orientation_s,
                                                *cgmath::Vector3::from_fixed_ref(orientation_v))
                        .to_matrix4()
                        .invert().unwrap()
                };
                let eye_transform = *cgmath::Matrix4::from_fixed_ref(&pose.projection_matrix) *
                    orientation_mat *
                    cgmath::Matrix4::look_at(&cgmath::Point::from_vec(&camera_position),
                                             &center,
                                             &up);

                draw(&eye_transform, surfaces.surface_for_eye(&pose.eye));
            }
        }

        for event in display.poll_events() {
            match event {
                glutin::Event::Closed => return,
                glutin::Event::KeyboardInput(_, _, key) => {
                    attachments.get_render_context().dismiss_hsw();
                    match key {
                        Some(glutin::VirtualKeyCode::Escape) => return,
                        Some(glutin::VirtualKeyCode::R) =>
                            attachments.get_render_context().recenter_pose(),
                        _ => {}
                    }
                },
                _ => {}
            }
        }
        frame_index = frame_index + 1;
    }
}

mod glium_oculus {
    use rovr;
    use glium;
    use glium::texture::{Texture2d, DepthTexture2d};
    use glium::framebuffer::SimpleFrameBuffer;

    pub struct Attachments<'a> {
        render_context: &'a rovr::render::RenderContext<'a>,
        left: PerEyeAttachments,
        right: PerEyeAttachments,
        binding: rovr::render::TextureBinding,
    }

    struct PerEyeAttachments {
        color: Texture2d,
        depth: DepthTexture2d,
    }

    impl<'a> Attachments<'a> {
        pub fn new(display: &glium::Display, 
                   render_context: &'a rovr::render::RenderContext) -> Attachments<'a> {
            use glium::GlObject;

            let left = Attachments::create_attachment(display, render_context, rovr::Eye::Left);
            let right = Attachments::create_attachment(display, render_context, rovr::Eye::Right);
            let binding = render_context.create_binding(left.color.get_id(), right.color.get_id());

            Attachments {
                render_context: render_context,
                left: left,
                right: right,
                binding: binding,
            }
        }

        pub fn start_frame(&self) -> rovr::render::Frame {
            rovr::render::Frame::new(self.render_context, &self.binding)
        }

        pub fn get_render_context(&'a self) -> &'a rovr::render::RenderContext {
            self.render_context
        }

        fn create_attachment(display: &glium::Display, 
                             render_context: &rovr::render::RenderContext, 
                             eye: rovr::Eye) -> PerEyeAttachments {
            let (w, h) = render_context.target_texture_size(&eye);
            let color: Texture2d = Texture2d::empty(display, w, h);
            let depth = DepthTexture2d::empty(display, w, h);

            PerEyeAttachments {
                color: color,
                depth: depth
            }
        }
    }

    pub struct Surfaces<'a> {
        left: SimpleFrameBuffer<'a>,
        right: SimpleFrameBuffer<'a>
    }

    impl<'a> Surfaces<'a> {
        pub fn new(display: &glium::Display, attachments: &'a Attachments) -> Surfaces<'a> {
            let left = SimpleFrameBuffer::with_depth_buffer(display, 
                                                            &attachments.left.color, 
                                                            &attachments.left.depth);
            let right = SimpleFrameBuffer::with_depth_buffer(display, 
                                                             &attachments.right.color, 
                                                             &attachments.right.depth);

            Surfaces {
                left: left,
                right: right
            }
        }

        pub fn surface_for_eye<'b>(&'b mut self, eye: &rovr::Eye) 
            -> &'b mut SimpleFrameBuffer<'a> where 'a: 'b {
            match eye {
                &rovr::Eye::Left => &mut self.left,
                &rovr::Eye::Right => &mut self.right
            }
        }
    }
}

mod basic_shader {
    extern crate glium;

    #[derive(Copy)]
    #[allow(non_snake_case)]
    struct Vertex {
        aPosition: [f32; 3],
        aColor: [f32; 3]
    }

    implement_vertex!(Vertex, aPosition, aColor);

    impl Vertex {
        fn new(position: [f32; 3], color: [f32; 3]) -> Vertex {
            Vertex { aPosition: position, aColor: color }
        }
    }

    pub fn compile(display: &glium::Display) -> glium::Program {
        static VERTEX: &'static str = "
            #version 410

            uniform mat4 uViewProjectionMatrix;

            in vec3 aPosition;
            in vec3 aColor;

            out vec3 vColor;

            void main() {
                gl_Position = uViewProjectionMatrix * vec4(aPosition, 1);
                vColor = aColor;
            }
        ";

        static FRAGMENT: &'static str = "
            #version 410

            in vec3 vColor;
            out vec4 outColor;

            void main() {
                outColor = vec4(vColor, 1);
            }
        ";

        glium::Program::from_source(display, VERTEX, FRAGMENT, None).unwrap()
    }

    pub fn cube(display: &glium::Display) -> (glium::vertex::VertexBufferAny, glium::IndexBuffer) {
        let vertex_buffer = {
            let blue = [0f32, 0f32, 1f32];
            let green = [0f32, 1f32, 0f32];
            let red = [1f32, 0f32, 0f32];
            glium::VertexBuffer::new(display,
                                     vec![
                                        Vertex::new([-0.5f32, -0.5f32,  0.5f32], blue),
                                        Vertex::new([-0.5f32,  0.5f32,  0.5f32], red),
                                        Vertex::new([ 0.5f32, -0.5f32,  0.5f32], green),
                                        Vertex::new([ 0.5f32,  0.5f32,  0.5f32], blue),
                                        Vertex::new([-0.5f32,  0.5f32, -0.5f32], blue),
                                        Vertex::new([ 0.5f32,  0.5f32, -0.5f32], red),
                                        Vertex::new([-0.5f32, -0.5f32, -0.5f32], green),
                                        Vertex::new([ 0.5f32, -0.5f32, -0.5f32], blue)
                                     ]).into_vertex_buffer_any()
        };

        let index_buffer = {
            let triangles = vec![
                // front
                1u32, 0u32, 2u32,
                2u32, 3u32, 1u32,

                // top
                1u32, 3u32, 4u32,
                4u32, 3u32, 5u32,

                // right
                2u32, 7u32, 3u32,
                3u32, 7u32, 5u32,

                // left
                6u32, 0u32, 4u32,
                0u32, 1u32, 4u32,

                // bottom
                7u32, 0u32, 6u32,
                2u32, 0u32, 7u32,

                // back
                7u32, 6u32, 5u32,
                5u32, 6u32, 4u32
            ];
            glium::IndexBuffer::new(display, glium::index::TrianglesList(triangles))
        };

        (vertex_buffer, index_buffer)
    }
}

