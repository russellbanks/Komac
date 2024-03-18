use color_eyre::eyre::{eyre, OptionExt};
use color_eyre::Result;
use crc32fast::Hasher;
use object::pe::RT_RCDATA;
use object::read::pe::{ImageNtHeaders, PeFile, ResourceDirectoryEntryData};
use object::{LittleEndian, ReadRef, U16, U32};
use std::io::Read;
use versions::SemVer;
use xz2::read::XzDecoder;
use xz2::stream::{LzmaOptions, Stream};

use crate::exe::inno::loader::{SetupLoader, SETUP_LOADER_RESOURCE};
use crate::exe::inno::version::{InnoVersion, KnownVersion, KNOWN_VERSIONS};

const VERSION_LEN: u64 = 1 << 6;

pub struct InnoFile<'data> {
    version: KnownVersion,
    setup_loader: SetupLoader<'data>,
}

impl<'data> InnoFile<'data> {
    pub fn new<Pe, R>(pe: &PeFile<'data, Pe, R>) -> Result<Self>
    where
        Pe: ImageNtHeaders,
        R: ReadRef<'data>,
    {
        let data = pe.data();
        let resource_directory = pe
            .data_directories()
            .resource_directory(data, &pe.section_table())?
            .ok_or_eyre("No resource directory")?;
        let rc_data = resource_directory
            .root()?
            .entries
            .iter()
            .find(|entry| entry.name_or_id().id() == Some(RT_RCDATA))
            .ok_or_eyre("No RT_RCDATA was found")?;
        let setup_loader = rc_data
            .data(resource_directory)?
            .table()
            .and_then(|table| {
                table
                    .entries
                    .iter()
                    .find(|entry| entry.name_or_id().id() == Some(SETUP_LOADER_RESOURCE))
            })
            .ok_or_eyre("No Setup loader resource was found")?;
        let entry = setup_loader
            .data(resource_directory)?
            .table()
            .and_then(|table| table.entries.first())
            .and_then(|entry| entry.data(resource_directory).ok())
            .and_then(ResourceDirectoryEntryData::data)
            .unwrap();
        let section = pe
            .section_table()
            .section_containing(entry.offset_to_data.get(LittleEndian))
            .unwrap();

        // Translate the offset into a usable one
        let mut offset = {
            let mut rva = entry.offset_to_data.get(LittleEndian);
            rva -= section.virtual_address.get(LittleEndian);
            rva += section.pointer_to_raw_data.get(LittleEndian);
            rva as u64
        };

        let setup_loader = SetupLoader::new(data, &mut offset)?;

        let header_offset = *setup_loader.header_offset as u64;
        let version_bytes = data
            .read_bytes_at_until(header_offset..header_offset + VERSION_LEN, u8::default())
            .unwrap();
        offset = header_offset + VERSION_LEN;
        let version = String::from_utf8_lossy(version_bytes);
        println!("{}", version);

        let known_version = KNOWN_VERSIONS
            .into_iter()
            .rfind(|know_version| know_version.name == version)
            .ok_or_else(|| eyre!("Unknown Inno Setup Version: {version}"))?;

        let expected_checksum = data
            .read::<U32<LittleEndian>>(&mut offset)
            .unwrap()
            .get(LittleEndian);

        let mut actual_checksum = Hasher::new();

        let mut compression = CompressionType::Stored;
        let mut stored_size = 0;
        if known_version.version > InnoVersion(4, 0, 9, 0) {
            stored_size = data
                .read::<U32<LittleEndian>>(&mut offset)
                .unwrap()
                .get(LittleEndian);
            actual_checksum.update(&stored_size.to_le_bytes());
            let compressed = data.read::<u8>(&mut offset).unwrap();
            actual_checksum.update(&compressed.to_le_bytes());
            compression = if compressed != &0 {
                if known_version.version > InnoVersion(4, 1, 6, 0) {
                    CompressionType::LZMA1
                } else {
                    CompressionType::Zlib
                }
            } else {
                CompressionType::Stored
            };
            println!("{}", stored_size);
            println!("{}", compressed);
            println!("{:?}", compression);
        }

        assert_eq!(expected_checksum, actual_checksum.finalize());

        if compression == CompressionType::LZMA1 {
            let properties = data.read::<u8>(&mut offset).unwrap();
            println!("{}", properties);
            assert!(properties < &(9 * 5 * 5));
            let pb = properties / (9 * 5);
            let lp = (properties % (9 * 5)) / 9;
            println!("pb {pb}");
            println!("lp: {lp}");
            let lc = properties % 9;
            println!("lc: {lc}");
            let dictionary_size = data
                .read::<U32<LittleEndian>>(&mut offset)
                .unwrap()
                .get(LittleEndian);
            println!("{}", dictionary_size);
            let mut lzma_options = LzmaOptions::new_preset(5).unwrap();
            // This doesn't support the right amount of lc so need a different library or workaround
            lzma_options.literal_context_bits(4);
            lzma_options.position_bits(pb as u32);
            lzma_options.literal_position_bits(lp as u32);
            let stream = Stream::new_lzma_encoder(&lzma_options).unwrap();
            let mut decompressor = XzDecoder::new_stream(
                data.read_slice_at(offset, stored_size as usize).unwrap(),
                stream,
            );
            let mut vec = [0; 4];
            decompressor.read_exact(&mut vec).unwrap();
            println!("{:?}", vec);
        }

        Ok(InnoFile {
            version: known_version,
            setup_loader,
        })
    }
}

#[derive(Debug, PartialEq)]
enum CompressionType {
    Stored,
    Zlib,
    LZMA1,
}
