use cpp::*;
use thiserror::Error;

mod tpu_context;

pub use tflite;
pub use tpu_context::{DeviceType, EdgeTpuContext};

use tflite::op_resolver::Registration;

cpp! {{
    #include <string.h>

    #include "edgetpu.h"
}}

#[derive(Debug, Error)]
pub enum EdgeTPUError {
    #[error("failed to open device")]
    OpenFailed,
    #[error("failed to set verbosity")]
    SetVerbosityFailed,
}

pub fn custom_op() -> &'static str {
    unsafe {
        let raw_ptr = cpp!([] -> *mut std::os::raw::c_char as "const char *" {
            return edgetpu::kCustomOp;
        });
        let c_str = std::ffi::CStr::from_ptr(raw_ptr);
        return std::str::from_utf8_unchecked(c_str.to_bytes());
    }
}

pub fn register_custom_op() -> &'static Registration {
    unsafe {
        let handle = cpp!([] -> *mut Registration as "TfLiteRegistration*" {
            return edgetpu::RegisterCustomOp();
        });
        return handle.as_ref().unwrap();
    }
}

pub fn version() -> String {
    unsafe {
        let raw_ptr = cpp!([] -> *mut std::os::raw::c_char as "const char *" {
            auto version = edgetpu::EdgeTpuManager::GetSingleton()->Version();
            char* cstr = new char[version.length()+1];
            strcpy(cstr, version.c_str());
            return cstr;
        });
        let c_str = std::ffi::CString::from_raw(raw_ptr);
        return c_str.into_string().unwrap();
    }
}

pub fn set_verbosity(verbosity: u8) -> Result<(), EdgeTPUError> {
    unsafe {
        let res = cpp!([verbosity as "unsigned char"] -> std::os::raw::c_int as "int" {
            auto res = edgetpu::EdgeTpuManager::GetSingleton()->SetVerbosity((int)verbosity);
            return !!res;
        });
        match res {
            0 => Ok(()),
            _ => Err(EdgeTPUError::OpenFailed),
        }
    }
}

extern "C" {
    fn edgetpu_version() -> *const std::os::raw::c_char;
}

/// For some reason, if I don't use the extern "C" link, Rust will refuse
/// to link the edgetpu libray.
#[doc(hidden)]
#[no_mangle]
pub fn version_force_link() -> String {
    unsafe {
        let v: *const std::os::raw::c_char = edgetpu_version();
        let v2 = std::ffi::CStr::from_ptr(v);
        return String::from(v2.to_string_lossy());
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_custom_op() {
        println!("{:?}", super::custom_op());
    }

    #[test]
    fn test_version() {
        println!("{}", super::version());
    }

    #[test]
    fn test_open() {
        match super::EdgeTpuContext::open_device() {
            Ok(ctx) => assert!(ctx.is_ready(), "if edge tpu was opened, it must be ready"),
            Err(err) => println!("failed to connect to edge tpu: {}", err),
        }
    }

    #[test]
    fn test_enumerate() {
        let devices = super::EdgeTpuContext::enumerate_devices();
        println!("{} TPU Devices", devices.len());
        for device in devices {
            println!(
                "\t{}: {}",
                device.path,
                match device.device_type {
                    super::DeviceType::ApexPCI => "PCI",
                    super::DeviceType::ApexUSB => "USB",
                }
            );
        }
    }

    #[test]
    fn test_options() {
        let options: std::collections::HashMap<String, String> = [
            ("Performance", "Low"),
            ("Usb.AlwaysDfu", "False"),
            ("Usb.MaxBulkInQueueLength", "32"),
        ]
        .iter()
        .map(|(x, y)| (x.to_string(), y.to_string()))
        .collect();
        match super::EdgeTpuContext::open_device_options(super::DeviceType::ApexUSB, "", options) {
            Ok(ctx) => {
                let options = ctx.device_options();
                println!("options map size: {}", options.len());
                options
                    .into_iter()
                    .for_each(|(key, value)| println!("\t{}: {}", key, value));
            }
            Err(err) => println!("failed to connect to edge tpu: {}", err),
        }
    }

    #[test]
    fn test_set_verbosity() {
        assert!(super::set_verbosity(1).is_ok(), "failed to set verbosity");
    }
}
