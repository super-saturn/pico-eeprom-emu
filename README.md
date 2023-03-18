# EEPROM Simulator for RP2040 / Raspi Pico

An Embedded Rust based solution to turn a Raspi Pico into an "EEPROM"-like device.

## Description

15-bit Addressable (32kB) input, 8-bit (1byte) output per address. 1 bit Chip Select input.

Takes any binary file as the ROM contents.

This was developed as part of a larger project to emulate a 6502 on a Raspi Pico. I don't own either a 6502 or a convenient EEPROM.

## Getting Started

This should build to any RP2040 based device.

Simply swap out the "blankrom.bin" with your preferred binary, and run:

`cargo run --release`

Connect pins 0-14 to your address bus, pins 15-22 to your data bus, and pin 26 to a CS / Bus Select signal (in many systems, this can simply be the highest bit of your address bus.)

You may need to install support for the `thumbv6m-none-eabi` target (see the RP2040 HAL project for more details.)

## Authors

Contributors names and contact info

JP Stringham
[@jotapeh](https://mastodon.gamedev.place/@jotapeh)

## Version History

* 0.7
    * Initial Public Release

## License

This project is licensed under the MIT License - see the LICENSE file for details