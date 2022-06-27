# edgetpu

This library is a rust binding for the C++ [Edge TPU](https://github.com/google-coral/edgetpu).

* `TENSORFLOW_COMMIT` = `d855adfc5a0195788bf5f92c3c7352e638aa1109`

## Requirements

Ensure that you have the libedge tpu library and headers installed on your system. On Linux (Debian),
this means installed both the `libedgetpu1-std` and `libedgetpu-dev` libraries. These libraries are
already included on Coral boards with Mendel installed, but for other Debian systems follow the 
[Debian Packages](https://coral.ai/software/#debian-packages) guide to install these libraries.

Additionally, a valid installation of libclang is required, which can be installed on Debian systems with:
```
sudo apt-get update
sudo apt-get install clang -y
```

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
edgetpu = { git = "https://github.com/ionosnetworks/edgetpu-rs" }
```

## Tensorflow Lite

Working with the native edgetpu APIs requires you to build and link against a specific Tensorflow build. This library handles that, and re-exports [tflite-rs](https://crates.io/crates/tflite) that is build against that version.

