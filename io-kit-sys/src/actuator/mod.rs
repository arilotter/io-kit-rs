use core_foundation_sys::base::CFTypeRef;

use crate::ret::IOReturn;

extern "C" {
  pub fn MTActuatorCreateFromDeviceID(device_id: u64) -> CFTypeRef;
  pub fn MTActuatorOpen(actuator_ref: CFTypeRef) -> IOReturn;
  pub fn MTActuatorClose(actuator_ref: CFTypeRef) -> IOReturn;
  /// NOTE: This function has not been fully reverse-engineered.
  /// unknown1, unknown2, and unknown 3 are used to calculate waveform.
  /// unknown1 looks like a 32bit bit field, and is passed to the 4th argument of MTActuationCalculateWaveform().
  /// Passing 0 or 0.0 for these arguments should be okay.
  pub fn MTActuatorActuate(
    actuator_ref: CFTypeRef,
    actuation_id: i32,
    unknown1: u32,
    unknown2: f32,
    unknown3: f32,
  ) -> IOReturn;

  pub fn MTActuatorIsOpen(actuator_ref: CFTypeRef) -> bool;
}

// To find predefined actuation IDs, run the following command:
// $ otool -s __TEXT __tpad_act_plist /System/Library/PrivateFrameworks/MultitouchSupport.framework/Versions/Current/MultitouchSupport|tail -n +3|awk -F'\t' '{print $2}'|xxd -r -p
// This show an embedded plist file in `MultitouchSupport.framework`.
// Valid IDs were, when last checked, 1, 2, 3, 4, 5, 6, 15, and 16.
pub const kIOActuatorActuationTypeNone: i32 = 0;
pub const kIOActuatorActuationTypeWeak: i32 = 3;
pub const kIOActuatorActuationTypeMedium: i32 = 4;
pub const kIOActuatorActuationTypeStrong: i32 = 6;
