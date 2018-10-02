use address::Address;
use error::ClarityError;
use failure::Error;
use secp256k1::{PublicKey, Secp256k1, SecretKey};
use sha3::{Digest, Keccak256};
use std::str::FromStr;
use utils::{hex_str_to_bytes, ByteDecodeError};

#[derive(Fail, Debug, PartialEq)]
pub enum PrivateKeyError {
    #[fail(display = "Private key should be exactly 64 bytes")]
    InvalidLengthError,
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct PrivateKey([u8; 32]);

impl FromStr for PrivateKey {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() != 64 {
            return Err(PrivateKeyError::InvalidLengthError.into());
        }
        let bytes = hex_str_to_bytes(&s)?;
        debug_assert_eq!(bytes.len(), 32);
        let mut res = [0x0u8; 32];
        res.copy_from_slice(&bytes[..]);
        Ok(PrivateKey(res))
    }
}

impl From<[u8; 32]> for PrivateKey {
    fn from(val: [u8; 32]) -> PrivateKey {
        PrivateKey(val)
    }
}

impl PrivateKey {
    pub fn new() -> PrivateKey {
        PrivateKey([0u8; 32])
    }

    pub fn from_slice(slice: &[u8]) -> Result<PrivateKey, Error> {
        if slice.len() != 32 {
            return Err(ClarityError::InvalidPrivKey.into());
        }
        let mut res = [0u8; 32];
        res.copy_from_slice(slice);
        Ok(PrivateKey(res))
    }

    pub fn to_bytes(&self) -> [u8; 32] {
        self.0
    }

    /// Given a private key it generates a valid public key.
    ///
    /// This is well explained in the EthereumYellow Paper Appendix F.
    pub fn to_public_key(&self) -> Result<Address, Error> {
        let secp256k1 = Secp256k1::new();
        let sk = SecretKey::from_slice(&secp256k1, &self.0)?;
        let pkey = PublicKey::from_secret_key(&secp256k1, &sk);
        // TODO: This part is duplicated with sender code.

        // Serialize the recovered public key in uncompressed format
        let pkey = pkey.serialize_uncompressed();
        assert_eq!(pkey.len(), 65);
        if pkey[1..].to_vec() == [0x00u8; 64].to_vec() {
            return Err(ClarityError::ZeroPrivKey.into());
        }
        // Finally an address is last 20 bytes of a hash of the public key.
        let sender = Keccak256::digest(&pkey[1..]);
        debug_assert_eq!(sender.len(), 32);
        Ok(Address::from(&sender[12..]))
    }
}

#[test]
#[should_panic]
fn too_short() {
    PrivateKey::from_str("abcdef").unwrap();
}

#[test]
#[should_panic]
fn invalid_data() {
    let key = "\u{012345}c85ef7d79691fe79573b1a7064c19c1a9819ebdbd1faaab1a8ec92344438";
    assert_eq!(key.len(), 64);
    PrivateKey::from_str(key).unwrap();
}

#[test]
fn parse_address_1() {
    use utils::bytes_to_hex_str;
    // https://github.com/ethereum/tests/blob/b44cea1cccf1e4b63a05d1ca9f70f2063f28da6d/BasicTests/txtest.json
    let key: PrivateKey = "c85ef7d79691fe79573b1a7064c19c1a9819ebdbd1faaab1a8ec92344438aaf4"
        .parse()
        .unwrap();
    assert_eq!(
        key.to_bytes(),
        [
            0xc8, 0x5e, 0xf7, 0xd7, 0x96, 0x91, 0xfe, 0x79, 0x57, 0x3b, 0x1a, 0x70, 0x64, 0xc1,
            0x9c, 0x1a, 0x98, 0x19, 0xeb, 0xdb, 0xd1, 0xfa, 0xaa, 0xb1, 0xa8, 0xec, 0x92, 0x34,
            0x44, 0x38, 0xaa, 0xf4
        ]
    );

    // geth account import <(echo c85ef7d79691fe79573b1a7064c19c1a9819ebdbd1faaab1a8ec92344438aaf4)
    assert_eq!(
        bytes_to_hex_str(&key.to_public_key().unwrap().as_bytes()),
        "cd2a3d9f938e13cd947ec05abc7fe734df8dd826"
    );
}

#[test]
fn parse_address_2() {
    use utils::bytes_to_hex_str;
    // https://github.com/ethereum/tests/blob/b44cea1cccf1e4b63a05d1ca9f70f2063f28da6d/BasicTests/txtest.json
    let key: PrivateKey = "c87f65ff3f271bf5dc8643484f66b200109caffe4bf98c4cb393dc35740b28c0"
        .parse()
        .unwrap();
    assert_eq!(
        key.to_bytes(),
        [
            0xc8, 0x7f, 0x65, 0xff, 0x3f, 0x27, 0x1b, 0xf5, 0xdc, 0x86, 0x43, 0x48, 0x4f, 0x66,
            0xb2, 0x00, 0x10, 0x9c, 0xaf, 0xfe, 0x4b, 0xf9, 0x8c, 0x4c, 0xb3, 0x93, 0xdc, 0x35,
            0x74, 0x0b, 0x28, 0xc0
        ]
    );

    // geth account import <(echo c87f65ff3f271bf5dc8643484f66b200109caffe4bf98c4cb393dc35740b28c0)
    assert_eq!(
        bytes_to_hex_str(&key.to_public_key().unwrap().as_bytes()),
        "13978aee95f38490e9769c39b2773ed763d9cd5f"
    );
}

#[test]
#[should_panic]
fn zero_address() {
    // A key full of zeros is an invalid private key.
    let key = PrivateKey::new();
    key.to_public_key().unwrap();
}
