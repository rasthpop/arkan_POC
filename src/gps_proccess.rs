use heapless;
use usbd_serial::SerialPort;
pub fn gps_proccess(line: &[u8], serial: &mut SerialPort<rp_pico::hal::usb::UsbBus>){
    
    let _ = serial.write(line);
    // TODO
}