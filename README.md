# rovr: VR headsets in Rust

`rovr` provides support for orientation and positional tracking plus rendering to VR headsets using the Oculus SDK. It exposes a safe, Rust-native API for working with the Oculus Rift DK2 and other supported headsets.

`rovr` currently supports version **0.5.0.1** of the Oculus runtime/SDK, on Windows, OS X, and Linux.

`rovr`'s API is functional, but brand new and should be considered unstable. Feedback and PRs welcome.

# Build notes

`rovr` should track stable Rust, and currently builds and runs against the 1.0.0 Beta. `rovr` dynamically binds to the Oculus runtime, so users of `rovr` programs will need the Oculus runtime installed.

