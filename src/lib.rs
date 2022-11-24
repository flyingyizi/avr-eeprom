// 告诉 rustc 只有在禁用 test 标志时才编译 “no-std”
#![cfg_attr(not(test), no_std)]
// 告诉 rustc 只有在启用 test 标志时才编译 “test feature”
#![cfg_attr(test, feature(test))]

pub enum CustomError {
    Bounds,
    Others,
}

mod ep;
pub use avr_device;
pub use embedded_storage;

pub mod prelude {
    pub use embedded_storage::nor_flash::MultiwriteNorFlash as _;
    pub use embedded_storage::nor_flash::NorFlash as _;
    pub use embedded_storage::nor_flash::ReadNorFlash as _;
}

// #[cfg(test)]
// pub mod testutil;

#[cfg(test)]
mod tests {

    #[test]
    fn it_works() {}
}
