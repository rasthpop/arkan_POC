#![allow(dead_code)]

use core::fmt;

/// - `lat_deg_e7` and `lon_deg_e7`: degrees scaled by 1e7 (e.g., 50.4501° => 504501000)
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct GpsCoord {
    pub lat_deg_e7: i32,
    pub lon_deg_e7: i32,
}

/// Configuration inputs commonly needed by encryption algorithms.
/// - `key`: secret key bytes (size depends on your algorithm)
/// - `iv`: optional nonce/IV (required for many stream/AEAD schemes)
/// - `aad`: optional associated data (for AEAD; not encrypted but authenticated)
#[derive(Clone, Copy)]
pub struct EncryptConfig<'a> {
    pub key: &'a [u8],
    pub iv: Option<&'a [u8]>,
    pub aad: Option<&'a [u8]>,
}

/// Errors that can occur during encryption.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum EncryptionError {
    InvalidKey,
    InvalidNonce,
    BufferTooSmall,
    SerializationFailed,
    Unimplemented,
    Other,
}

impl fmt::Debug for EncryptionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            EncryptionError::InvalidKey => "InvalidKey",
            EncryptionError::InvalidNonce => "InvalidNonce",
            EncryptionError::BufferTooSmall => "BufferTooSmall",
            EncryptionError::SerializationFailed => "SerializationFailed",
            EncryptionError::Unimplemented => "Unimplemented",
            EncryptionError::Other => "Other",
        })
    }
}

/// Convenient alias for results from this module.
pub type Result<T> = core::result::Result<T, EncryptionError>;

/// Trait describing the contract for a coordinate encryption algorithm.
pub trait CoordinateEncryptor {
    /// Encrypts the given coordinates into `out` using provided config.
    /// Returns the number of bytes written.
    fn encrypt_into(&mut self, coords: &GpsCoord, cfg: &EncryptConfig, out: &mut [u8]) -> Result<usize>;
}

/// A stub cipher you can replace with your own implementation.
/// Fill the TODO sections inside `encrypt_into`.
pub struct MyCipher {
    // TODO: add fields needed by your algorithm (e.g., key schedule, precomputed tables, etc.)
}

impl MyCipher {
    pub const fn new() -> Self {
        Self {}
    }

    /// Example helper to serialize coordinates into a small fixed buffer.
    /// You can replace this with your own canonical serialization.
    ///
    /// Binary format (little-endian):
    /// - lat_deg_e7: i32 (4 bytes)
    /// - lon_deg_e7: i32 (4 bytes)
    fn serialize_coords<'b>(&self, coords: &GpsCoord, buf: &'b mut [u8]) -> Result<&'b [u8]> {
        // Minimum 9 bytes without altitude, 13 bytes with altitude.
        let need = if coords.alt_mm.is_some() { 13 } else { 9 };
        if buf.len() < need { return Err(EncryptionError::BufferTooSmall); }

        // Write lat (LE)
        buf[0..4].copy_from_slice(&coords.lat_deg_e7.to_le_bytes());
        // Write lon (LE)
        buf[4..8].copy_from_slice(&coords.lon_deg_e7.to_le_bytes());

        Ok(&buf[..used])
    }
}

impl Default for MyCipher { fn default() -> Self { Self::new() } }


impl CoordinateEncryptor for MyCipher {
    fn encrypt_into(&mut self, coords: &GpsCoord, cfg: &EncryptConfig, out: &mut [u8]) -> Result<usize> {
        use chacha20::{cipher::{KeyIvInit, StreamCipher}, ChaCha20};

        // 1) Validate key (32 bytes) and nonce (12 bytes)
        let key = cfg.key;
        if key.len() != 32 {
            return Err(EncryptionError::InvalidKey);
        }
        let nonce = cfg.iv.ok_or(EncryptionError::InvalidNonce)?;
        if nonce.len() != 12 {
            return Err(EncryptionError::InvalidNonce);
        }

        // 2) Serialize coords
        let mut tmp = [0u8; 16];
        let plaintext = self.serialize_coords(coords, &mut tmp)?;
        let pt_len = plaintext.len();

        if out.len() < pt_len {
            return Err(EncryptionError::BufferTooSmall);
        }

        // 3) Copy plaintext into out, then apply stream cipher in place
        out[..pt_len].copy_from_slice(plaintext);

        // ChaCha20 counter starts at 0 (default). If you need per‑packet sequencing,
        // fold a message counter into the nonce instead of bumping the internal block counter.
        let mut cipher = ChaCha20::new_from_slices(key, nonce).map_err(|_| EncryptionError::Other)?;
        cipher.apply_keystream(&mut out[..pt_len]);

        Ok(pt_len)
    }
}