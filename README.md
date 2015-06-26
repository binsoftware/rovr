# rovr: VR headsets in Rust

`rovr` provides support for orientation and positional tracking plus rendering to VR headsets, currently using the Oculus SDK. It exposes a safe, Rust-native API for working with the Oculus Rift DK2 and other supported headsets.

`rovr` currently supports version **0.5.0.1** of the Oculus runtime/SDK, on Windows, OS X, and Linux.

`rovr`'s API is functional, but a work in progress and should be considered unstable as the VR SDK landscape evolves. Feedback and PRs welcome.

# Documentation

Documentation is available [here](http://binsoftware.github.io/rovr/doc/rovr/).

# Build notes

`rovr` dynamically binds to the Oculus runtime, so users of `rovr` programs will need the Oculus runtime installed.

