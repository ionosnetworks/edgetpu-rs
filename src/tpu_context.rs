use std::collections::HashMap;

use cpp::*;

use super::EdgeTPUError;

cpp! {{
    #include <string.h>

    #include "edgetpu.h"
}}

#[derive(PartialEq, Copy, Clone)]
pub enum DeviceType {
    ApexPCI,
    ApexUSB,
}

#[derive(PartialEq, Clone)]
pub struct DeviceRecord {
    pub device_type: DeviceType,
    pub path: String,
}

cpp_class!(unsafe struct InnerEdgeTpuContext as "std::shared_ptr<edgetpu::EdgeTpuContext>");
cpp_class!(unsafe struct EdgeTpuDeviceType as "edgetpu::DeviceType");
cpp_class!(unsafe struct EdgeTpuDeviceOptions as "edgetpu::EdgeTpuManager::DeviceOptions");

#[derive(Clone)]
pub struct EdgeTpuContext {
    inner: InnerEdgeTpuContext,
}

impl EdgeTpuContext {
    pub fn enumerate_devices() -> Vec<DeviceRecord> {
        let mut devices = Vec::new();
        let devices_ptr = &mut devices;

        cpp!(unsafe [devices_ptr as "void *"] {
          const auto& available_tpus = edgetpu::EdgeTpuManager::GetSingleton()->EnumerateEdgeTpu();
          for (auto& device : available_tpus) {
              char* path = new char[device.path.length()+1];
              strcpy(path, device.path.c_str());
              char device_type;
              if (device.type == edgetpu::DeviceType::kApexPci) {
                  device_type = 0;
              } else if (device.type == edgetpu::DeviceType::kApexUsb) {
                  device_type = 1;
              }

              rust!(enumerate_devices_cb [devices_ptr: *mut Vec<DeviceRecord> as "void *", path: *mut std::os::raw::c_char as "const char *", device_type: u8 as "char"] {
                  let path = std::ffi::CString::from_raw(path);
                  let device_type = match device_type {
                      0 => DeviceType::ApexPCI,
                      1 => DeviceType::ApexUSB,
                      _ => panic!("unresolved device type"),
                  };
                  devices_ptr.as_mut().unwrap().push(DeviceRecord{
                      device_type,
                      path: path.to_string_lossy().into(),
                  });
              });
          }
        });

        devices
    }

    pub fn open_device() -> Result<Self, EdgeTPUError> {
        let inner = cpp!(unsafe [] -> InnerEdgeTpuContext as "std::shared_ptr<edgetpu::EdgeTpuContext>" {
          return edgetpu::EdgeTpuManager::GetSingleton()->OpenDevice();
        });

        let ok = cpp!(unsafe [inner as "std::shared_ptr<edgetpu::EdgeTpuContext>"] -> std::os::raw::c_int as "int" {
          return !!inner;
        });

        match ok {
            0 => Err(EdgeTPUError::OpenFailed),
            _ => Ok(Self { inner }),
        }
    }

    pub fn open_device_type(device_type: DeviceType) -> Result<Self, EdgeTPUError> {
        let device_type = match device_type {
            DeviceType::ApexPCI => cpp!(unsafe [] -> EdgeTpuDeviceType as "edgetpu::DeviceType" {
                return edgetpu::DeviceType::kApexPci;
            }),
            DeviceType::ApexUSB => cpp!(unsafe [] -> EdgeTpuDeviceType as "edgetpu::DeviceType" {
                return edgetpu::DeviceType::kApexUsb;
            }),
        };

        let inner = cpp!(unsafe [device_type as "edgetpu::DeviceType"] -> InnerEdgeTpuContext as "std::shared_ptr<edgetpu::EdgeTpuContext>" {
          return edgetpu::EdgeTpuManager::GetSingleton()->OpenDevice(device_type);
        });

        let ok = cpp!(unsafe [inner as "std::shared_ptr<edgetpu::EdgeTpuContext>"] -> std::os::raw::c_int as "int" {
          return !!inner;
        });

        match ok {
            0 => Err(EdgeTPUError::OpenFailed),
            _ => Ok(Self { inner }),
        }
    }

    pub fn open_device_path<T: AsRef<str>>(
        device_type: DeviceType,
        path: T,
    ) -> Result<Self, EdgeTPUError> {
        let device_type = match device_type {
            DeviceType::ApexPCI => cpp!(unsafe [] -> EdgeTpuDeviceType as "edgetpu::DeviceType" {
                return edgetpu::DeviceType::kApexPci;
            }),
            DeviceType::ApexUSB => cpp!(unsafe [] -> EdgeTpuDeviceType as "edgetpu::DeviceType" {
                return edgetpu::DeviceType::kApexUsb;
            }),
        };

        let path_s = std::ffi::CString::new(path.as_ref()).unwrap();
        let path = path_s.as_ptr();

        let inner = cpp!(unsafe [device_type as "edgetpu::DeviceType", path as "const char *"] -> InnerEdgeTpuContext as "std::shared_ptr<edgetpu::EdgeTpuContext>" {
            auto device_path = std::string(path);
            return edgetpu::EdgeTpuManager::GetSingleton()->OpenDevice(device_type, device_path);
        });

        let ok = cpp!(unsafe [inner as "std::shared_ptr<edgetpu::EdgeTpuContext>"] -> std::os::raw::c_int as "int" {
          return !!inner;
        });

        match ok {
            0 => Err(EdgeTPUError::OpenFailed),
            _ => Ok(Self { inner }),
        }
    }

    pub fn open_device_options<T: AsRef<str>>(
        device_type: DeviceType,
        path: T,
        options: HashMap<String, String>,
    ) -> Result<Self, EdgeTPUError> {
        let device_type = match device_type {
            DeviceType::ApexPCI => cpp!(unsafe [] -> EdgeTpuDeviceType as "edgetpu::DeviceType" {
                return edgetpu::DeviceType::kApexPci;
            }),
            DeviceType::ApexUSB => cpp!(unsafe [] -> EdgeTpuDeviceType as "edgetpu::DeviceType" {
                return edgetpu::DeviceType::kApexUsb;
            }),
        };

        let mut device_options = cpp!(unsafe [] -> EdgeTpuDeviceOptions as "edgetpu::EdgeTpuManager::DeviceOptions" {
            edgetpu::EdgeTpuManager::DeviceOptions x;
            x.reserve(4);
            return x;
        });

        for (key, val) in options {
            let ks = std::ffi::CString::new(key.as_str()).unwrap();
            let vs = std::ffi::CString::new(val.as_str()).unwrap();
            let k = ks.as_ptr();
            let v = vs.as_ptr();
            cpp!(unsafe [mut device_options as "edgetpu::EdgeTpuManager::DeviceOptions", k as "const char *", v as "const char *"] {
               device_options.insert({std::string(k), std::string(v)});
            });
        }

        let path_s = std::ffi::CString::new(path.as_ref()).unwrap();
        let path = path_s.as_ptr();

        let inner = cpp!(unsafe [device_type as "edgetpu::DeviceType", path as "const char *", mut device_options as "edgetpu::EdgeTpuManager::DeviceOptions"] -> InnerEdgeTpuContext as "std::shared_ptr<edgetpu::EdgeTpuContext>" {
            auto device_path = std::string(path);
            return edgetpu::EdgeTpuManager::GetSingleton()->OpenDevice(device_type, device_path, device_options);
        });

        let ok = cpp!(unsafe [inner as "std::shared_ptr<edgetpu::EdgeTpuContext>"] -> std::os::raw::c_int as "int" {
          return !!inner;
        });

        match ok {
            0 => Err(EdgeTPUError::OpenFailed),
            _ => Ok(Self { inner }),
        }
    }

    pub fn is_ready(&self) -> bool {
        let inner = self.inner.clone();
        let ok = cpp!(unsafe [inner as "std::shared_ptr<edgetpu::EdgeTpuContext>"] -> std::os::raw::c_int as "int" {
          return !!inner->IsReady();
        });
        ok != 0
    }

    pub fn device_options(&self) -> HashMap<String, String> {
        let inner = self.inner.clone();
        let mut options = HashMap::new();
        let options_ptr = &mut options;
        cpp!(unsafe [inner as "std::shared_ptr<edgetpu::EdgeTpuContext>", options_ptr as "void *"] {
            auto opts = inner->GetDeviceOptions();
            for (std::pair<std::string, std::string> element : opts) {
                char* k = new char[element.first.length()+1];
                strcpy(k, element.first.c_str());
                char* v = new char[element.second.length()+1];
                strcpy(v, element.second.c_str());

                rust!(edgetpu_device_options_cb [options_ptr: *mut HashMap<String, String> as "void *", k: *mut std::os::raw::c_char as "const char *", v: *mut std::os::raw::c_char as "const char *"] {
                    let key = std::ffi::CString::from_raw(k);
                    let value = std::ffi::CString::from_raw(v);
                    options_ptr.as_mut().unwrap().insert(key.to_string_lossy().into(), value.to_string_lossy().into());
                });
            }
        });
        return options;
    }

    pub fn to_external_context(&self) -> tflite::ExternalContext {
        unsafe {
            let inner = self.inner.clone();
            let handle = cpp!([inner as "std::shared_ptr<edgetpu::EdgeTpuContext>"] -> *mut tflite::TfLiteExternalContext as "TfLiteExternalContext*" {
                return inner.get();
            });
            return tflite::ExternalContext::from_raw(handle);
        }
    }
}
