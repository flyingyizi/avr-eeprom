

# example notes

## test data notes

in this example, we will read font glyphs data from atmega328p eeprom , then show in ssd1306 LCD. the LCD is drivered by embedded-graphic.

the embedded glyphs (located in generated dir) is generated through wenquanyi_9pt.bdf (located in testdata dir)  by convert-bdf tool.  this tool is in http://github.com/flyingyizi/embedded-fonts.



## proteus simulate notes

- eeprom init data only support .bin (binary )format, not support .eep (ihex) format. 
- Before running a simulation with internal eeprom data (in serial eeprom, uc eeprom,...) isis need
    to "reset the internal data" with the command DEBUG>RESET PERSISTENT MODEL DATA.

- connections:

   - `A4`: I2C SDA signal
   - `A5`: I2C SCL signal

## run

for ease of use, provide "ssd1306eeprom.toml" 

- generate hex file run in proteus

    ```shell
    $ cargo make --makefile ./ssd1306eeprom.toml hex
    $ ls 
    ssd1306_eeprom.hex   ...
    ```

- generate eeprom init data file run in proteus

    ```shell
    $ cargo make --makefile ./ssd1306eeprom.toml eepbin
    $ ls 
    ssd1306_eeprom.eep.bin   ...
    ```


- shows target memusage

    ```shell
    cargo make --makefile ./ssd1306eeprom.toml eep
    ```
    ```shell
    $ cargo make --makefile ./ssd1306eeprom.toml usage
    ...
    target/avr-atmega328p/release/ssd1306_eeprom.elf:     file format elf32-avr
    AVR Memory Usage
    ----------------
    Device: atmega328p

    Program:    9714 bytes (29.6% Full)
    (.text + .data + .bootloader)

    Data:       1061 bytes (51.8% Full)
    (.data + .bss + .noinit)

    EEPROM:      222 bytes (21.7% Full)
    (.eeprom)


    Sections:
    Idx Name          Size      VMA       LMA       File off  Algn
    0 .data         00000424  00800100  000021ce  000022a2  2**0
                    CONTENTS, ALLOC, LOAD, DATA
    ...                  
    ```

