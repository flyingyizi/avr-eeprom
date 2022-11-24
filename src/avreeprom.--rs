//! implement embedded_storage trait for avr EEPROM
//!
//! #example
//! ```no_run
//! use avr_eeprom::{Eeprom, embedded_storage::nor_flash::ReadNorFlash};
//! let  ptr =unsafe{ &*arduino_hal::hal::pac::EEPROM::ptr()};
//! // instance ep has embedded_storage capability
//! let mut ep = Eeprom(& ptr);
//! // ufmt::uwriteln!(&mut serial, "eeprom capacity is:{}\r", ep.capacity()).void_unwrap();
//! //	starting the read operation at start_address(the given address offset), and reading `data.len()` bytes.
//! const S_DATA_LEN:usize=256;
//! let mut data = [0_u8; S_DATA_LEN];
//! let start_address: u32 = 0;
//! if ep.read(start_address, &mut data).is_err() {
//!     //ufmt::uwriteln!(&mut serial, "read eeprom fail:\r").void_unwrap();
//!     loop {}
//! }
//! ```

pub use storage::Eeprom;

mod storage {
    use embedded_storage::nor_flash::{MultiwriteNorFlash, NorFlash, ReadNorFlash};

    type CustomError = ();

    impl<'a> ReadNorFlash for Eeprom<'a> {
        type Error = CustomError;

        const READ_SIZE: usize = 1;

        fn read(&mut self, offset: u32, bytes: &mut [u8]) -> Result<(), Self::Error> {
            let len = bytes.len();
            let mut offset = offset as u16;
            for i in 0..len {
                bytes[i] = self.eeprom_get_char(offset);
                offset += 1;
            }

            Ok(())
        }

        fn capacity(&self) -> usize {
            cfg_if::cfg_if! {
                if #[cfg(any( feature = "mcuavr-arduino-diecimila",
                            feature = "mcuavr-nano168" ))] {
                    512  //atmega168
                }
                else if #[cfg(any( feature = "mcuavr-arduino-leonardo",
                                   feature = "mcuavr-sparkfun-promicro" ))] {
                    1024 //atmega32u4
                    //– 512Bytes/1K Bytes Internal EEPROM (ATmega16U4/ATmega32U4)
                }
                else if #[cfg(any( feature = "mcuavr-arduino-mega2560"))] {
                    4*1024 //atmega2560
                }
                else if #[cfg(any( feature = "mcuavr-arduino-nano",
                                   feature = "mcuavr-arduino-uno",
                                   feature = "mcuavr-arduino-pro", ))] {
                    1024 //atmega328p
                }
                else if #[cfg(any( feature = "mcuavr-trinket"))] {
                    512 //attiny85
                    // – 128/256/512 Bytes In-System Programmable EEPROM (ATtiny25/45/85)
                }
                else {
                    compile_error!(
                        "This crate requires you to specify your target Arduino board as a feature.
                
                    Please select one of the following
                
                    * mcuavr-arduino-diecimila
                    * mcuavr-arduino-leonardo
                    * mcuavr-arduino-mega2560
                    * mcuavr-arduino-nano
                    * mcuavr-arduino-uno
                    * mcuavr-sparkfun-promicro
                    * mcuavr-trinket-pro
                    * mcuavr-trinket
                    * mcuavr-nano168
                    "
                    );

                }
            }
        }
    }

    impl<'a> NorFlash for Eeprom<'a> {
        const WRITE_SIZE: usize = 1;

        const ERASE_SIZE: usize = 1;

        fn erase(&mut self, from: u32, to: u32) -> Result<(), Self::Error> {
            avr_device::interrupt::free(|_cs| {
                for i in from..to {
                    self.wait_ready();

                    // set EEPROM address
                    self.0.eear.write(|w| unsafe { w.bits(i as u16) });

                    // Now we know that all bits should be erased.
                    self.0.eecr.write(|w| {
                        w.eempe().set_bit(); // Set Master Write Enable bit
                        w.eepm().val_0x01() // ...and Erase-only mode..
                    });
                    self.0.eecr.write(|w| w.eepe().set_bit()); // Start Erase-only operation.
                }
            });

            Ok(())
        }

        fn write(&mut self, offset: u32, bytes: &[u8]) -> Result<(), Self::Error> {
            self.program(offset as usize, bytes.iter())
        }
    }

    // AVR supports multiple writes
    impl<'a> MultiwriteNorFlash for Eeprom<'a> {}

    use arduino_hal::hal::pac::eeprom::RegisterBlock;
    use core::iter::Iterator;

    /// it implement [embedded_storage::nor_flash] trait for avr EEPROM
    ///
    /// #example
    /// ```no_run
    /// use avr_eeprom::{Eeprom, embedded_storage::nor_flash::ReadNorFlash};
    /// let  ptr =unsafe{ &*arduino_hal::hal::pac::EEPROM::ptr()};
    /// // instance ep has embedded_storage capability
    /// let ep = Eeprom(& ptr);
    /// let len = ep.capacity();
    /// ```
    pub struct Eeprom<'a>(pub &'a RegisterBlock);

    impl Eeprom<'_> {
        /// Program bytes with offset into flash memory,
        fn program<'a, I>(&mut self, mut offset: usize, bytes: I) -> Result<(), CustomError>
        where
            I: Iterator<Item = &'a u8>,
        {
            for i in bytes {
                avr_device::interrupt::free(|_cs| {
                    self.eeprom_put_char(offset as u16, *i);
                    offset += 1;
                });
            }

            Ok(())
        }

        #[inline]
        fn wait_ready(&self) {
            // unsafe {
            // wait until EEPE become to zero by hardware. on other word,
            //Wait for completion of previous write.
            while self.0.eecr.read().eepe().bit_is_set() {}
            // }
        }
        fn eeprom_get_char(&mut self, address: u16) -> u8 {
            // let ptr = arduino_hal::hal::pac::EEPROM::ptr();

            unsafe {
                self.wait_ready();
                // set EEPROM address register
                self.0.eear.write(|w| w.bits(address));
                //Start EEPROM read operation
                self.0.eecr.write(|w| w.eere().set_bit());
            }
            // Return the byte read from EEPROM
            self.0.eedr.read().bits()
        }

        /// attention: if call it, should better call between disab/enable interrupt
        fn eeprom_put_char(&mut self, address: u16, data: u8) -> () {
            unsafe {
                // wait until EEPE become to zero by hardware
                self.wait_ready();

                // set EEPROM address
                self.0.eear.write(|w| w.bits(address));

                //Start EEPROM read operation
                self.0.eecr.write(|w| w.eere().set_bit());
                let old_value = self.0.eedr.read().bits();
                let diff_mask = old_value ^ data;

                // Check if any bits are changed to '1' in the new value.
                if (diff_mask & data) != 0 {
                    // Now we know that _some_ bits need to be erased to '1'.

                    // // Check if any bits in the new value are '0'.
                    if data != 0xff {
                        // Now we know that some bits need to be programmed to '0' also.
                        self.0.eedr.write(|w| w.bits(data)); // Set EEPROM data register.
                        self.0.eecr.write(|w| {
                            w.eempe().set_bit(); // Set Master Write Enable bit
                            w.eepm().val_0x00() // ...and Erase+Write mode.
                        });
                        self.0.eecr.write(|w| w.eepe().set_bit()); // Start Erase+Write operation.
                    } else {
                        // Now we know that all bits should be erased.
                        self.0.eecr.write(|w| {
                            w.eempe().set_bit(); // Set Master Write Enable bit
                            w.eepm().val_0x01() // ...and Erase-only mode..
                        });
                        self.0.eecr.write(|w| w.eepe().set_bit()); // Start Erase-only operation.
                    }
                }
                //Now we know that _no_ bits need to be erased to '1'.
                else {
                    // Check if any bits are changed from '1' in the old value.
                    if diff_mask != 0 {
                        // Now we know that _some_ bits need to the programmed to '0'.
                        self.0.eedr.write(|w| w.bits(data)); // Set EEPROM data register.
                        self.0.eecr.write(|w| {
                            w.eempe().set_bit(); // Set Master Write Enable bit
                            w.eepm().val_0x02() // ...and Write-only mode..
                        });
                        self.0.eecr.write(|w| w.eepe().set_bit()); // Start Write-only operation.
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn stack_new() {}
}
