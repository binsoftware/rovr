# rovr: VR headsets in Rust

`rovr` provides support for orientation and positional tracking plus rendering to VR headsets using the Oculus SDK. It exposes a safe, Rust-native API for working with the Oculus Rift DK2 and other supported headsets.

`rovr` currently supports version **0.4.4** of the Oculus runtime/SDK, on Windows, OS X, and Linux.

`rovr`'s API is functional, but brand new and should be considered unstable. Feedback and PRs welcome.

# Build notes

rovr should be kept up-to-date with the latest Rust nightlies. The Oculus SDK itself is built as a dynamic library, and will need to be distributed with any resulting applications manually.

