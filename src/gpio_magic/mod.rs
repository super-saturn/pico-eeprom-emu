// This class is effectively an adaptation of a portion of the RP2040 C SDK.
// Needed because the RP2040 HAL does not support simultaneous Pin writes/reads.
//
// RP2040 C/C++ SDK Copyright 2020 (c) 2020 Raspberry Pi (Trading) Ltd.,
// Ported portions licensed under BSD-3. (See RP2040SDK-BSD3-LICENSE)
//
// Other content Copyright (c) 2023 JP Stringham (see MIT-LICENSE)

use pac::{SIO, PADS_BANK0, IO_BANK0};
use rp2040_pac as pac;

pub struct GPIOMagic {
    sio:SIO,
    pads_bank0:PADS_BANK0,
    io_bank0:IO_BANK0
}

impl GPIOMagic {

    pub fn new(sio:SIO, pads:PADS_BANK0, iob:IO_BANK0) -> Self {
        GPIOMagic { 
            sio: sio,
            pads_bank0: pads,
            io_bank0: iob
         }
    }

    #[inline]
    pub fn gpio_clr(&mut self, pin:u32,) {
        let mask:u32 = 1<<pin;
        self.sio.gpio_out_clr.write(|w| unsafe { w.bits(mask) });
    }

    #[inline]
    pub fn gpio_set(&mut self, pin:u32) {
        let mask:u32 = 1<<pin;
        self.sio.gpio_out_set.write(|w| unsafe { w.bits(mask) });
    }

    pub fn gpio_init(&mut self, pin:u32) {
        
        self.pads_bank0.gpio[pin as usize].write(|w| {
            w
            .pue().clear_bit()
            .pde().clear_bit()
            .od().clear_bit()
            .ie().set_bit()
            .slewfast().set_bit()
        });

        self.io_bank0.gpio[pin as usize].gpio_ctrl.write(|w| {
            w
            .funcsel().sio()
        });
        self.sio.gpio_oe_clr.write(|w| unsafe { w.bits(1<<pin)});
        self.sio.gpio_out_clr.write(|w| unsafe { w.bits(1<<pin)});
    }

    pub fn gpio_set_pio0func(&mut self, pin:u32) {
        self.io_bank0.gpio[pin as usize].gpio_ctrl.write(|w| unsafe {
            w
            .bits(0)
            .funcsel().pio0()
        });
    }

    pub fn gpio_put_masked(&mut self, mask:u32, value:u32) {
        let result = self.sio.gpio_out.read().gpio_out().bits();

        self.sio.gpio_out_xor.write(|w| unsafe { w.bits((result ^ value) & mask)});
    }

    pub fn gpio_set_dir_out_masked(&mut self, mask:u32) {
        self.sio.gpio_oe_set.write(|w| unsafe { w.bits(mask)});
    }

    pub fn gpio_set_out_disabled_masked(&mut self, mask:u32) {
        self.sio.gpio_oe_clr.write(|w| unsafe { w.bits(mask) });
    }

    pub fn gpio_get(&mut self, pin:u32) -> bool {
        self.sio.gpio_in.read().bits() & (1 << pin) > 0
    }

    pub fn gpio_get_masked(&mut self, mask:u32) -> u32 {
        self.sio.gpio_in.read().bits() & mask
    }

    pub fn gpio_enable_pulldown(&mut self, pin:u32) {
        self.pads_bank0.gpio[pin as usize].modify(|_, w| {
            w.pde().set_bit()
        });
    }

}