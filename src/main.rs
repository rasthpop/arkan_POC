#![no_std]
#![no_main]

use embedded_hal::serial::Read;
mod encryption;
// use crate::encryption::{CoordinateEncryptor, EncryptConfig, GpsCoord, MyCipher};

use embedded_hal::digital::v2::OutputPin;
use panic_halt as _;
mod gps_proccess;
use rp_pico::entry;
use rp_pico::hal::fugit::HertzU32;
use rp_pico::hal::{
    spi::Spi,
    clocks::{init_clocks_and_plls, Clock},
    pac,
    uart::{DataBits, StopBits, UartConfig, UartPeripheral},
    watchdog::Watchdog,
    Sio
};

mod sleep;
use sleep::{disable_uart0, enable_uart0, sleep_ms};

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
    let mut timer = rp_pico::hal::timer::Timer::new(pac.TIMER, &mut pac.RESETS, &clocks);

    let sio = Sio::new(pac.SIO);
    let core = pac::CorePeripherals::take().unwrap();
    let delay = cortex_m::delay::Delay::new(core.SYST, clocks.system_clock.freq().to_Hz());
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
    let spi_sck = pins.gpio18.into_function::<rp_pico::hal::gpio::FunctionSpi>();
    let spi_mosi = pins.gpio19.into_function::<rp_pico::hal::gpio::FunctionSpi>();
    let spi_miso = pins.gpio16.into_function::<rp_pico::hal::gpio::FunctionSpi>();
    let spi= Spi::new(
        pac.SPI0,
        (
            spi_mosi,
            spi_miso,
            spi_sck
        ),
    );
    let spi0:Spi<_,_,_,8> = spi.init(
        &mut pac.RESETS,
        clocks.peripheral_clock.freq(),
        HertzU32::Hz(8_000_000),
        embedded_hal::spi::MODE_0,
    );

    let mut nss = pins.gpio17.into_push_pull_output();
    nss.set_high().unwrap();
    let mut rst = pins.gpio20.into_push_pull_output();
    rst.set_high().unwrap();
    let mut lora = sx127x_lora::LoRa::new(
        spi0,
        nss,
        rst,
        433,
        delay
    ).expect("Could not connect to LoRa");
    let _ = lora.set_tx_power(17, 1);

    let mut serial = SerialPort::new(&usb_bus);
    let mut usb_dev = UsbDeviceBuilder::new(&usb_bus, UsbVidPid(0x16c0, 0x27dd))
        .device_class(2)
        .build();

    let mut led_pin = pins.led.into_push_pull_output();
    let uart_pins = (
        pins.gpio8.into_function::<rp_pico::hal::gpio::FunctionUart>(),//tx
        pins.gpio9.into_function::<rp_pico::hal::gpio::FunctionUart>()//rx
    );
    let mut uart = UartPeripheral::new(
        pac.UART1, 
        uart_pins, 
        &mut pac.RESETS)
        .enable(
            UartConfig::new(HertzU32::Hz(9600), DataBits::Eight, None, StopBits::One),
            clocks.peripheral_clock.freq()
        )
        .unwrap();
    let mut buf = [0u8; 128];
    let mut i = 0;
    let mut lora_buf = [0u8; 255];
    let mut last_success_time: u64 = timer.get_counter().ticks();
    loop {
        if usb_dev.poll(&mut [&mut serial]) {
            // todo
        }
        
        if let Ok(b) = uart.read() {
            if b == b'\n' {
                let line = &buf[..i];
                if let Some(len) = gps_proccess::gps_proccess(line, &mut serial,&mut lora_buf) {
                    last_success_time = timer.get_counter().ticks();
                    match lora.transmit_payload(lora_buf, len) {
                        Ok(_) => {
                            let _ = serial.write(b"sent data to LoRa\r\n");
                        },
                        Err(_) => { 
                            let _ = serial.write(b"ERR\r\n");
                        }
                    }
                }
                
                i = 0;
            } else if b == b'\r' {
                // ignore CR from CRLF
            } else if i < buf.len() {
                buf[i] = b;
                i += 1;
            } else {
                // buffer full: reset and store current byte as first byte
                i = 0;
                if !buf.is_empty() {
                    buf[0] = b;
                    i = 1;
                }
            }
            let now = timer.get_counter().ticks();
            if now.wrapping_sub(last_success_time) > 30_000_000 {
                let _ = serial.write(b"No valid GPS data for 30 sec, going to sleep...\r\n");
                disable_uart0();
                let _ = led_pin.set_low();
                sleep_ms(&mut timer, 30_000); // sleep 30 s
                enable_uart0();
                let _ = led_pin.set_high();

                let _ = serial.write(b"Woke up from sleep, retrying GPS connection...\r\n");
                last_success_time = timer.get_counter().ticks();
            }
        }
    }
}
