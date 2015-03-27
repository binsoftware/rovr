
extern crate rovr;

fn main() {
    let (w, h) = rovr::Context::new().unwrap()
        .build_hmd()
        .allow_debug()
        .build()
        .unwrap()
        .resolution();
    println!("{}x{}", w, h);
}

