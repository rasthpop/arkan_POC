#![no_std]
#![no_main]

use embedded_hal::serial::Read;
use panic_halt as _;
mod gps_proccess;
use rp_pico::entry;
use rp_pico::hal::fugit::HertzU32;
use rp_pico::hal::{
    clocks::{init_clocks_and_plls, Clock},
    pac,
    uart::{DataBits, StopBits, UartConfig, UartPeripheral},
    watchdog::Watchdog,
    Sio
};

use usb_device::class_prelude::UsbBusAllocator;
use usbd_serial::SerialPort;
use usb_device::prelude::UsbVidPid;
use usb_device::prelude::UsbDeviceBuilder;
#[entry]
fn main() -> ! {
    let mut pac = pac::Peripherals::take().unwrap();
    let mut watchdog = Watchdog::new(pac.WATCHDOG);
    let clocks = init_clocks_and_plls(
        rp_pico::XOSC_CRYSTAL_FREQ,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();

    let sio = Sio::new(pac.SIO);
    let core = pac::CorePeripherals::take().unwrap();
    let mut delay = cortex_m::delay::Delay::new(core.SYST, clocks.system_clock.freq().to_Hz());
    let pins = rp_pico::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    let usb_bus = UsbBusAllocator::new(rp_pico::hal::usb::UsbBus::new(
        pac.USBCTRL_REGS,
        pac.USBCTRL_DPRAM,
        clocks.usb_clock,
        true,
        &mut pac.RESETS,
    ));

    let mut serial = SerialPort::new(&usb_bus);
    let mut usb_dev = UsbDeviceBuilder::new(&usb_bus, UsbVidPid(0x16c0, 0x27dd))
        .device_class(2)
        .build();

    let mut led_pin = pins.led.into_push_pull_output();
    let uart_pins = (
        pins.gpio0.into_function::<rp_pico::hal::gpio::FunctionUart>(),//tx
        pins.gpio1.into_function::<rp_pico::hal::gpio::FunctionUart>()//rx
    );
    let mut uart = UartPeripheral::new(
        pac.UART0, 
        uart_pins, 
        &mut pac.RESETS)
        .enable(
            UartConfig::new(HertzU32::Hz(9600), DataBits::Eight, None, StopBits::One),
            clocks.peripheral_clock.freq()
        )
        .unwrap();
    let mut buf = [0u8; 128];
    let mut i = 0;
    loop {
        if usb_dev.poll(&mut [&mut serial]) {
            // todo
        }
        
        if let Ok(b) = uart.read() {
            if b == b'\n' {
                let line = &buf[..i];
                gps_proccess::gps_proccess(line, &mut serial);
                i=0;
            } else if i < buf.len() {
                buf[i] = b;
                i+=1;
            } else {
                i=0;
            }
        }
    }
}
