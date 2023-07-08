use std::ffi::c_void;

use core_foundation::{
  base::{kCFAllocatorDefault, CFTypeRef},
  string::{kCFStringEncodingUTF8, CFStringCreateWithCString, CFStringRef},
};
use io_kit_sys::{
  actuator::{MTActuatorActuate, MTActuatorClose, MTActuatorCreateFromDeviceID, MTActuatorOpen},
  hid::keys::kIOHIDProductKey,
  ret::{kIOReturnSuccess, IOReturn},
};
use objc_foundation::{INSObject, INSString};

use crate::{
  base::{io_service_matching, IORegistryEntry, IOService},
  registry::keys::{
    kRegistryActuationSupported, kRegistryMTBuiltIn, kRegistryMultitouchId,
    kRegistryTrackpadDeviceName,
  },
};

pub struct MultitouchActuator(Option<CFTypeRef>);

impl Drop for MultitouchActuator {
  fn drop(&mut self) {
    if let Some(actuator_ref) = self.0 {
      unsafe {
        let error: IOReturn = MTActuatorClose(actuator_ref);
        if error != kIOReturnSuccess {
          panic!("Failed to MTActuatorClose: {:?} error: {error}", self.0);
        }
      }
    }
  }
}

impl Default for MultitouchActuator {
  fn default() -> Self {
    Self::new()
  }
}

impl MultitouchActuator {
  pub fn new() -> Self {
    Self(None)
  }

  pub fn from_actuator_ref(actuator_ref: CFTypeRef) -> Self {
    Self(Some(actuator_ref))
  }

  /// Safety: no idea what those params do.
  pub unsafe fn actuate_actuation_id_raw(
    &mut self,
    actuation_id: u32,
    unknown1: u32,
    unknown2: f32,
    unknown3: f32,
  ) -> Result<(), String> {
    let actuator_ref = self.open_actuator()?;

    let error = unsafe {
      MTActuatorActuate(
        actuator_ref,
        actuation_id as i32,
        unknown1,
        unknown2,
        unknown3,
      )
    };

    // In case we failed to actuate with existing actuator, reopen it and try again.
    if error != kIOReturnSuccess {
      eprintln!(
        "Failed to actuate with existing actuator, closing and re-opening...: {:?}, error: {}",
        actuator_ref, error
      );
      self.close_actuator()?;
      let actuator_ref = self.open_actuator()?;

      let error = unsafe {
        MTActuatorActuate(
          actuator_ref,
          actuation_id as i32,
          unknown1,
          unknown2,
          unknown3,
        )
      };
      if error != kIOReturnSuccess {
        return Err(format!(
          "Fail to MTActuatorActuate: {:?}, {}, {}, {}, {} error: {}",
          actuator_ref, actuation_id, 0, 0.0, 0.0, error
        ));
      }
    }
    Ok(())
  }

  pub fn actuate_actuation_id(&mut self, actuation_id: u32) -> Result<(), String> {
    unsafe { self.actuate_actuation_id_raw(actuation_id, 0, 0.0, 0.0) }
  }

  pub fn open_actuator(&mut self) -> Result<*const c_void, &'static str> {
    // See https://github.com/niw/HapticKey/blob/8386f835551cb529aacac6c1937933b0545c72b3/HapticKey/Sources/HTKMultitouchActuator.m#L82
    // for the reverse-engineered low-level haptics APIs.

    if let Some(actuator_ref) = self.0 {
      return Ok(actuator_ref);
    }

    let io_service_dict = io_service_matching(kRegistryTrackpadDeviceName)
      .ok_or("Could not find an IO service named AppleMultitouchDevice.")?;

    let services = IOService::get_matching_services(io_service_dict)
      .map_err(|_| "Failed to get matching services")?;
    for service in services {
      let properties = service
        .create_cf_properties()
        .ok_or("Failed to get CF properties from service.")?;
      // Use the first actuation supported built-in multitouch device, which should be a trackpad.
      let Some(product) = properties
      .find(unsafe { create_cf_string_from_c_string(
        kIOHIDProductKey
      ) as *const c_void})
      else {
        continue;
      };
      let Some(actuation_supported) = properties.find(unsafe {
        create_cf_string_from_c_string(kRegistryActuationSupported) as *const c_void
      }) else {
        continue;
      };
      let Some(mt_built_in) = properties
        .find(unsafe { create_cf_string_from_c_string(kRegistryMTBuiltIn) as *const c_void }) else{
        continue;};

      if actuation_supported.is_null() || mt_built_in.is_null() || product.is_null() {
        println!("Null key 'ActuationSupported' or 'MTBuiltIn' or 'Product'");
        continue;
      }
      unsafe {
        let Some(mt_id_ref) = properties
        .find(create_cf_string_from_c_string(kRegistryMultitouchId) as *const c_void ) else{continue;};
        let mt_id = {
          // okay so this is a disgusting hack -
          // i have no idea what type of object this pointer points to and i can't figure it out
          // but description as NSObject returns the right thing
          // and it's not like this is hot code sooooo whatever :D
          let Some(num) = (*mt_id_ref as *const objc_foundation::NSObject)
            .as_ref() else {
              println!("Null key 'MTBuiltIn'");
              continue;
            };
          if let Ok(number) = (num).description().as_str().parse::<u64>() {
            number
          } else {
            println!("Failed to parse multitouch ID u64 out of string.");
            continue;
          }
        };
        let actuator_ref = MTActuatorCreateFromDeviceID(mt_id);
        if actuator_ref.is_null() {
          return Err("failed to create actuator from multitouch device ID.");
        }
        let open_error = MTActuatorOpen(actuator_ref);
        if open_error != kIOReturnSuccess {
          return Err("Failed to MTActuatorOpen.");
        }
        return Ok(actuator_ref);
      };
    }
    Err("Couldn't find any devices that support haptics.")
  }

  fn close_actuator(&mut self) -> Result<(), String> {
    let Some(actuator_ref) = self.0 else { return Ok(()); };
    unsafe {
      let error = MTActuatorClose(actuator_ref);
      if error != kIOReturnSuccess {
        return Err(format!("Failed to MTActuatorClose: {:?}", actuator_ref));
      }
    }
    Ok(())
  }
}

unsafe fn create_cf_string_from_c_string(c_string: *const i8) -> CFStringRef {
  CFStringCreateWithCString(kCFAllocatorDefault, c_string, kCFStringEncodingUTF8)
}
