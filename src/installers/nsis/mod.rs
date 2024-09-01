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
use camino::Utf8PathBuf;
use color_eyre::Result;
use std::io::Cursor;
use strings::encoding::nsis_string;
use zerocopy::FromBytes;

pub struct Nsis {
    pub architecture: Architecture,
    pub install_dir: Utf8PathBuf,
}

impl Nsis {
    pub fn new(data: &[u8]) -> Result<Self> {
        let mut reader = Cursor::new(data);
        let first_header = FirstHeader::read(&mut reader)?;

        let d_data = Header::decompress(&mut reader, &first_header)?;
        let header = Header::ref_from(&d_data[..size_of::<Header>()]).unwrap();

        let strings_block = BlockType::Strings.get(&d_data, &header.blocks);
        let unicode = u16::from_le_bytes((&strings_block[..size_of::<u16>()]).try_into()?) == 0;

        let names = get_name_offsets(&d_data, header)?;

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

fn get_name_offsets(data: &[u8], header: &Header) -> Result<[u32; 3]> {
    let mut names: [u32; 3] = [0; 3];

    let strings_count = (header.langtable_size.get() - 10) as usize / size_of::<u32>();
    let lang_table_block_header = &header.blocks[BlockType::LangTables as usize];
    for index in 0..lang_table_block_header.num.get() {
        let offset = lang_table_block_header.offset.get() as usize
            + (header.langtable_size.get() * index) as usize;
        let lang_table = &data[offset..offset + header.langtable_size.get() as usize];
        let lang_id = u16::from_le_bytes((&lang_table[..size_of::<u16>()]).try_into()?);

        for name_index in 0..names.len().min(strings_count) {
            let offset = 10 + name_index * size_of::<u32>();
            let name_offset =
                u32::from_le_bytes((&lang_table[offset..offset + size_of::<u32>()]).try_into()?);
            if name_offset != 0 && (lang_id == 1033 || names[name_index] == 0) {
                names[name_index] = name_offset;
            }
        }
    }

    Ok(names)
}
