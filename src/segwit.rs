// SPDX-License-Identifier: MIT

//! Segregated Witness API - enables typical usage for encoding and decoding segwit addresses.
//!
//! [BIP-173] and [BIP-350] contain some complexity. This module aims to allow you to create modern
//! Bitcoin addresses correctly and easily without intimate knowledge of the BIPs. However, if you
//! do posses such knowledge and are doing unusual things you may prefer to use the `primitives`
//! submodules directly.
//!
//! # Examples
//!
//! ```
//! # #[cfg(feature = "alloc")] {
//! use bech32::primitives::hrp::{self, Hrp};
//! use bech32::{Fe32, segwit};
//!
//! let witness_prog = [
//!     0x75, 0x1e, 0x76, 0xe8, 0x19, 0x91, 0x96, 0xd4,
//!     0x54, 0x94, 0x1c, 0x45, 0xd1, 0xb3, 0xa3, 0x23,
//!     0xf1, 0x43, 0x3b, 0xd6,
//! ];
//!
//! // Encode a taproot address suitable for use on mainnet.
//! let _ = segwit::encode_v1(&hrp::GRS, &witness_prog);
//!
//! // Encode a segwit v0 address suitable for use on testnet.
//! let _ = segwit::encode_v0(&hrp::TGRS, &witness_prog);
//!
//! // If you have the witness version already you can use:
//! # let witness_version = segwit::VERSION_0;
//! let _ = segwit::encode(&hrp::GRS, witness_version, &witness_prog);
//!
//! // Decode a Groestlcoin bech32 segwit address.
//! let address = "grs1q2s3rjwvam9dt2ftt4sqxqjf3twav0gdx0k0q2etxflx38c3x8tnslkylay";
//! let (hrp, witness_version, witness_program) = segwit::decode(address).expect("failed to decode address");
//! # }
//! ```
//!
//! [BIP-173]: <https://github.com/bitcoin/bips/blob/master/bip-0173.mediawiki>
//! [BIP-350]: <https://github.com/bitcoin/bips/blob/master/bip-0350.mediawiki>
//! [`bip_173_test_vectors.rs`]: <https://github.com/rust-bitcoin/rust-bech32/blob/master/tests/bip_173_test_vectors.rs>
//! [`bip_350_test_vectors.rs`]: <https://github.com/rust-bitcoin/rust-bech32/blob/master/tests/bip_350_test_vectors.rs>

#[cfg(all(feature = "alloc", not(feature = "std"), not(test)))]
use alloc::{string::String, vec::Vec};
use core::fmt;

use crate::error::write_err;
#[cfg(feature = "alloc")]
use crate::primitives::decode::{SegwitHrpstring, SegwitHrpstringError};
use crate::primitives::gf32::Fe32;
use crate::primitives::hrp::Hrp;
use crate::primitives::iter::{ByteIterExt, Fe32IterExt};
use crate::primitives::segwit::{self, InvalidWitnessVersionError, WitnessLengthError};
pub use crate::primitives::segwit::{VERSION_0, VERSION_1};
use crate::primitives::{Bech32, Bech32m};

/// Decodes a segwit address.
///
/// # Examples
///
/// ```
/// use bech32::segwit;
/// let address = "grs1py3m7vwnghyne9gnvcjw82j7gqt2rafgdmlmwmqnn3hvcmdm09rjqhnu8f5";
/// let (_hrp, _witness_version, _witness_program) = segwit::decode(address).expect("failed to decode address");
/// ```
#[cfg(feature = "alloc")]
#[inline]
pub fn decode(s: &str) -> Result<(Hrp, Fe32, Vec<u8>), SegwitHrpstringError> {
    let segwit = SegwitHrpstring::new(s)?;
    Ok((segwit.hrp(), segwit.witness_version(), segwit.byte_iter().collect::<Vec<u8>>()))
}

/// Encodes a segwit address.
///
/// Does validity checks on the `witness_version` and length checks on the `witness_program`.
///
/// As specified by [`BIP-350`] we use the [`Bech32m`] checksum algorithm for witness versions 1 and
/// above, and for witness version 0 we use the original ([`BIP-173`]) [`Bech32`] checksum
/// algorithm.
///
/// See also [`encode_v0`] or [`encode_v1`].
///
/// [`Bech32`]: crate::primitives::Bech32
/// [`Bech32m`]: crate::primitives::Bech32m
/// [BIP-173]: <https://github.com/bitcoin/bips/blob/master/bip-0173.mediawiki>
/// [BIP-350]: <https://github.com/bitcoin/bips/blob/master/bip-0350.mediawiki>
#[cfg(feature = "alloc")]
#[inline]
pub fn encode(
    hrp: &Hrp,
    witness_version: Fe32,
    witness_program: &[u8],
) -> Result<String, EncodeError> {
    segwit::validate_witness_version(witness_version)?;
    segwit::validate_witness_program_length(witness_program.len(), witness_version)?;

    let mut buf = String::new();
    encode_to_fmt_unchecked(&mut buf, hrp, witness_version, witness_program)?;
    Ok(buf)
}

/// Encodes a segwit version 0 address.
#[cfg(feature = "alloc")]
#[inline]
pub fn encode_v0(hrp: &Hrp, witness_program: &[u8]) -> Result<String, EncodeError> {
    encode(hrp, VERSION_0, witness_program)
}

/// Encodes a segwit version 1 address.
#[cfg(feature = "alloc")]
#[inline]
pub fn encode_v1(hrp: &Hrp, witness_program: &[u8]) -> Result<String, EncodeError> {
    encode(hrp, VERSION_1, witness_program)
}

/// Encodes a segwit address to a writer ([`fmt::Write`]) using lowercase characters.
///
/// Does not check the validity of the witness version and witness program lengths (see
/// the [`crate::primitives::segwit`] module for validation functions).
#[inline]
pub fn encode_to_fmt_unchecked<W: fmt::Write>(
    fmt: &mut W,
    hrp: &Hrp,
    witness_version: Fe32,
    witness_program: &[u8],
) -> fmt::Result {
    let iter = witness_program.iter().copied().bytes_to_fes();
    match witness_version {
        VERSION_0 => {
            for c in iter.with_checksum::<Bech32>(hrp).with_witness_version(VERSION_0).chars() {
                fmt.write_char(c)?;
            }
        }
        version => {
            for c in iter.with_checksum::<Bech32m>(hrp).with_witness_version(version).chars() {
                fmt.write_char(c)?;
            }
        }
    }
    Ok(())
}

/// Encodes a segwit address to a writer ([`fmt::Write`]) using uppercase characters.
///
/// This is provided for use when creating QR codes.
///
/// Does not check the validity of the witness version and witness program lengths (see
/// the [`crate::primitives::segwit`] module for validation functions).
#[inline]
pub fn encode_to_fmt_unchecked_uppercase<W: fmt::Write>(
    fmt: &mut W,
    hrp: &Hrp,
    witness_version: Fe32,
    witness_program: &[u8],
) -> fmt::Result {
    let iter = witness_program.iter().copied().bytes_to_fes();
    match witness_version {
        VERSION_0 => {
            for c in iter.with_checksum::<Bech32>(hrp).with_witness_version(VERSION_0).chars() {
                fmt.write_char(c.to_ascii_uppercase())?;
            }
        }
        version => {
            for c in iter.with_checksum::<Bech32m>(hrp).with_witness_version(version).chars() {
                fmt.write_char(c.to_ascii_uppercase())?;
            }
        }
    }

    Ok(())
}

/// An error while constructing a [`SegwitHrpstring`] type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EncodeError {
    /// Invalid witness version (must be 0-16 inclusive).
    WitnessVersion(InvalidWitnessVersionError),
    /// Invalid witness length.
    WitnessLength(WitnessLengthError),
    /// Writing to formatter failed.
    Write(fmt::Error),
}

impl fmt::Display for EncodeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use EncodeError::*;

        match *self {
            WitnessVersion(ref e) => write_err!(f, "witness version"; e),
            WitnessLength(ref e) => write_err!(f, "witness length"; e),
            Write(ref e) => write_err!(f, "writing to formatter failed"; e),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for EncodeError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        use EncodeError::*;

        match *self {
            WitnessVersion(ref e) => Some(e),
            WitnessLength(ref e) => Some(e),
            Write(ref e) => Some(e),
        }
    }
}

impl From<InvalidWitnessVersionError> for EncodeError {
    fn from(e: InvalidWitnessVersionError) -> Self { Self::WitnessVersion(e) }
}

impl From<WitnessLengthError> for EncodeError {
    fn from(e: WitnessLengthError) -> Self { Self::WitnessLength(e) }
}

impl From<fmt::Error> for EncodeError {
    fn from(e: fmt::Error) -> Self { Self::Write(e) }
}

#[cfg(all(test, feature = "alloc"))]
mod tests {
    use super::*;
    use crate::primitives::hrp;

    #[test]
    // Just shows we handle both v0 and v1 addresses, for complete test
    // coverage see primitives submodules and test vectors.
    fn roundtrip_valid_mainnet_addresses() {
        // A few recent addresses from mainnet (Block 801266).
        let addresses = vec![
            "grs1q2s3rjwvam9dt2ftt4sqxqjf3twav0gdx0k0q2etxflx38c3x8tnslkylay", // Segwit v0
            "grs1py3m7vwnghyne9gnvcjw82j7gqt2rafgdmlmwmqnn3hvcmdm09rjqhnu8f5", // Segwit v1
        ];

        for address in addresses {
            let (hrp, version, program) = decode(address).expect("failed to decode valid address");
            let encoded = encode(&hrp, version, &program).expect("failed to encode address");
            assert_eq!(encoded, address);
        }
    }

    fn witness_program() -> [u8; 20] {
        [
            0x75, 0x1e, 0x76, 0xe8, 0x19, 0x91, 0x96, 0xd4, 0x54, 0x94, 0x1c, 0x45, 0xd1, 0xb3,
            0xa3, 0x23, 0xf1, 0x43, 0x3b, 0xd6,
        ]
    }

    #[test]
    fn encode_to_fmt_lowercase() {
        let program = witness_program();
        let mut address = String::new();
        encode_to_fmt_unchecked(&mut address, &hrp::GRS, VERSION_0, &program)
            .expect("failed to encode address to QR code");

        let want = "grs1qw508d6qejxtdg4y5r3zarvary0c5xw7k3k4sj5";
        assert_eq!(address, want);
    }

    #[test]
    fn encode_to_fmt_uppercase() {
        let program = witness_program();
        let mut address = String::new();
        encode_to_fmt_unchecked_uppercase(&mut address, &hrp::GRS, VERSION_0, &program)
            .expect("failed to encode address to QR code");

        let want = "GRS1QW508D6QEJXTDG4Y5R3ZARVARY0C5XW7K3K4SJ5";
        assert_eq!(address, want);
    }
}
