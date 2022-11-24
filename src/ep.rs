//! macro to define eeprom wrapper that support embedded_storage::nor_flash

/// wrapper eeprom,
///
/// notes, eeprom capacity: atmega328p/atmega328pb( 1k), atmega2560(4k),atmega32u4(1k),atmega168(512), atmega1284p(4k),atmega1280(4k),atmega48p(256)
///
/// # example
/// ```no_run
/// use avr_eeprom::prelude::*;
/// avr_eeprom::impl_eeprom_traditional! {Eeprom,avr_device::atmega328p::EEPROM,1024}
/// let mut ep = Eeprom {};
/// let capacity = ep.capacity();
/// let mut data = [0_u8; 256];
/// ep.read(0, &mut data);
/// ```
/// field1: Name, field2: eeprom pac. e.g. arduino_hal::hal::pac::EEPROM; field: capacity size
#[macro_export]
macro_rules! impl_eeprom_traditional {
    ($Name:ident, $ep:ty, $capacity:literal) => {
        pub struct $Name {}
        impl $Name {
            #[inline]
            unsafe fn wait_ready(&self) {
                //Wait for completion of previous write.
                while (*<$ep>::ptr()).eecr.read().eepe().bit_is_set() {}
            }
            unsafe fn eeprom_get_char(&mut self, address: u16) -> u8 {
                self.wait_ready();
                // set EEPROM address register
                (*<$ep>::ptr()).eear.write(|w| w.bits(address));
                //Start EEPROM read operation
                (*<$ep>::ptr()).eecr.write(|w| w.eere().set_bit());

                // Return the byte read from EEPROM
                (*<$ep>::ptr()).eedr.read().bits()
            }
            /// attention: if call it, should better call between disab/enable interrupt
            unsafe fn eeprom_put_char(&mut self, address: u16, data: u8) -> () {
                // wait until EEPE become to zero by hardware
                self.wait_ready();

                // set EEPROM address
                (*<$ep>::ptr()).eear.write(|w| w.bits(address));

                //Start EEPROM read operation
                (*<$ep>::ptr()).eecr.write(|w| w.eere().set_bit());
                let old_value = (*<$ep>::ptr()).eedr.read().bits();
                let diff_mask = old_value ^ data;

                // Check if any bits are changed to '1' in the new value.
                if (diff_mask & data) != 0 {
                    // Now we know that _some_ bits need to be erased to '1'.

                    // Check if any bits in the new value are '0'.
                    if data != 0xff {
                        // Now we know that some bits need to be programmed to '0' also.
                        (*<$ep>::ptr()).eedr.write(|w| w.bits(data)); // Set EEPROM data register.
                        (*<$ep>::ptr()).eecr.write(|w| {
                            w.eempe().set_bit(); // Set Master Write Enable bit
                            w.eepm().val_0x00() // ...and Erase+Write mode.
                        });
                        (*<$ep>::ptr()).eecr.write(|w| w.eepe().set_bit()); // Start Erase+Write operation.
                    } else {
                        // Now we know that all bits should be erased.
                        (*<$ep>::ptr()).eecr.write(|w| {
                            w.eempe().set_bit(); // Set Master Write Enable bit
                            w.eepm().val_0x01() // ...and Erase-only mode..
                        });
                        (*<$ep>::ptr()).eecr.write(|w| w.eepe().set_bit()); // Start Erase-only operation.
                    }
                }
                //Now we know that _no_ bits need to be erased to '1'.
                else {
                    // Check if any bits are changed from '1' in the old value.
                    if diff_mask != 0 {
                        // Now we know that _some_ bits need to the programmed to '0'.
                        (*<$ep>::ptr()).eedr.write(|w| w.bits(data)); // Set EEPROM data register.
                        (*<$ep>::ptr()).eecr.write(|w| {
                            w.eempe().set_bit(); // Set Master Write Enable bit
                            w.eepm().val_0x02() // ...and Write-only mode..
                        });
                        (*<$ep>::ptr()).eecr.write(|w| w.eepe().set_bit()); // Start Write-only operation.
                    }
                }
            }
        }

        impl $crate::embedded_storage::nor_flash::ReadNorFlash for $Name {
            type Error = $crate::CustomError;
            const READ_SIZE: usize = 1;

            fn read(&mut self, offset: u32, bytes: &mut [u8]) -> Result<(), Self::Error> {
                if bytes.len() + offset as usize > $capacity {
                    return Err($crate::CustomError::Bounds);
                }
                let len = bytes.len();
                let mut offset = offset as u16;
                for i in 0..len {
                    bytes[i] = unsafe { self.eeprom_get_char(offset) };
                    offset += 1;
                }
                Ok(())
            }
            fn capacity(&self) -> usize {
                $capacity
            }
        }

        impl $crate::embedded_storage::nor_flash::NorFlash for $Name {
            const WRITE_SIZE: usize = 1;
            const ERASE_SIZE: usize = 1;
            fn erase(&mut self, from: u32, to: u32) -> Result<(), Self::Error> {
                if to > $capacity {
                    return Err($crate::CustomError::Bounds);
                }

                $crate::avr_device::interrupt::free(|_cs| {
                    unsafe {
                        for i in from..to {
                            self.wait_ready();

                            // set EEPROM address
                            (*<$ep>::ptr()).eear.write(|w| w.bits(i as u16));

                            // Now we know that all bits should be erased.
                            (*<$ep>::ptr()).eecr.write(|w| {
                                w.eempe().set_bit(); // Set Master Write Enable bit
                                w.eepm().val_0x01() // ...and Erase-only mode..
                            });
                            (*<$ep>::ptr()).eecr.write(|w| w.eepe().set_bit()); // Start Erase-only operation.
                        }
                    }
                });

                Ok(())
            }

            fn write(&mut self, offset: u32, bytes: &[u8]) -> Result<(), Self::Error> {
                let mut offset = offset as u16;
                for i in bytes {
                    $crate::avr_device::interrupt::free(|_cs| {
                        unsafe { self.eeprom_put_char(offset as u16, *i) };
                        offset += 1;
                    });
                }
                Ok(())
            }
        }
        // AVR supports multiple writes
        impl $crate::embedded_storage::nor_flash::MultiwriteNorFlash for $Name {}
    }; // () => {};
}
