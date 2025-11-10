#![no_std]
#![no_main]
use embedded_hal::prelude::_embedded_hal_serial_Read;
use embedded_hal::digital::v2::OutputPin;
use panic_halt as _;

use rp_pico::entry;
use rp_pico::hal::fugit::HertzU32;
use rp_pico::hal::{
    clocks::{init_clocks_and_plls, Clock},
    pac,
    uart::{DataBits, StopBits, UartConfig, UartPeripheral},
    watchdog::Watchdog,
    Sio,
};

#[entry]
fn main() -> ! {
    let mut pac = pac::Peripherals::take().unwrap();
    let core = pac::CorePeripherals::take().unwrap();
    let mut watchdog = Watchdog::new(pac.WATCHDOG);
    let clocks = init_clocks_and_plls(
        rp_pico::XOSC_CRYSTAL_FREQ,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog).ok().unwrap();
    let mut delay = cortex_m::delay::Delay::new(core.SYST, clocks.system_clock.freq().to_Hz());
    let sio = Sio::new(pac.SIO);
    let pins = rp_pico::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS);
    let mut led_pin = pins.led.into_push_pull_output();
    // gps uses gpio8 and gpio9
    let uart_pins = (
        pins.gpio8.into_function::<rp_pico::hal::gpio::FunctionUart>(),
        pins.gpio9.into_function::<rp_pico::hal::gpio::FunctionUart>(),
    );
    
    let mut uart = UartPeripheral::new(pac.UART1, uart_pins, &mut pac.RESETS)
        .enable(
            UartConfig::new(HertzU32::Hz(9600), DataBits::Eight, None, StopBits::One),
            clocks.peripheral_clock.freq(),
        )
        .unwrap();
    let mut gps_buffer: [u8;256] = [0;256];
    let mut buffer_index = 0;
    led_pin.set_high().unwrap();
    delay.delay_ms(500);
    led_pin.set_low().unwrap();
    delay.delay_ms(500);
    loop {
        led_pin.set_high().unwrap();
        delay.delay_ms(500);
        led_pin.set_low().unwrap();
        delay.delay_ms(500);
        while let Ok(byte) = uart.read() {
            gps_buffer[buffer_index] = byte;
            buffer_index = (buffer_index+1) % gps_buffer.len();
        }
    }
}
