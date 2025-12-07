#![no_std]
#![no_main]

use core::fmt::Write;
use embedded_hal::serial::Read;
use heapless::String;
use embedded_hal::blocking::delay::DelayMs;

use embedded_hal::digital::v2::OutputPin;
use panic_halt as _;
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

mod decryption;
use decryption::{decrypt_packet, DecryptError};

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
    rst.set_low().unwrap();
    timer.delay_ms(10);
    rst.set_high().unwrap();
    timer.delay_ms(10);
    for _ in 0..100 {
        usb_dev.poll(&mut [&mut serial]);
        timer.delay_ms(10);
    }
    let mut lora = sx127x_lora::LoRa::new(
        spi0,
        nss,
        rst,
        433,
        delay
    ).expect("Could not connect to LoRa");
    let _ = lora.set_tx_power(17, 1);
    let _ = lora.set_crc(true);

    let mut led_pin = pins.led.into_push_pull_output();
    let mut buf = [0u8; 128];
    let mut i = 0;
    let mut lora_buf = [0u8; 255];
    let mut last_success_time: u64 = timer.get_counter().ticks();
    loop {
        if usb_dev.poll(&mut [&mut serial]) {
            // todo
        }
        if let Ok(size) = lora.poll_irq(None) {
            if let Ok(r_buf) = lora.read_packet() {
                let packet = &r_buf[..size];

                let _ = serial.write(b"RX RAW: ");
                for b in packet {
                    let mut hex = heapless::String::<4>::new();
                    let _ = write!(hex, "{:02X} ", b);
                    let _ = serial.write(hex.as_bytes());
                }
                let _ = serial.write(b"\r\n");

                match decrypt_packet(packet) {
                    Ok(coord) => {
                        let _ = serial.write(b"DEC lat/lon: ");
                        let mut msg = heapless::String::<64>::new();
                        let _ = write!(msg, "{}, {}\r\n", coord.lat_deg_e7, coord.lon_deg_e7);
                        let _ = serial.write(msg.as_bytes());
                    }
                    Err(err) => {
                        let mut msg = heapless::String::<64>::new();
                        let _ = match err {
                            DecryptError::PacketTooShort => write!(msg, "Decrypt error: packet too short\r\n"),
                            DecryptError::CipherError => write!(msg, "Decrypt error: cipher init failed\r\n"),
                            DecryptError::MalformedPlaintext => write!(msg, "Decrypt error: invalid plaintext\r\n"),
                        };
                        let _ = serial.write(msg.as_bytes());
                    }
                }
            }
        }
    }
}
