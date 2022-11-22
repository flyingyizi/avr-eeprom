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
            avr_device::interrupt::disable();
            for i in from..to {
                self.wait_ready();

                // set EEPROM address
                unsafe {
                    self.0.eear.write(|w| w.bits(i as u16));
                }

                // Now we know that all bits should be erased.
                self.0.eecr.write(|w| {
                    w.eempe().set_bit(); // Set Master Write Enable bit
                    w.eepm().val_0x01() // ...and Erase-only mode..
                });
                self.0.eecr.write(|w| w.eepe().set_bit()); // Start Erase-only operation.
            }
            unsafe {
                avr_device::interrupt::enable();
            }

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

//[STM32读写内部Flash（介绍+附代码）](https://blog.csdn.net/qq_36075612/article/details/124087574)

//##  STM32 flash 说明
//
// stm32 halY已经实现了embedded_storage::nor_flash，它的读写特别是flash起始地址是特定的，下面记录下。
//
//stm32的flash地址起始于0x0800 0000，结束地址是0x0800 0000加上芯片实际的flash大小，不同的芯片flash大小不同。
//
//RAM起始地址是0x2000 0000，结束地址是0x2000 0000加上芯片的RAM大小。不同的芯片RAM也不同。
//
//Flash中的内容一般用来存储代码和一些定义为const的数据，断电不丢失，
//
//一般情况下，程序文件是从 0x0800 0000 地址写入（注），这个是STM32开始执行的地方，0x0800 0004是STM32的中断向量表的起始地址。
//
//
//- 注：这个写入地址是可以修改的，在avr-gcc是通过配置memory.x link script 上午FLASH MEMORY来配置，在使用keil进行编写程序时，其编程地址的设置是通过GUI设置IROM1来配置。虽然是可以配置，但显然必须要满足“>=0x0800 0000，  ”因为这是ARM架构确定的。
//
//```text
//MEMORY
//{
//  /* NOTE 1 K = 1 KiBi = 1024 bytes */
//  /* These values correspond to the STM32F411RETx */
//   RAM (xrw)      : ORIGIN = 0x20000000, LENGTH = 128K
//   FLASH (rx)      : ORIGIN = 0x8000000, LENGTH = 512K
//}
//```
//所以我们在编写flash的读写函数时，输入的读写地址必须是 大于等于“text + data”的部分。换句话说，在使用内部 FLASH 存储其它数据前需要了解哪一些空间已经写入了程序代码，存储了程序代码的扇区都不应作任何修改。通过查询应用程序编译时产生的“ *.map”后缀文件，或objdump来查看工程内存分布  以下面的
//
//```text
//stm32board> cargo size --release
//   text    data     bss     dec     hex filename
//   2916       0    1052    3968     f80 stm32board
//
//stm32board> cargo objdump --release -- -x
//stm32board:     file format elf32-littlearm
//architecture: arm
//start address: 0x08000199
//
//Program Header:
//    LOAD off    0x00000114 vaddr 0x08000000 paddr 0x08000000 align 2**2
//         filesz 0x00000198 memsz 0x00000198 flags r--
//    LOAD off    0x000002ac vaddr 0x08000198 paddr 0x08000198 align 2**2
//         filesz 0x000008b4 memsz 0x000008b4 flags r-x
//    LOAD off    0x00000b60 vaddr 0x08000a4c paddr 0x08000a4c align 2**2
//         filesz 0x00000118 memsz 0x00000118 flags r--
//    LOAD off    0x00000c80 vaddr 0x20000000 paddr 0x20000000 align 2**2
//         filesz 0x00000000 memsz 0x0000041c flags rw-
//   STACK off    0x00000000 vaddr 0x00000000 paddr 0x00000000 align 2**64
//         filesz 0x00000000 memsz 0x00000000 flags rw-
//
//Dynamic Section:
//
//Sections:
//Idx Name            Size     VMA      Type
//  0                 00000000 00000000
//  1 .vector_table   00000198 08000000 DATA
//  2 .text           000008b4 08000198 TEXT
//  3 .rodata         00000118 08000a4c DATA
//  4 .data           00000000 20000000 DATA
//  5 .gnu.sgstubs    00000000 08000b80 DATA
//  6 .bss            0000041c 20000000 BSS
//  7 .uninit         00000000 2000041c BSS
//  8 .debug_loc      00001764 00000000 DEBUG
//  9 .debug_abbrev   0000087e 00000000 DEBUG
// 10 .debug_info     0000e240 00000000 DEBUG
// 11 .debug_aranges  00000338 00000000 DEBUG
// 12 .debug_ranges   00001220 00000000 DEBUG
// 13 .debug_str      00015008 00000000 DEBUG
// 14 .debug_pubnames 00006edb 00000000 DEBUG
// 15 .debug_pubtypes 00007ba3 00000000 DEBUG
// 16 .ARM.attributes 00000038 00000000
// 17 .debug_frame    000007c4 00000000 DEBUG
// 18 .debug_line     0000278b 00000000 DEBUG
// 19 .debug_line_str 00000022 00000000 DEBUG
// 20 .debug_rnglists 00000019 00000000 DEBUG
// 21 .comment        00000013 00000000
// 22 .symtab         00000b50 00000000
// 23 .shstrtab       00000116 00000000
// 24 .strtab         00000cad 00000000
//
//SYMBOL TABLE:
//00000000 l    df *ABS*  00000000 stm32board.9d87a564-cgu.0
//080001f8 l     F .text  0000000c stm32f4xx_hal::bb::write::h8c081ffbc6d69b05
//080001f8 l       .text  00000000 $t.0
//08000204 l     F .text  0000008e alloc::raw_vec::RawVec<T,A>::reserve_for_push::h73d38b3bf9ab612e
//08000204 l       .text  00000000 $t.1
//080007c6 l     F .text  00000100 alloc::raw_vec::finish_grow::h1e50f34ff562033d
//080004c6 l     F .text  0000000a alloc::raw_vec::capacity_overflow::h82d448957da1f87d
//080004d0 l     F .text  0000000a alloc::alloc::handle_alloc_error::h26ba0b7db5195589
//08000292 l       .text  00000000 $t.2
//0800029c l     F .text  00000224 stm32board::__cortex_m_rt_main::h402ba0757f0d46b2
//0800029c l       .text  00000000 $t.3
//20000000 l     O .bss   00000018 stm32board::ALLOCATOR::h6b2aef7423b464b6
//20000018 l     O .bss   00000400 stm32board::__cortex_m_rt_main::HEAP::h8385a62aa6fc6664
//080004fa l     F .text  0000016a <alloc_cortex_m::CortexMHeap as core::alloc::global::GlobalAlloc>::alloc::h94064c6633c09da7
//08000a5c l     O .rodata        00000010 .Lanon.0515b32cc6957b17c4f18fe4cb15d04c.1
//0800068a l     F .text  0000000a core::result::unwrap_failed::h894631a26b811a7e
//080004c0 l     F .text  00000006 __rust_alloc_error_handler
//080004c0 l       .text  00000000 $t.4
//080004ee l     F .text  0000000a __rdl_oom
//080004c6 l       .text  00000000 $t.5
//08000664 l     F .text  0000000a core::panicking::panic_fmt::h003e14b1ee10c7ae
//080004d0 l       .text  00000000 $t.6
//080004da l     F .text  0000000a core::ops::function::FnOnce::call_once::h32a3e73bbc0813d2
//080004da l       .text  00000000 $t.7
//080004e4 l     F .text  0000000a alloc::alloc::handle_alloc_error::rt_error::h8623ad4ac045c14c
//080004e4 l       .text  00000000 $t.8
//080004ee l       .text  00000000 $t.9
//080004f8 l     F .text  00000002 core::ptr::drop_in_place<core::cell::BorrowMutError>::h4a0ef62afb3416f2
//080004f8 l       .text  00000000 $t.10
//080004fa l       .text  00000000 $t.11
//080006aa l     F .text  00000072 linked_list_allocator::hole::HoleList::align_layout::hba4d8d9804ee3563
//0800071e l     F .text  000000a6 linked_list_allocator::hole::deallocate::h4ec77e3a83fdfdf7
//08000b1b l     O .rodata        0000001c .Lanon.9e3d958609b9f930804be982b00ad258.13
//0800066e l     F .text  0000000a core::panicking::panic::h28bd834d4ca66e2f
//08000b37 l     O .rodata        0000002b .Lanon.fbe9f4897a67ae86c66a5e3a6ec2fccc.3
//08000664 l       .text  00000000 $t.12
//080007c4 l     F .text  00000002 rust_begin_unwind
//0800066e l       .text  00000000 $t.13
//08000678 l     F .text  00000012 <core::cell::BorrowMutError as core::fmt::Debug>::fmt::hd7a9f175581c3d57
//08000678 l       .text  00000000 $t.14
//08000a6c l     O .rodata        0000000e .Lanon.d5c21dfd4bc70099ffb5c3b394a6310e.150
//0800068a l       .text  00000000 $t.15
//08000694 l     F .text  00000012 <core::alloc::layout::LayoutError as core::fmt::Debug>::fmt::h87034e055adb6f84
//08000694 l       .text  00000000 $t.16
//08000a7a l     O .rodata        0000000b .Lanon.d5c21dfd4bc70099ffb5c3b394a6310e.681
//08000a4a l       .text  00000000 $t.17
//080006a6 l       .text  00000000 $t.18
//080006a8 l       .text  00000000 $t.19
//080006aa l       .text  00000000 $t.20
//08000a85 l     O .rodata        0000002b .Lanon.9e3d958609b9f930804be982b00ad258.1
//08000ab0 l     O .rodata        00000010 .Lanon.9e3d958609b9f930804be982b00ad258.2
//0800071c l     F .text  00000002 core::ptr::drop_in_place<core::alloc::layout::LayoutError>::ha93a775a673c7765
//0800071c l       .text  00000000 $t.21
//0800071e l       .text  00000000 $t.22
//08000ac0 l     O .rodata        0000002e .Lanon.9e3d958609b9f930804be982b00ad258.8
//08000aee l     O .rodata        0000002d .Lanon.9e3d958609b9f930804be982b00ad258.10
//080007c4 l       .text  00000000 $t.23
//080007c6 l       .text  00000000 $t.24
//08000a36 l       .text  00000000 $t
//08000198 l       .text  00000000 $t
//080001de l       .text  00000000 $d
//080001e0 l       .text  00000000 $d
//00000000 l    df *ABS*  00000000 lib.f2c86790-cgu.0
//080008c6 l       .text  00000000 $t.3
//080008ca l       .text  00000000 $t.4
//080008ce l       .text  00000000 $t.5
//080008da l       .text  00000000 $t.7
//080008e0 l       .text  00000000 $t.11
//080008e4 l       .text  00000000 $t.12
//00000000 l    df *ABS*  00000000 compiler_builtins.bad99fe4-cgu.165
//080008ea l       .text  00000000 $t.0
//080008ea l     F .text  00000004 .hidden __aeabi_memcpy
//00000000 l    df *ABS*  00000000 compiler_builtins.bad99fe4-cgu.155
//080008ee l       .text  00000000 $t.0
//080008ee l     F .text  00000004 .hidden compiler_builtins::arm::__aeabi_memcpy::hc6530cb97c283429
//00000000 l    df *ABS*  00000000 compiler_builtins.bad99fe4-cgu.99
//080008f2 l       .text  00000000 $t.0
//080008f2 l     F .text  00000144 .hidden compiler_builtins::mem::memcpy::h6c7f4436352a7ed0
//08000004 g     O .vector_table  00000004 __RESET_VECTOR
//08000198 g     F .text  00000046 Reset
//08000008 g     O .vector_table  00000038 __EXCEPTIONS
//080006a6 g     F .text  00000000 DefaultHandler
//08000a36 g     F .text  00000014 HardFaultTrampoline
//08000040 g     O .vector_table  00000158 __INTERRUPTS
//08000292 g     F .text  0000000a main
//20000418 g     O .bss   00000001 DEVICE_PERIPHERALS
//080006a6 g     F .text  00000002 DefaultHandler_
//080006a8 g     F .text  00000002 DefaultPreInit
//08000a4a g     F .text  00000002 HardFault_
//080008e4 g     F .text  00000006 __primask_r
//080008c6 g     F .text  00000004 __cpsid
//080008ca g     F .text  00000004 __cpsie
//080008ce g     F .text  0000000c __delay
//080008da g     F .text  00000006 __dsb
//080008e0 g     F .text  00000004 __nop
//080006a6 g     F .text  00000000 NonMaskableInt
//080006a6 g     F .text  00000000 MemoryManagement
//080006a6 g     F .text  00000000 BusFault
//080006a6 g     F .text  00000000 UsageFault
//080006a6 g     F .text  00000000 SVCall
//080006a6 g     F .text  00000000 DebugMonitor
//080006a6 g     F .text  00000000 PendSV
//080006a6 g     F .text  00000000 SysTick
//080006a6 g     F .text  00000000 WWDG
//080006a6 g     F .text  00000000 PVD
//080006a6 g     F .text  00000000 TAMP_STAMP
//080006a6 g     F .text  00000000 RTC_WKUP
//080006a6 g     F .text  00000000 FLASH
//080006a6 g     F .text  00000000 RCC
//080006a6 g     F .text  00000000 EXTI0
//080006a6 g     F .text  00000000 EXTI1
//080006a6 g     F .text  00000000 EXTI2
//080006a6 g     F .text  00000000 EXTI3
//080006a6 g     F .text  00000000 EXTI4
//080006a6 g     F .text  00000000 DMA1_STREAM0
//080006a6 g     F .text  00000000 DMA1_STREAM1
//080006a6 g     F .text  00000000 DMA1_STREAM2
//080006a6 g     F .text  00000000 DMA1_STREAM3
//080006a6 g     F .text  00000000 DMA1_STREAM4
//080006a6 g     F .text  00000000 DMA1_STREAM5
//080006a6 g     F .text  00000000 DMA1_STREAM6
//080006a6 g     F .text  00000000 ADC
//080006a6 g     F .text  00000000 EXTI9_5
//080006a6 g     F .text  00000000 TIM1_BRK_TIM9
//080006a6 g     F .text  00000000 TIM1_UP_TIM10
//080006a6 g     F .text  00000000 TIM1_TRG_COM_TIM11
//080006a6 g     F .text  00000000 TIM1_CC
//080006a6 g     F .text  00000000 TIM2
//080006a6 g     F .text  00000000 TIM3
//080006a6 g     F .text  00000000 TIM4
//080006a6 g     F .text  00000000 I2C1_EV
//080006a6 g     F .text  00000000 I2C1_ER
//080006a6 g     F .text  00000000 I2C2_EV
//080006a6 g     F .text  00000000 I2C2_ER
//080006a6 g     F .text  00000000 SPI1
//080006a6 g     F .text  00000000 SPI2
//080006a6 g     F .text  00000000 USART1
//080006a6 g     F .text  00000000 USART2
//080006a6 g     F .text  00000000 EXTI15_10
//080006a6 g     F .text  00000000 RTC_ALARM
//080006a6 g     F .text  00000000 OTG_FS_WKUP
//080006a6 g     F .text  00000000 DMA1_STREAM7
//080006a6 g     F .text  00000000 SDIO
//080006a6 g     F .text  00000000 TIM5
//080006a6 g     F .text  00000000 SPI3
//080006a6 g     F .text  00000000 DMA2_STREAM0
//080006a6 g     F .text  00000000 DMA2_STREAM1
//080006a6 g     F .text  00000000 DMA2_STREAM2
//080006a6 g     F .text  00000000 DMA2_STREAM3
//080006a6 g     F .text  00000000 DMA2_STREAM4
//080006a6 g     F .text  00000000 OTG_FS
//080006a6 g     F .text  00000000 DMA2_STREAM5
//080006a6 g     F .text  00000000 DMA2_STREAM6
//080006a6 g     F .text  00000000 DMA2_STREAM7
//080006a6 g     F .text  00000000 USART6
//080006a6 g     F .text  00000000 I2C3_EV
//080006a6 g     F .text  00000000 I2C3_ER
//080006a6 g     F .text  00000000 FPU
//080006a6 g     F .text  00000000 SPI4
//080006a6 g     F .text  00000000 SPI5
//08000a4a g     F .text  00000000 HardFault
//080006a8 g     F .text  00000000 __pre_init
//20000000 g       .bss   00000000 __sbss
//2000041c g       .bss   00000000 __ebss
//20000000 g       .data  00000000 __sdata
//20000000 g       .data  00000000 __edata
//08000b64 g       *ABS*  00000000 __sidata
//20020000 g       *ABS*  00000000 _stack_start
//08000198 g       .vector_table  00000000 _stext
//2000041c g       .uninit        00000000 __euninit
//2000041c g       .uninit        00000000 __sheap
//08000008 g       .vector_table  00000000 __reset_vector
//08000040 g       .vector_table  00000000 __eexceptions
//08000198 g       .text  00000000 __stext
//08000a4c g       .text  00000000 __etext
//08000a4c g       .rodata        00000000 __srodata
//08000b64 g       .rodata        00000000 __erodata
//08000b80 g       .gnu.sgstubs   00000000 __veneer_base
//08000b80 g       .gnu.sgstubs   00000000 __veneer_limit
//2000041c g       .uninit        00000000 __suninit
//```

#[cfg(test)]
mod tests {

    #[test]
    fn stack_new() {}
}
