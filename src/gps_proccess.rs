use usbd_serial::SerialPort;
use crate::encryption::{MyCipher, GpsCoord, EncryptConfig, CoordinateEncryptor};
type UsbBus = rp_pico::hal::usb::UsbBus;

fn field<'a>(s: &'a [u8], idx: usize) -> Option<&'a [u8]> {
    let mut start = 0;
    let mut current = 0;
    for i in 0..=s.len() {
        if i == s.len() || s[i] == b',' || s[i] == b'*' {
            if current == idx {
                return Some(&s[start..i]);
            }
            current += 1;
            start = i + 1;
        }
    }
    None
}

// Convert "ddmm.mmmm" (lat) or "dddmm.mmmm" (lon) to degrees * 1e7 (i32)
fn nmea_to_e7(txt: &[u8], is_lat: bool) -> Option<i32> {
    let s = core::str::from_utf8(txt).ok()?;
    let deg_len = if is_lat { 2 } else { 3 };
    if s.len() < deg_len + 2 { return None; }
    let (deg_part, min_part) = s.split_at(deg_len);
    let deg: i64 = deg_part.parse().ok()?;

    // Parse minutes with fractional part
    let mut minutes_scaled = 0i64;
    let mut scale = 1i64;
    let mut after_dot = false;
    for c in min_part.bytes() {
        match c {
            b'0'..=b'9' => {
                minutes_scaled = minutes_scaled * 10 + (c - b'0') as i64;
                if after_dot { scale *= 10; }
            }
            b'.' if !after_dot => after_dot = true,
            _ => break,
        }
    }
    let deg_e7 = deg * 10_000_000 + ((minutes_scaled * 10_000_000 + (60 * scale / 2)) / (60 * scale));
    Some(deg_e7 as i32)
}

// 32‑byte shared key (same on sender and receiver)
const KEY: [u8; 32] = [
    0x47, 0xa5, 0x00, 0x52, 0x7a, 0xef, 0x77, 0x0d,
    0x36, 0x3c, 0x0b, 0xe3, 0xe2, 0xaf, 0x50, 0xa8,
    0x1d, 0x62, 0x3e, 0x9e, 0x2d, 0x1a, 0x21, 0xc0,
    0x15, 0x3a, 0x9d, 0x53, 0xa7, 0x0f, 0x79, 0xd4,
];

static mut NONCE_COUNTER: u64 = 0;

fn next_nonce() -> [u8; 12] {
    // nonce = 4 bytes constant + 8‑byte counter
    let ctr = unsafe {
        let c = NONCE_COUNTER;
        NONCE_COUNTER = NONCE_COUNTER.wrapping_add(1);
        c
    };
    let mut n = [0u8; 12];
    n[0..4].copy_from_slice(&[0x01, 0x02, 0x03, 0x04]); // device id, for example
    n[4..12].copy_from_slice(&ctr.to_be_bytes());
    n
}

pub fn gps_proccess(
    line: &[u8],
    serial: &mut SerialPort<UsbBus>,
    lora_buf: &mut [u8; 255],
) -> Option<usize> {
    if field(line, 0) != Some(b"$GNGGA") {
        return None;
    }
    let fix_quality = field(line, 6);
    if fix_quality != Some(b"1") && fix_quality != Some(b"2") {
        let _ = serial.write(b"GGA invalid\r\n");
        return None;
    }
    let lat_raw = field(line, 2)?;
    let lat_hemi = field(line, 3)?;
    let lon_raw = field(line, 4)?;
    let lon_hemi = field(line, 5)?;

    let count_sat = field(line, 7)?;
    let _ = serial.write(b"Satellites: ");
    let _ = serial.write(count_sat);
    let _ = serial.write(b"\r\n");

    let mut lat = nmea_to_e7(lat_raw, true)?;
    let mut lon = nmea_to_e7(lon_raw, false)?;

    if lat_hemi == b"S" { lat = -lat; }
    if lon_hemi == b"W" { lon = -lon; }

    // Print raw fixed‑point coords to serial
    use core::fmt::Write;
    let mut out = heapless::String::<64>::new();
    let _ = write!(out, "RAW lat_e7={}, lon_e7={}\r\n", lat, lon);
    let _ = serial.write(out.as_bytes());

    // Prepare coords struct for encryption
    let coords = GpsCoord {
        lat_deg_e7: lat,
        lon_deg_e7: lon,
    };

    // Build fresh nonce and encryption config
    let nonce = next_nonce();
    let enc_cfg = EncryptConfig {
        key: &KEY,
        iv: Some(&nonce),
        aad: None,
    };

    // Encrypt into a small temp buffer
    let mut cipher = MyCipher::new();
    let mut ct = [0u8; 16];
    let enc_len = match cipher.encrypt_into(&coords, &enc_cfg, &mut ct) {
        Ok(len) => len,
        Err(_) => {
            let _ = serial.write(b"Encryption error\r\n");
            return None;
        }
    };

    // Log encrypted bytes to serial as hex
    let _ = serial.write(b"NONCE: ");
    for b in &nonce {
        let mut s = heapless::String::<4>::new();
        let _ = write!(s, "{:02X}", b);
        let _ = serial.write(s.as_bytes());
    }
    let _ = serial.write(b"\r\n");

    let _ = serial.write(b"CIPHERTEXT: ");
    for b in &ct[..enc_len] {
        let mut s = heapless::String::<4>::new();
        let _ = write!(s, "{:02X}", b);
        let _ = serial.write(s.as_bytes());
    }
    let _ = serial.write(b"\r\n");

    // Final LoRa payload: [nonce || ciphertext]
    if lora_buf.len() < 12 + enc_len {
        let _ = serial.write(b"LoRa buffer too small\r\n");
        return None;
    }

    lora_buf[..12].copy_from_slice(&nonce);
    lora_buf[12..12 + enc_len].copy_from_slice(&ct[..enc_len]);

    // Return total payload length for caller to send
    Some(12 + enc_len)
}