use std::{
    io,
    io::{Error, ErrorKind, Read},
    str::FromStr,
};

use derive_more::Display;
use heapless::String;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

// 256 bits / 4 bits per hex character
const SHA256_DIGEST_LEN: usize = 256 / 0xFu8.count_ones() as usize;

#[derive(Clone, Debug, Display, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub struct Sha256String(String<SHA256_DIGEST_LEN>);

impl Sha256String {
    pub fn from_hasher(data: &[u8]) -> Result<Self, base16ct::Error> {
        let mut encode_buf = [0; SHA256_DIGEST_LEN];
        let sha_256 = base16ct::upper::encode_str(data, &mut encode_buf)?;
        Ok(Self(
            String::<SHA256_DIGEST_LEN>::from_str(sha_256).unwrap_or_else(|()| unreachable!()),
        ))
    }

    pub fn from_reader<R: Read>(mut reader: R) -> io::Result<Self> {
        let mut hasher = Sha256::new();
        let mut buffer = [0; 1 << 12];

        loop {
            let count = reader.read(&mut buffer)?;
            if count == 0 {
                break;
            }
            hasher.update(&buffer[..count]);
        }

        Self::from_hasher(&hasher.finalize()).map_err(|err| Error::new(ErrorKind::Other, err))
    }
}

impl Default for Sha256String {
    fn default() -> Self {
        Self(std::iter::repeat_n('0', SHA256_DIGEST_LEN).collect::<_>())
    }
}
