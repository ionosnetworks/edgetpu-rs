# edgetpu

This library is a rust binding for the C++ [Edge TPU](https://github.com/google-coral/edgetpu).

* `TENSORFLOW_COMMIT` = `d855adfc5a0195788bf5f92c3c7352e638aa1109`

## Requirements

Ensure that you have the libedge tpu library and headers installed on your system. On Linux (Debian),
this means installed both the `libedgetpu1-std` and `libedgetpu-dev` libraries.

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
edgetpu = { git = "https://github.com/ionosnetworks/edgetpu-rs" }
```

## Tensorflow Lite

Working with the native edgetpu APIs requires you to build and link against a specific Tensorflow build. This library handles that, and re-exports [tflite-rs](https://crates.io/crates/tflite) that is build against that version.

