pub type IORegistryEntryOptionsType = u32;
pub const kIORegistryEntryOptionsTypeNone: IORegistryEntryOptionsType = 0x00;

// Trackpad Haptics Property Keys
pub const kRegistryTrackpadDeviceName: *const ::std::os::raw::c_char =
  b"AppleMultitouchDevice\x00" as *const [u8; 22usize] as *const ::std::os::raw::c_char;
pub const kRegistryActuationSupported: *const ::std::os::raw::c_char =
  b"ActuationSupported\x00" as *const [u8; 19usize] as *const ::std::os::raw::c_char;
pub const kRegistryMTBuiltIn: *const ::std::os::raw::c_char =
  b"MT Built-In\x00" as *const [u8; 12usize] as *const ::std::os::raw::c_char;
pub const kRegistryMultitouchId: *const ::std::os::raw::c_char =
  b"Multitouch ID\x00" as *const [u8; 14usize] as *const ::std::os::raw::c_char;
