use usbd_serial::SerialPort;
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

pub fn gps_proccess(line: &[u8], serial: &mut SerialPort<UsbBus>) -> bool {
    if field(line, 0) != Some(b"$GNGGA") {
        return false;
    }
    let fix_quality = field(line, 6);
    if fix_quality != Some(b"1") && fix_quality != Some(b"2") {
        let _ = serial.write(b"GGA invalid\r\n");
        return false;
    }
    let lat_raw = match field(line, 2) { Some(v) => v, None => return false, };
    let lat_hemi = match field(line, 3) { Some(v) => v, None => return false, };
    let lon_raw = match field(line, 4) { Some(v) => v, None => return false, };
    let lon_hemi = match field(line, 5) { Some(v) => v, None => return false, };

    let count_sat = match field(line, 7) { Some(v) => v, None => return false, };
    let _ = serial.write(b"Satellites: ");
    let _ = serial.write(count_sat);
    let _ = serial.write(b"\r\n");

    let mut lat = match nmea_to_e7(lat_raw, true) { Some(v) => v, None => return false, };
    let mut lon = match nmea_to_e7(lon_raw, false) { Some(v) => v, None => return false, };

    if lat_hemi == b"S" { lat = -lat; }
    if lon_hemi == b"W" { lon = -lon; }

    // Convert to decimal degrees string
    let print_deg = |val: i32, buf: &mut [u8; 32]| {
        let sign = if val < 0 { b"-" } else { b" " };
        let a = val.abs() as u32;
        let int = a / 10_000_000;
        let frac = a % 10_000_000;
        let s = core::fmt::Write::write_fmt(
            &mut heapless::String::<32>::new(),
            format_args!("{}{}.{:07}", core::str::from_utf8(sign).unwrap(), int, frac)
        );
        s
    };

    // Direct fixed-point print (simpler)
    use core::fmt::Write;
    let mut out = heapless::String::<64>::new();
    let _ = write!(out, "lat_e7={}, lon_e7={}\r\n", lat, lon);
    let _ = serial.write(out.as_bytes());

    return true;
}