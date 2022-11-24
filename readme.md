implement embedded_storage trait for avr EEPROM


# example overview
```rust
use avr_eeprom::prelude::*;
avr_eeprom::impl_eeprom_traditional! {Eeprom,avr_device::atmega328p::EEPROM,1024}
let mut ep = Eeprom {};
ufmt::uwriteln!(&mut serial, "eeprom capacity is:{}\r", ep.capacity()).void_unwrap();
let mut data = [0_u8; 256];
if ep.read(start_address, &mut data).is_err() {
    ufmt::uwriteln!(&mut serial, "read eeprom fail:\r").void_unwrap();
    loop {}
}
```

## example in proteus

see examples dir

## attention

in you app, at least one device is selected in your app that use this crate. 

this sample shows a vaild dependencies description:

[dependencies]

# its related avr-device version is "0.3" 
avr-eeprom ={path = "../../../avr-eeprom/" }

# notes, the version of avr-device used in arduino-hal, must be equal to the version of avr-device used in avr-eeprom.
# below arduino-hal rev, its inner related avr-device version is "0.3"
arduino-hal = {git="https://github.com/rahix/avr-hal",rev="1aacefb335517f85d0de858231e11055d9768cdf",features = ["arduino-nano"]}
