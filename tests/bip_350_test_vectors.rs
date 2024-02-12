// BIP-350 test vectors.

#![cfg(feature = "alloc")]

use bech32grs::primitives::decode::{
    CheckedHrpstring, CheckedHrpstringError, ChecksumError, SegwitHrpstring, SegwitHrpstringError,
    UncheckedHrpstring,
};
use bech32grs::{Bech32, Bech32m};

// This is a separate test because we correctly identify this string as invalid but not for the
// reason given in the bip.
#[test]
fn bip_350_checksum_calculated_with_uppercase_form() {
    // BIP-350 states reason for error should be: "checksum calculated with uppercase form of HRP".
    let s = "M1VUXWEZ";

    assert_eq!(
        CheckedHrpstring::new::<Bech32m>(s).unwrap_err(),
        CheckedHrpstringError::Checksum(ChecksumError::InvalidChecksum)
    );

    assert_eq!(
        SegwitHrpstring::new(s).unwrap_err(),
        SegwitHrpstringError::Checksum(ChecksumError::InvalidChecksum)
    );
}

macro_rules! check_valid_bech32m {
    ($($test_name:ident, $valid_bech32m:literal);* $(;)?) => {
        $(
            #[test]
            fn $test_name() {
                let p = UncheckedHrpstring::new($valid_bech32m).unwrap();
                p.validate_checksum::<Bech32m>().expect("valid bech32m");
                // Valid bech32m strings are by definition invalid bech32.
                assert_eq!(p.validate_checksum::<Bech32>().unwrap_err(), ChecksumError::InvalidChecksum);
            }
        )*
    }
}
check_valid_bech32m! {
    valid_bech32m_hrp_string_0, "A1LQFN3A";
    valid_bech32m_hrp_string_1, "a1lqfn3a";
    valid_bech32m_hrp_string_2, "an83characterlonghumanreadablepartthatcontainsthetheexcludedcharactersbioandnumber11sg7hg6";
    valid_bech32m_hrp_string_3, "abcdef1l7aum6echk45nj3s0wdvt2fg8x9yrzpqzd3ryx";
    valid_bech32m_hrp_string_4, "11llllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllludsr8";
    valid_bech32m_hrp_string_5, "split1checkupstagehandshakeupstreamerranterredcaperredlc445v";
    valid_bech32m_hrp_string_6, "?1v759aa";
}

macro_rules! check_valid_address_roundtrip {
    ($($test_name:ident, $addr:literal);* $(;)?) => {
        $(
            #[test]
            #[cfg(feature = "alloc")]
            fn $test_name() {
                let (hrp, version, program) = bech32grs::segwit::decode($addr).expect("failed to decode valid address");
                let encoded = bech32grs::segwit::encode(&hrp, version, &program).expect("failed to encode address");

                // The bips specifically say that encoder should output lowercase characters so we uppercase manually.
                if encoded != $addr {
                    let got = encoded.to_uppercase();
                    assert_eq!(got, $addr)
                }
            }
        )*
    }
}
// Note these test vectors include various witness versions.
check_valid_address_roundtrip! {
    bip_350_valid_address_roundtrip_0, "GRS1QW508D6QEJXTDG4Y5R3ZARVARY0C5XW7K3K4SJ5";
    bip_350_valid_address_roundtrip_1, "tgrs1qrp33g0q5c5txsp9arysrx4k6zdkfs4nce4xj0gdcccefvpysxf3quvjfuq";
    bip_350_valid_address_roundtrip_2, "tgrs1qqqqqp399et2xygdj5xreqhjjvcmzhxw4aywxecjdzew6hylgvsess668a6";
    bip_350_valid_address_roundtrip_3, "tgrs1pqqqqp399et2xygdj5xreqhjjvcmzhxw4aywxecjdzew6hylgvses6d6w9x";
    bip_350_valid_address_roundtrip_4, "grs1p0xlxvlhemja6c4dqv22uapctqupfhlxm9h8z3k2e72q4k9hcz7vqddt7at";
}

macro_rules! check_invalid_address {
    ($($test_name:ident, $addr:literal);* $(;)?) => {
        $(
            #[test]
            #[cfg(feature = "alloc")]
            fn $test_name() {
                match SegwitHrpstring::new($addr) {
                    Err(_) => {},
                    // We do not enforce the bip specified restrictions when constructing
                    // SegwitHrpstring so must explicitly do check.
                    Ok(segwit) => assert!(!segwit.has_valid_hrp()),
                }
            }
        )*
    }
}
check_invalid_address! {
    // Invalid human-readable part
    bip_350_invalid_address_0, "tgrt1p0xlxvlhemja6c4dqv22uapctqupfhlxm9h8z3k2e72q4k9hcz7vqddt7at";
    // Invalid checksums (Bech32 instead of Bech32m):
    bip_350_invalid_address_1, "grs1p0xlxvlhemja6c4dqv22uapctqupfhlxm9h8z3k2e72q4k9hcz7vqh2y7hd";
    bip_350_invalid_address_2, "tgrs1z0xlxvlhemja6c4dqv22uapctqupfhlxm9h8z3k2e72q4k9hcz7vqglt7rf";
    bip_350_invalid_address_3, "GRS1S0XLXVLHEMJA6C4DQV22UAPCTQUPFHLXM9H8Z3K2E72Q4K9HCZ7VQ54WELL";
    bip_350_invalid_address_4, "grs1qw508d6qejxtdg4y5r3zarvary0c5xw7kemeawh";
    bip_350_invalid_address_5, "tgrs1q0xlxvlhemja6c4dqv22uapctqupfhlxm9h8z3k2e72q4k9hcz7vq24jc47";
    // Invalid character in checksum
    bip_350_invalid_address_6, "grs1p38j9r5y49hruaue7wxjce0updqjuyyx0kh56v8s25huc6995vvpql3jow4";
    // Invalid witness version
    bip_350_invalid_address_7, "GRS130XLXVLHEMJA6C4DQV22UAPCTQUPFHLXM9H8Z3K2E72Q4K9HCZ7VQ7ZWS8R";
    // Invalid program length (1 byte)
    bip_350_invalid_address_8, "grs1pw5dgrnzv";
    // Invalid program length (41 bytes)
    bip_350_invalid_address_9, "grs1p0xlxvlhemja6c4dqv22uapctqupfhlxm9h8z3k2e72q4k9hcz7v8n0nx0muaewav253zgeav";
    // Invalid program length for witness version 0 (per BIP-141)
    bip_350_invalid_address_10, "GRS1QR508D6QEJXTDG4Y5R3ZARVARYV98GJ9P";
    // Mixed case
    bip_350_invalid_address_11, "tgrs1p0xlxvlhemja6c4dqv22uapctqupfhlxm9h8z3k2e72q4k9hcz7vq47Zagq";
    // zero padding of more than 4 bits
    bip_350_invalid_address_12, "grs1p0xlxvlhemja6c4dqv22uapctqupfhlxm9h8z3k2e72q4k9hcz7v07qwwzcrf";
    // Non-zero padding in 8-to-5 conversion
    bip_350_invalid_address_13, "tgrs1p0xlxvlhemja6c4dqv22uapctqupfhlxm9h8z3k2e72q4k9hcz7vpggkg4j";
    // Empty data section
    bip_350_invalid_address_14, "grs1gmk9yu";
}
