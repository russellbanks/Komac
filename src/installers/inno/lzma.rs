use byteorder::{LittleEndian, ReadBytesExt};
use color_eyre::eyre::{bail, Error};
use color_eyre::Result;
use liblzma::stream::{Filters, LzmaOptions, Stream};
use std::io::Read;

const PROPERTIES_MAX: u8 = 9 * 5 * 5;

const LC_LP_MAX: u8 = 4;

// The LZMA1 streams used by Inno Setup differ slightly from the LZMA Alone file format:
// The stream header only stores the properties (lc, lp, pb) and the dictionary size and
// is missing the uncompressed size field.
pub fn read_inno_lzma_stream_header<R: Read>(reader: &mut R) -> Result<Stream> {
    let properties = reader.read_u8()?;
    if properties >= PROPERTIES_MAX {
        bail!("LZMA properties value must be less than {PROPERTIES_MAX} but was {properties}",)
    }

    let lc = u32::from(properties % 9);
    let lp = u32::from((properties % (9 * 5)) / 9);
    if lc + lp > u32::from(LC_LP_MAX) {
        bail!(
            "LZMA lc + lp must not be greater than {LC_LP_MAX} but was {}",
            lc + lp
        )
    }
    let pb = u32::from(properties / (9 * 5));

    let dictionary_size = reader.read_u32::<LittleEndian>()?;

    let mut lzma_options = LzmaOptions::new();
    lzma_options
        .literal_context_bits(lc)
        .literal_position_bits(lp)
        .position_bits(pb)
        .dict_size(dictionary_size);

    let mut filters = Filters::new();
    filters.lzma1(&lzma_options);

    Stream::new_raw_decoder(&filters).map_err(Error::msg)
}
