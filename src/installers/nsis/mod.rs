mod block;
mod first_header;
mod header;
mod strings;
mod version;

use crate::installers::nsis::block::BlockType;
use crate::installers::nsis::first_header::FirstHeader;
use crate::installers::nsis::header::Header;
use crate::installers::nsis::version::NsisVersion;
use crate::installers::utils::RELATIVE_PROGRAM_FILES_64;
use crate::types::architecture::Architecture;
use byteorder::{ByteOrder, LE};
use camino::Utf8PathBuf;
use color_eyre::eyre::OptionExt;
use color_eyre::Result;
use strings::encoding::nsis_string;
use yara_x::mods::PE;
use zerocopy::FromBytes;

pub struct Nsis {
    pub architecture: Architecture,
    pub install_dir: Utf8PathBuf,
}

impl Nsis {
    pub fn new(data: &[u8], pe: &PE) -> Result<Self> {
        // The first header is positioned after the PE data
        let first_header_offset = pe
            .sections
            .iter()
            .max_by_key(|section| section.raw_data_offset())
            .map(|section| section.raw_data_offset() as usize + section.raw_data_size() as usize)
            .map(|offset| offset.next_multiple_of(FirstHeader::ALIGNMENT as usize))
            .ok_or_eyre("Unable to get NSIS first header offset")?;

        let data_offset = first_header_offset + size_of::<FirstHeader>();
        let first_header = FirstHeader::read(&data[first_header_offset..data_offset])?;

        let decompressed_data = Header::decompress(&data[data_offset..], first_header)?;
        let header = Header::ref_from(&decompressed_data[..size_of::<Header>()]).unwrap();

        let strings_block = BlockType::Strings.get(&decompressed_data, &header.blocks);
        let unicode = LE::read_u16(&strings_block) == 0;

        let names = get_name_offsets(&decompressed_data, header);

        let branding_text = nsis_string()
            .strings_block(strings_block)
            .relative_offset(names[0])
            .unicode(unicode)
            .get()?;

        let nsis_version = NsisVersion::from_branding_text(&branding_text).unwrap_or_default();

        let install_dir = nsis_string()
            .strings_block(strings_block)
            .relative_offset(header.install_directory_ptr.get())
            .nsis_version(nsis_version)
            .unicode(unicode)
            .get()?;

        let architecture = if install_dir.contains(RELATIVE_PROGRAM_FILES_64) {
            Architecture::X64
        } else {
            Architecture::X86
        };

        Ok(Self {
            architecture,
            install_dir: Utf8PathBuf::from(install_dir),
        })
    }
}

fn get_name_offsets(data: &[u8], header: &Header) -> [u32; 3] {
    let mut names: [u32; 3] = [0; 3];

    let strings_count = (header.langtable_size.get() - 10) as usize / size_of::<u32>();
    let lang_table_block_header = &header.blocks[BlockType::LangTables as usize];

    for index in 0..lang_table_block_header.num.get() {
        let offset = lang_table_block_header.offset.get() as usize
            + (header.langtable_size.get() * index) as usize;
        let lang_table = &data[offset..offset + header.langtable_size.get() as usize];
        let lang_id = LE::read_u16(lang_table);

        for name_index in 0..names.len().min(strings_count) {
            let offset = 10 + name_index * size_of::<u32>();
            let name_offset = LE::read_u32(&lang_table[offset..]);
            if name_offset != 0 && (lang_id == 1033 || names[name_index] == 0) {
                names[name_index] = name_offset;
            }
        }
    }

    names
}
