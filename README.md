# EEPROM Simulator for RP2040 / Raspi Pico

An Embedded Rust based solution to turn a Raspi Pico into an "EEPROM"-like device.

## Description

15-bit Addressable (32kB) input, 8-bit (1byte) output per address. 1 bit Chip Select input.

Takes any binary file as the ROM contents.

This was developed as part of a larger project to emulate a 6502 on a Raspi Pico. I don't own either a 6502 or a convenient EEPROM. Here they are working together, (mostly) passing the Klaus25m 6502 Functional Test Suite.
![pico-eeprom-emu](https://user-images.githubusercontent.com/127321359/226114714-88d45b2d-086f-4fc7-95b3-566d89027b5a.jpeg)

In the course of development of that project, this EEPROM software/hardware solution has undergone many hundreds of millions of access cycles and proven to be very robust.

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
