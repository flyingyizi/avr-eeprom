implement embedded_storage trait for avr EEPROM


# example overview
```rust
use avr_eeprom::{Eeprom, embedded_storage::nor_flash::ReadNorFlash};
let  ptr =unsafe{ &*arduino_hal::hal::pac::EEPROM::ptr()};

// instance ep has embedded_storage capability
let mut ep = Eeprom(& ptr);
ufmt::uwriteln!(&mut serial, "eeprom capacity is:{}\r", ep.capacity()).void_unwrap();

//	starting the read operation at start_address(the given address offset), and reading `data.len()` bytes.
const S_DATA_LEN:usize=256;
let mut data = [0_u8; S_DATA_LEN];
let start_address: u32 = 0;
if ep.read(start_address, &mut data).is_err() {
    ufmt::uwriteln!(&mut serial, "read eeprom fail:\r").void_unwrap();
    loop {}
}
```

## example in proteus

see examples dir