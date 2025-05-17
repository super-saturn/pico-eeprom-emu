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

Connect pins 0-14 inclusive to your address bus, pins 15-22 to your data bus, and pin 26 to a CS / Bus Select signal (in many systems, this can simply be the highest bit of your address bus.)

You must install support for the `thumbv6m-none-eabi` target (see the RP2040 HAL project for more details.)
You must also install `flip-link` and `elf2uf2-rs`.
For a slightly more robust debug experience, use a pico probe and `probe-rs`.

## Authors

Contributors names and contact info

JP Stringham
[@jotapeh](https://mastodon.gamedev.place/@jotapeh)

## Version History

* 0.8.1
    * Revert to using `elf2uf2-rs` as the default runner
    * Revert to starting `ADDR_BUS_START_PIN` at 0 and a 15-bit address; previous limitation was based on a misunderstanding
    * Slight cleanup to `logging` feature

* 0.8
    * Updated several dependencies including `embedded-hal` and `cortex-m-rt`
    * Updated to use `probe-rs` rather than `elf2uf2-rs` by default (Change `.cargo/config.toml` if you want to revert)
    * Updated to use `flip-link` linker
    * Defaults to clock to 200MHz as it was announced that all existing Pico boards should support this out of the box
    * Fixes correctly supporting `ADDR_BUS_START_PIN`
    * Provides `logging` feature which reports intermittently when running via `probe-rs`

* 0.7
    * Initial Public Release

## License

This project is licensed under the MIT License - see the LICENSE file for details
