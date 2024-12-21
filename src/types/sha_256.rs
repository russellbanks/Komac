use std::io::Read;
use std::str::FromStr;

use color_eyre::eyre::eyre;
use color_eyre::Result;
use heapless::String;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

// 256 bits / 4 bits per hex character
const SHA256_DIGEST_LEN: usize = 256 / 0xFu8.count_ones() as usize;

#[derive(Clone, Debug, Default, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct Sha256String(String<SHA256_DIGEST_LEN>);

impl Sha256String {
    pub fn from_hasher(data: &[u8]) -> Result<Self> {
        let mut encode_buf = [0; SHA256_DIGEST_LEN];
        let sha_256 = base16ct::upper::encode_str(data, &mut encode_buf)?;
        Ok(Self(
            String::<SHA256_DIGEST_LEN>::from_str(sha_256)
                .map_err(|()| eyre!("SHA256 必须 {SHA256_DIGEST_LEN} 字节 (bytes) 长"))?,
        ))
    }

    pub fn from_reader<R: Read>(mut reader: R) -> Result<Self> {
        let mut hasher = Sha256::new();
        let mut buffer = [0; 1 << 12];

        loop {
            let count = reader.read(&mut buffer)?;
            if count == 0 {
                break;
            }
            hasher.update(&buffer[..count]);
        }

        Self::from_hasher(&hasher.finalize())
    }
}
