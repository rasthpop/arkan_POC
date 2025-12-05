use core::convert::TryInto;
use chacha20::{cipher::{KeyIvInit, StreamCipher}, ChaCha20};

// 32‑byte shared key (same on sender and receiver)
const KEY: [u8; 32] = [
    0x47, 0xa5, 0x00, 0x52, 0x7a, 0xef, 0x77, 0x0d,
    0x36, 0x3c, 0x0b, 0xe3, 0xe2, 0xaf, 0x50, 0xa8,
    0x1d, 0x62, 0x3e, 0x9e, 0x2d, 0x1a, 0x21, 0xc0,
    0x15, 0x3a, 0x9d, 0x53, 0xa7, 0x0f, 0x79, 0xd4,
];

pub const NONCE_LEN: usize = 12;
pub const COORD_LEN: usize = 8;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GpsCoord {
    pub lat_deg_e7: i32,
    pub lon_deg_e7: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecryptError {
    PacketTooShort,
    CipherError,
    MalformedPlaintext,
}

pub fn decrypt_packet(packet: &[u8]) -> Result<GpsCoord, DecryptError> {
    if packet.len() < NONCE_LEN + COORD_LEN {
        return Err(DecryptError::PacketTooShort);
    }

    let (nonce, ciphertext) = packet.split_at(NONCE_LEN);

    let mut buf = [0u8; COORD_LEN];
    if ciphertext.len() < COORD_LEN {
        return Err(DecryptError::PacketTooShort);
    }
    buf.copy_from_slice(&ciphertext[..COORD_LEN]);

    let mut cipher = ChaCha20::new_from_slices(&KEY, nonce).map_err(|_| DecryptError::CipherError)?;
    cipher.apply_keystream(&mut buf);

    let lat = i32::from_le_bytes(buf[0..4].try_into().map_err(|_| DecryptError::MalformedPlaintext)?);
    let lon = i32::from_le_bytes(buf[4..8].try_into().map_err(|_| DecryptError::MalformedPlaintext)?);

    Ok(GpsCoord { lat_deg_e7: lat, lon_deg_e7: lon })
}

