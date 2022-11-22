// 告诉 rustc 只有在禁用 test 标志时才编译 “no-std”
#![cfg_attr(not(test), no_std)]
// 告诉 rustc 只有在启用 test 标志时才编译 “test feature”
#![cfg_attr(test, feature(test))]

mod avreeprom;

pub use avreeprom::Eeprom;
// re export
/// re-export
pub use arduino_hal;
/// re-export
pub use avr_device;
pub use embedded_storage;

// #[cfg(test)]
// pub mod testutil;

#[cfg(test)]
mod tests {

    #[test]
    fn it_works() {}
}
