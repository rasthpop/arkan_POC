//! Encryption template for GPS coordinates (no_std-friendly).
//!
//! What this gives you:
//! - A `GpsCoord` data model (fixed-point, integer-based).
//! - An `EncryptConfig` for key/IV/AAD inputs.
//! - An `EncryptionError` and `Result` alias.
//! - A `CoordinateEncryptor` trait describing the contract.
//! - A stub `MyCipher` you can fill with your algorithm.
//!
//! What it does NOT include:
//! - Any GPS receiving/driver logic (you said this is elsewhere).
//! - Any real encryption logic (intentionally left as TODOs).
//!
//! Example (how you might use it from `main.rs`):
//!
//! ```ignore
//! // Add at the top of main.rs
//! // mod encryption;
//!
//! `use crate::encryption::{CoordinateEncryptor, EncryptConfig, GpsCoord, MyCipher};`
//!
//! // ... after you have valid coordinates from your GPS module:
//! let coords = GpsCoord {
//!     // Degrees in 1e-7 fixed-point format (e.g., 50.4501000° => 504501000)
//!     lat_deg_e7: 504501000,
//!     lon_deg_e7: 306232000,
//!     // Altitude in millimeters (optional). For example 123.456 m = 123456 mm
//!     alt_mm: Some(123_456),
//! };
//!
//! let cfg = EncryptConfig {
//!     key: b"YOUR-KEY-BYTES-HERE",   // TODO: supply your real key bytes
//!     iv: Some(&[0u8; 12]),           // TODO: supply your IV/nonce if your scheme needs it
//!     aad: None,                      // Optional AAD for AEAD schemes
//! };
//!
//! let mut cipher = MyCipher::default();
//! let mut out = [0u8; 128]; // caller-provided buffer for ciphertext/tag
//! let used = cipher.encrypt_into(&coords, &cfg, &mut out).unwrap();
//! let ciphertext = &out[..used];
//! // Transmit or store `ciphertext` 
//! ```

#![allow(dead_code)]

use core::fmt;

/// GPS coordinates in fixed-point format for deterministic, no_std-friendly use.
/// - `lat_deg_e7` and `lon_deg_e7`: degrees scaled by 1e7 (e.g., 50.4501° => 504501000)
/// - `alt_mm`: optional altitude in millimeters
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct GpsCoord {
    pub lat_deg_e7: i32,
    pub lon_deg_e7: i32,
    pub alt_mm: Option<i32>,
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
///
/// Contract (recommended):
/// - Deterministic serialization of coordinates to bytes.
/// - Proper handling of key/IV sizes and errors.
/// - Caller-provided output buffer to avoid allocations.
/// - Return the number of bytes written into `out`.
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
    /// Create a new instance; extend as needed.
    pub const fn new() -> Self {
        Self {}
    }

    /// Example helper to serialize coordinates into a small fixed buffer.
    /// You can replace this with your own canonical serialization.
    ///
    /// Binary format (little-endian):
    /// - lat_deg_e7: i32 (4 bytes)
    /// - lon_deg_e7: i32 (4 bytes)
    /// - alt_mm_present: u8 (1 if Some, 0 if None)
    /// - alt_mm: i32 (4 bytes) only when present
    fn serialize_coords<'b>(&self, coords: &GpsCoord, buf: &'b mut [u8]) -> Result<&'b [u8]> {
        // Minimum 9 bytes without altitude, 13 bytes with altitude.
        let need = if coords.alt_mm.is_some() { 13 } else { 9 };
        if buf.len() < need { return Err(EncryptionError::BufferTooSmall); }

        // Write lat (LE)
        buf[0..4].copy_from_slice(&coords.lat_deg_e7.to_le_bytes());
        // Write lon (LE)
        buf[4..8].copy_from_slice(&coords.lon_deg_e7.to_le_bytes());
        // Flag for altitude presence
        buf[8] = if coords.alt_mm.is_some() { 1 } else { 0 };

        let used = if let Some(alt) = coords.alt_mm {
            buf[9..13].copy_from_slice(&alt.to_le_bytes());
            13
        } else {
            9
        };

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