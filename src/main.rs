// Raspi Pico EEPROM Emulator
// Copyright (c) 2023-2025 JP Stringham (see MIT-LICENSE)
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

use defmt::println;
use rp2040_hal::{self as hal, fugit::RateExtU32, pll::{common_configs::PLL_USB_48MHZ, PLLConfig}};
use hal::pac as pac;

use defmt_rtt as _;
use rp2040_panic_usb_boot as _;

pub mod gpio_magic;

#[link_section = ".boot_loader"]
#[used]
pub static BOOT_LOADER: [u8; 256] = rp2040_boot2::BOOT_LOADER_W25Q080;


// DEAR READER: USE THIS SECTION TO CONFIGURE
const LED_PIN:u32 = 25;
const ADDR_BUS_START_PIN:u32 = 2;
const ADDR_BUS_BITS:u32 = 8;
const DATA_BUS_START_PIN:u32 = 15;

// nb. your ROM file can't be bigger than this or you wouldn't be able to access all of it.
const MEM_SIZE:usize = 1 << ADDR_BUS_BITS;

// when this pin is HIGH, GPIO pins will be set to OUT.
// otherwise they allow floating and assert no control over the bus
const CS_PIN:u32 = 26;

const XTAL_FREQ_HZ: u32 = 12_000_000u32;

#[rp2040_hal::entry]
fn main() -> ! {
    // Grab our singleton objects
    let mut pac = pac::Peripherals::take().unwrap();
    let _core = pac::CorePeripherals::take().unwrap();

    // Set up the watchdog driver - needed by the clock setup code
    let mut watchdog = hal::Watchdog::new(pac.WATCHDOG);
    
    hal::vreg::set_voltage(&mut pac.VREG_AND_CHIP_RESET, pac::vreg_and_chip_reset::vreg::VSEL_A::VOLTAGE1_15);
    // settle
    cortex_m::asm::delay(3_000_000);

    pac.VREG_AND_CHIP_RESET.bod().write(|w| {
        w.vsel().variant(0b1000)
    });


    let xosc = hal::xosc::setup_xosc_blocking_custom_delay(pac.XOSC, XTAL_FREQ_HZ.Hz(),128)
        .map_err(|_x| false)
        .unwrap();

    watchdog.enable_tick_generation((XTAL_FREQ_HZ / 1_000_000) as u8);


    // Configure the clocks
    let mut clocks = hal::clocks::ClocksManager::new(pac.CLOCKS);

    let pll_usb = rp2040_hal::pll::setup_pll_blocking(
        pac.PLL_USB,
        xosc.operating_frequency(),
        PLL_USB_48MHZ,
        &mut clocks,
        &mut pac.RESETS
    ).unwrap();

    let pll_sys = hal::pll::setup_pll_blocking(
        pac.PLL_SYS,
        xosc.operating_frequency(),
        PLLConfig {
            vco_freq: 1600.MHz(),
            refdiv: 1,
            post_div1: 4,
            post_div2: 2,
        },
        &mut clocks,
        &mut pac.RESETS
    ).unwrap();

    clocks.init_default(&xosc, &pll_sys, &pll_usb).unwrap();

    do_shady_init_stuff();

    let mut gpio_magician = gpio_magic::GPIOMagic::new(pac.SIO, pac.PADS_BANK0, pac.IO_BANK0);

    // The delay object lets us wait for specified amounts of time (in
    // milliseconds)
    // let mut _delay = cortex_m::delay::Delay::new(core.SYST, clocks.system_clock.freq().to_Hz());

    gpio_magician.gpio_init(LED_PIN);

    // gpio_set_dir_out_masked(DATABUS_PINS_MASK, &mut sio);
    gpio_magician.gpio_set_dir_out_masked(1<<LED_PIN);
    gpio_magician.gpio_clr(LED_PIN);

    let mut i = 0;
    while i < ADDR_BUS_BITS {
        gpio_magician.gpio_init(ADDR_BUS_START_PIN+i);
        i+=1;
    }
    i = 0;
    while i < 8 {
        gpio_magician.gpio_init(DATA_BUS_START_PIN+i);
        i+=1;
    }
    
    let address_bus_pinmask = generate_bus_pinmask(ADDR_BUS_START_PIN, ADDR_BUS_BITS);
    let data_bus_pinmask = generate_bus_pinmask(DATA_BUS_START_PIN, 8); // 8bit output (1byte)

    assert_eq!(address_bus_pinmask & data_bus_pinmask, 0);

    gpio_magician.gpio_set_dir_out_masked(data_bus_pinmask);

    println!("addr bus pinmask: {=u32:#034b}", address_bus_pinmask);
    println!("data bus pinmask: {=u32:#034b}", data_bus_pinmask);

    // // NB. if you provide a memory_file that's bigger than 1 << ADDR_BUS_BITS you can't access all of it
    let memory:[u8; MEM_SIZE] = [0x42u8; MEM_SIZE];
    // // let memory_file = [0x42u8; MEM_SIZE];

    // // memory[..memory_file.len()].copy_from_slice(&memory_file[..]);

    const CS_PIN_MASK:u32 = 1 << CS_PIN;
    let data_and_led_pinmask = data_bus_pinmask;// | (1 << LED_PIN);

    println!("Setup complete");
    let mut ticks:u32 = 0;

    gpio_magician.gpio_set(LED_PIN);
    
    loop {
        let cs = gpio_magician.gpio_get_masked(CS_PIN_MASK);

        if cs > 0 { // chip select for ROM
            gpio_magician.gpio_set_dir_out_masked(data_and_led_pinmask);
        } else {
            gpio_magician.gpio_set_out_disabled_masked(data_and_led_pinmask);
        }

        let addr_in = gpio_magician.gpio_get_masked(address_bus_pinmask) >> ADDR_BUS_START_PIN;
        let addr_in = (addr_in as usize) % MEM_SIZE;

        let pinmask_out = (memory[addr_in] as u32) << DATA_BUS_START_PIN;

        gpio_magician.gpio_put_masked(data_bus_pinmask, pinmask_out);

        #[cfg(feature = "logging")] {
            ticks = ticks + 1;

            if ticks % 10_000_000 == 0 {
                println!("\n~~~~~~~~~\nts: {}\nAddr: {:#x}\nROM {:#x}\nPins: {=u32:#034b}", ticks, addr_in, memory[addr_in] as u32, pinmask_out);
            }
        }
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