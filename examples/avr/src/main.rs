//! Connections
//! -----------
//!  - `A4`: I2C SDA signal
//!  - `A5`: I2C SCL signal
//!

#![no_std]
#![no_main]
#![feature(abi_avr_interrupt)]

mod generated;
use embedded_fonts::BdfFont;
use embedded_fonts::BdfTextStyle;

use generated::{LINE_HEIGHT, REPLACEMENT_CHARACTER, S_DATA_LEN, S_GLYPHS};

use embedded_graphics::{pixelcolor::BinaryColor, prelude::*, text::Text};

use arduino_hal::prelude::*;
use avr_eeprom::{impl_eeprom_traditional, prelude::*};
use ssd1306::{prelude::*, I2CDisplayInterface, Ssd1306};

use panic_halt as _;

impl_eeprom_traditional! {Eeprom,arduino_hal::hal::pac::EEPROM,1024}

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);
    let mut serial = arduino_hal::default_serial!(dp, pins, 115200);
    ufmt::uwriteln!(
        &mut serial,
        "Hello from Arduino! simulate ssd1306 oled 128X32 display, glyphs data store in eeprom:\r"
    )
    .void_unwrap();

    ////////////////////////////////
    // instance ep has embedded_storage capability
    let mut ep = Eeprom {};
    ufmt::uwriteln!(&mut serial, "eeprom capacity is:{}\r", ep.capacity()).void_unwrap();
    let mut data = [0_u8; S_DATA_LEN];
    let _start_address: u16 = 0;

    if ep.read(0, &mut data).is_err() {
        ufmt::uwriteln!(&mut serial, "read eeprom fail:\r").void_unwrap();
        loop {}
    }

    let font_wenquanyi_9pt: BdfFont = BdfFont {
        glyphs: &S_GLYPHS,
        data: &data,
        line_height: LINE_HEIGHT,
        replacement_character: REPLACEMENT_CHARACTER,
    };

    ////////////////////////////////
    let i2c = arduino_hal::I2c::new(
        dp.TWI,
        pins.a4.into_pull_up_input(),
        pins.a5.into_pull_up_input(),
        50000,
    );

    let interface = I2CDisplayInterface::new(i2c);
    let mut display = Ssd1306::new(interface, DisplaySize128x32, DisplayRotation::Rotate0)
        .into_buffered_graphics_mode();
    display.init().unwrap();

    ////////////////////
    let my_style = BdfTextStyle::new(&font_wenquanyi_9pt, BinaryColor::On /*Rgb888::BLUE*/);
    Text::new("中国欢迎China welcomes", Point::new(0, 10), my_style)
        .draw(&mut display)
        .unwrap();

    Text::new(
        "日本へようこそWelcome to Japan",
        Point::new(0, 24),
        my_style,
    )
    .draw(&mut display)
    .unwrap();
    ufmt::uwriteln!(&mut serial, "draw text close:\r").void_unwrap();

    display.flush().unwrap();

    loop {}
}

// mod help {
//     use embedded_graphics::{
//         pixelcolor::BinaryColor,
//         prelude::*,
//         primitives::{Circle, PrimitiveStyleBuilder, Rectangle, Triangle},
//     };
//     use ssd1306::{mode::BufferedGraphicsMode, prelude::*, Ssd1306};

//     #[allow(dead_code)]
//     pub fn draw<DI>(
//         display: &mut Ssd1306<DI, DisplaySize128x32, BufferedGraphicsMode<DisplaySize128x32>>,
//     ) where
//         DI: WriteOnlyDataCommand,
//     {
//         display.init().unwrap();

//         let yoffset = 8;

//         let style = PrimitiveStyleBuilder::new()
//             .stroke_width(1)
//             .stroke_color(BinaryColor::On)
//             .build();

//         // screen outline
//         // default display size is 128x64 if you don't pass a _DisplaySize_
//         // enum to the _Builder_ struct
//         Rectangle::new(Point::new(0, 0), Size::new(127, 31))
//             .into_styled(style)
//             .draw(display)
//             .unwrap();

//         // // triangle
//         // Triangle::new(
//         //     Point::new(16, 16 + yoffset),
//         //     Point::new(16 + 16, 16 + yoffset),
//         //     Point::new(16 + 8, yoffset),
//         // )
//         // .into_styled(style)
//         // .draw(display)
//         // .unwrap();

//         // square
//         Rectangle::new(Point::new(52, yoffset), Size::new_equal(16))
//             .into_styled(style)
//             .draw(display)
//             .unwrap();

//         // circle
//         Circle::new(Point::new(88, yoffset), 16)
//             .into_styled(style)
//             .draw(display)
//             .unwrap();

//         display.flush().unwrap();
//     }
// }
