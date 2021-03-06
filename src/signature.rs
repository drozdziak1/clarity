use constants::SECPK1N;
use error::ClarityError;
use failure::Error;
use num256::Uint256;
use num_traits::Zero;
use serde::ser::SerializeTuple;
use serde::Serialize;
use serde::Serializer;
use utils::big_endian_int_serialize;
use utils::bytes_to_hex_str;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Signature {
    #[serde(serialize_with = "big_endian_int_serialize")]
    pub v: Uint256,
    #[serde(serialize_with = "big_endian_int_serialize")]
    pub r: Uint256,
    #[serde(serialize_with = "big_endian_int_serialize")]
    pub s: Uint256,
}

impl Signature {
    pub fn new(v: Uint256, r: Uint256, s: Uint256) -> Signature {
        Signature { v, r, s }
    }

    pub fn is_valid(&self) -> bool {
        if self.s >= *SECPK1N {
            return false;
        }

        if self.r >= *SECPK1N
            || self.s >= *SECPK1N
            || self.r == Uint256::zero()
            || self.s == Uint256::zero()
        {
            return false;
        }

        true
    }

    pub fn network_id(&self) -> Option<Uint256> {
        if self.r == Uint256::zero() && self.s == Uint256::zero() {
            Some(self.v.clone())
        } else if self.v == 27u32.into() || self.v == 28u32.into() {
            None
        } else {
            Some(((self.v.clone() - 1u32) / 2u32) - 17u32)
        }
    }

    pub fn check_low_s_metropolis(&self) -> Result<(), Error> {
        if self.s > (SECPK1N.clone() / Uint256::from(2u32)) {
            return Err(ClarityError::InvalidS.into());
        }
        Ok(())
    }

    pub fn check_low_s_homestead(&self) -> Result<(), Error> {
        if self.s > (SECPK1N.clone() / Uint256::from(2u32)) || self.s == Uint256::zero() {
            return Err(ClarityError::InvalidS.into());
        }
        Ok(())
    }
}

impl Default for Signature {
    fn default() -> Signature {
        Signature {
            r: Uint256::zero(),
            v: Uint256::zero(),
            s: Uint256::zero(),
        }
    }
}

impl ToString for Signature {
    // Constructs a string from a given signature
    // The resulting string's length is 130
    // first 32 bytes is "r" value
    // second 32 bytes i s "s" value
    // last byte is "v"
    fn to_string(&self) -> String {
        let r: [u8; 32] = self.r.clone().into();
        let s: [u8; 32] = self.s.clone().into();
        let mut wtr = vec![];
        wtr.extend(&r);
        wtr.extend(&s);

        let v = self.v.to_bytes_be();
        wtr.extend(&v[v.len() - 1..]);

        let mut result = "0x".to_owned();
        result += &bytes_to_hex_str(&wtr);
        result
    }
}

#[test]
fn new_signature() {
    let sig = Signature::new(1u32.into(), 2u32.into(), 3u32.into());
    assert_eq!(sig.v, 1u32.into());
    assert_eq!(sig.r, 2u32.into());
    assert_eq!(sig.s, 3u32.into());
}

#[test]
fn to_string() {
    let sig = Signature::new(1u32.into(), 2u32.into(), 3u32.into());
    // assert_eq!(sig.to_string().len(), 132);
    assert_eq!(
        sig.to_string(),
        concat!(
            "0x",
            "0000000000000000000000000000000000000000000000000000000000000002",
            "0000000000000000000000000000000000000000000000000000000000000003",
            "01"
        )
    );
}

#[test]
fn to_string_with_zero_v() {
    let sig = Signature::new(0u32.into(), 2u32.into(), 3u32.into());
    // assert_eq!(sig.to_string().len(), 132);
    assert_eq!(
        sig.to_string(),
        concat!(
            "0x",
            "0000000000000000000000000000000000000000000000000000000000000002",
            "0000000000000000000000000000000000000000000000000000000000000003",
            "00"
        )
    );
}
