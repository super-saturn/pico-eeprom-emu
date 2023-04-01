// Raspi Pico EEPROM Emulator
// Copyright (c) 2023 JP Stringham (see MIT-LICENSE)
//
// 8-bit output, 15-bit Addressing (32kB) + Chip Select
// when CS is low, output is in high impedance state.
//
// Typ resp time: ~50ns
//
// Hardware setup:
// Busses must be sequential and take every possible GPIO from 0-22
// see ADDR_BUS_START_PIN and DATA_BUS_START_PIN for where they start
// 
// You can swap them if wiring is made easier and/or expand/reduce
// if you have a different board with more/less GPIOs exposed.
//
// ROM Contents:
// Simply swap out "blankrom.bin" for any binary file up to 32kB in size

#![no_std]
#![no_main]

use rp2040_hal as hal;
use hal::pac as pac;

pub mod gpio_magic;

use panic_halt as _;

#[link_section = ".boot_loader"]
#[used]
pub static BOOT_LOADER: [u8; 256] = rp2040_boot2::BOOT_LOADER_W25Q080;

const LED_PIN:u32 = 25;
const ADDR_BUS_START_PIN:u32 = 0;
const ADDR_BUS_BITS:u32 = 15;
const DATA_BUS_START_PIN:u32 = 15;

const CS_PIN:u32 = 26;

#[rp2040_hal::entry]
fn main() -> ! {
    // Grab our singleton objects
    let mut pac = pac::Peripherals::take().unwrap();
    let core = pac::CorePeripherals::take().unwrap();

    // Set up the watchdog driver - needed by the clock setup code
    let mut watchdog = hal::Watchdog::new(pac.WATCHDOG);

    const XTAL_FREQ_HZ: u32 = 12_000_000u32;

    // Configure the clocks
    let _clocks = hal::clocks::init_clocks_and_plls(
        XTAL_FREQ_HZ,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();

    // abuse the hal package to init because there's some secret sauce in it
    // I haven't yet figured out. :(
    do_shady_init_stuff();

    let mut gpio_magician = gpio_magic::GPIOMagic::new(pac.SIO,pac.PADS_BANK0, pac.IO_BANK0);

    // The delay object lets us wait for specified amounts of time (in
    // milliseconds)
    let _delay = cortex_m::delay::Delay::new(core.SYST, 133_000_000u32);

    gpio_magician.gpio_init(LED_PIN);

    // gpio_set_dir_out_masked(DATABUS_PINS_MASK, &mut sio);
    gpio_magician.gpio_set_dir_out_masked(1<<LED_PIN);
    gpio_magician.gpio_set(LED_PIN);

    let mut i = 0;
    while i < 12 {
        gpio_magician.gpio_init(ADDR_BUS_START_PIN+i);
        i+=1;
    }
    i = 0;
    while i < 8 {
        gpio_magician.gpio_init(DATA_BUS_START_PIN+i);
        i+=1;
    }
    
    let address_bus_pinmask = generate_bus_pinmask(ADDR_BUS_START_PIN, ADDR_BUS_BITS);
    let data_bus_pinmask = generate_bus_pinmask(DATA_BUS_START_PIN, 16); // 8bit output (1byte)
    gpio_magician.gpio_set_dir_out_masked(data_bus_pinmask);

    let mut memory:[u8; 1 << ADDR_BUS_BITS] = [0; 1 << ADDR_BUS_BITS];
    let memory_file = *include_bytes!("../user_ROMS/6502_functional_test_truncated.bin");

    memory[..memory_file.len()].copy_from_slice(&memory_file[..]);

    const CS_PIN_MASK:u32 = 1 << CS_PIN;
    let data_and_led_pinmask = data_bus_pinmask | (1 << LED_PIN); 
    
    loop {
        let cs = gpio_magician.gpio_get_masked(CS_PIN_MASK);

        if cs > 0 { // chip select for ROM
            gpio_magician.gpio_set_dir_out_masked(data_and_led_pinmask);
        } else {
            gpio_magician.gpio_set_out_disabled_masked(data_and_led_pinmask);
        }

        let addr_in = gpio_magician.gpio_get_masked(address_bus_pinmask);
        let mem_fetch = (memory[addr_in as usize] as u32) << DATA_BUS_START_PIN;

        gpio_magician.gpio_put_masked(data_bus_pinmask, mem_fetch);
    }
}

// There's some magic initialization sauce in the rp2040 hal Pins::new()
// which I haven't figured out yet.
// Would rather not use it this way - not safe, obviously.
fn do_shady_init_stuff() {
    unsafe {
        let mut pac = pac::Peripherals::steal();
        let sio = hal::Sio::new(pac.SIO);

        let _pins = hal::gpio::Pins::new(
            pac.IO_BANK0,
            pac.PADS_BANK0,
            sio.gpio_bank0,
            &mut pac.RESETS
        );

        pac.CLOCKS.clk_sys_div.write(|w| {
            w.int().bits(1)
        });
    }
}

// Generate a mask that we can use to simultaneously init/output to multiple pins.
fn generate_bus_pinmask(start_pin:u32, num_pins:u32) -> u32 {
    let mut pin_mask:u32 = 0;
    let mut i = 0;
    while i < num_pins {
        pin_mask |= 1 << (i+start_pin);
        i += 1;
    }
    pin_mask
}