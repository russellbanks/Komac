use std::{io, io::Read};

use zerocopy::{Immutable, IntoBytes, KnownLayout, LittleEndian, TryFromBytes, U16, U32};

use super::signature::DosSignature;

/// In `winnt.h` and `pe.h`, it's `IMAGE_DOS_HEADER`. It's a DOS header present in all PE binaries.
///
/// The DOS header is a relic from the MS-DOS era. It used to be useful to display an error message
/// if the binary is run in MS-DOS by utilizing the DOS stub.
///
/// Nowadays, only two fields from the DOS header are used on Windows:
/// [`signature` (aka `e_magic`)](DosHeader::signature) and
/// [`pe_pointer` (aka `e_lfanew`)](DosHeader::pe_pointer).
///
/// ## Position in a modern PE file
///
/// The DOS header is located at the beginning of the PE file and is usually followed by the DOS
/// stub.
///
/// ## Note on the archaic "formatted header"
///
/// The subset of the structure spanning from its start to the `overlay_number` (aka `e_ovno`) field
/// included (i.e. till the offset 0x1C) used to be commonly known as "formatted header", since
/// their position and contents were fixed. Optional information used by overlay managers could have
/// followed the formatted header. In the absence of optional information, the formatted header was
/// followed by the [`relocation pointer table`](https://www.tavi.co.uk/phobos/exeformat.html#reloctable).
///
/// Overlays were sections of a program that remained on disk until the program actually required
/// them. Different overlays could thus share the same memory area. The overlays were loaded and
/// unloaded by special code provided by the program or its run-time library.
///
/// [Source](https://www.tavi.co.uk/phobos/exeformat.html#:~:text=Format%20of%20the%20.EXE%20file%20header).
#[doc(alias("IMAGE_DOS_HEADER"))]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Immutable, IntoBytes, KnownLayout, TryFromBytes)]
#[repr(C)]
pub struct DosHeader {
    /// Magic number: `[0x5A, 0x4D]`. In [little endian](https://en.wikipedia.org/wiki/Endianness)
    /// [ASCII](https://en.wikipedia.org/wiki/ASCII), it reads "MZ" for [Mark Zbikowski](https://en.wikipedia.org/wiki/Mark_Zbikowski)).
    ///
    /// ## Non-MZ DOS executables
    ///
    /// * For [IBM OS/2](https://www.britannica.com/technology/IBM-OS-2), the value was "NE".
    /// * For IBM OS/2 LE, the value was "LE".
    /// * For [NT](https://en.wikipedia.org/wiki/Windows_NT), the value was "PE00".
    ///
    /// Sources:
    ///
    /// * <https://stixproject.github.io/data-model/1.2/WinExecutableFileObj/DOSHeaderType/>
    /// * <https://learn.microsoft.com/archive/msdn-magazine/2002/february/inside-windows-win32-portable-executable-file-format-in-detail>
    #[doc(alias("e_magic"))]
    signature: DosSignature,

    /// See
    #[doc(alias("e_cblp"))]
    bytes_on_last_page: U16<LittleEndian>,

    #[doc(alias("e_cp"))]
    pages_in_file: U16<LittleEndian>,

    #[doc(alias("e_crlc"))]
    relocations: U16<LittleEndian>,

    #[doc(alias("e_cparhdr"))]
    size_of_header_in_paragraphs: U16<LittleEndian>,

    #[doc(alias("e_minalloc"))]
    minimum_extra_paragraphs_needed: U16<LittleEndian>,

    #[doc(alias("e_maxalloc"))]
    maximum_extra_paragraphs_needed: U16<LittleEndian>,

    #[doc(alias("e_ss"))]
    initial_relative_ss: U16<LittleEndian>,

    #[doc(alias("e_sp"))]
    initial_sp: U16<LittleEndian>,

    #[doc(alias("e_csum"))]
    checksum: U16<LittleEndian>,

    #[doc(alias("e_ip"))]
    initial_ip: U16<LittleEndian>,

    #[doc(alias("e_cs"))]
    initial_relative_cs: U16<LittleEndian>,

    #[doc(alias("e_lfarlc"))]
    file_address_of_relocation_table: U16<LittleEndian>,

    #[doc(alias("e_ovno"))]
    overlay_number: U16<LittleEndian>,

    /// In `winnt.h` and `pe.h`, it's `e_res[4]`.
    ///
    /// It used to specify the reserved words for the program, i.e. an array reserved for future use.
    /// Usually, the array was zeroed by the linker.
    /// [Source](https://stixproject.github.io/data-model/1.2/WinExecutableFileObj/DOSHeaderType/).
    ///
    /// Typically, this field is set to 0. [Source](https://offwhitesecurity.dev/malware-development/portable-executable-pe/dos-header/).
    #[doc(alias("e_res"))]
    reserved: [U16<LittleEndian>; 4],

    #[doc(alias("e_oemid"))]
    oem_id: U16<LittleEndian>,

    #[doc(alias("e_oeminfo"))]
    oem_info: U16<LittleEndian>,
    /// In `winnt.h` and `pe.h`, it's `e_res2[10]`.
    ///
    /// It used to specify the reserved words for the program, i.e. an array reserved for future use.
    /// Usually, the array was zeroed by the linker.
    /// [Source](https://stixproject.github.io/data-model/1.2/WinExecutableFileObj/DOSHeaderType/).
    ///
    /// Typically, this field is set to 0. [Source](https://offwhitesecurity.dev/malware-development/portable-executable-pe/dos-header/).
    #[doc(alias("e_res2"))]
    reserved2: [U16<LittleEndian>; 10],

    #[doc(alias("e_lfanew"))]
    pe_pointer: U32<LittleEndian>,
}

impl DosHeader {
    pub fn try_read_from_io<R>(mut src: R) -> io::Result<Self>
    where
        Self: Sized,
        R: Read,
    {
        let mut buf = [0; size_of::<Self>()];
        src.read_exact(&mut buf)?;
        Self::try_read_from_bytes(&buf)
            .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err.to_string()))
    }

    /// In `winnt.h` and `pe.h`, it's `e_cblp`.
    ///
    /// It used to specify the number of bytes actually used in the last "page".
    /// Page used to refer to a segment of memory, usually of 512 bytes size.
    ///
    /// The case of full page was represented by 0x0000 (since the last page is never empty).
    ///
    /// For example, assuming a page size of 512 bytes, this value would
    /// be 0x0000 for a 1024 byte file, and 0x0001 for a 1025 byte file
    /// (since it only contains one valid byte).
    /// [Source](https://stixproject.github.io/data-model/1.2/WinExecutableFileObj/DOSHeaderType/).
    ///
    /// Typically, this field is set to 0. [Source](https://offwhitesecurity.dev/malware-development/portable-executable-pe/dos-header/).
    #[inline]
    pub const fn bytes_on_last_page(&self) -> u16 {
        self.bytes_on_last_page.get()
    }

    /// In `winnt.h` and `pe.h`, it's `e_cp`.
    ///
    /// It used to specify the number of pages required to hold a file. For example,
    /// if the file contained 1024 bytes, and the file had pages of a size of 512 bytes,
    /// this [word](https://en.wikipedia.org/wiki/Word_(computer_architecture)) would contain
    /// 0x0002 (2 pages); if the file contained 1025 bytes, this word would contain 0x0003 (3 pages).
    /// [Source](https://stixproject.github.io/data-model/1.2/WinExecutableFileObj/DOSHeaderType/).
    ///
    /// Typically, this field is set to 0. [Source](https://offwhitesecurity.dev/malware-development/portable-executable-pe/dos-header/).
    #[inline]
    pub const fn pages_in_file(&self) -> u16 {
        self.pages_in_file.get()
    }

    /// In `winnt.h` and `pe.h`, it's `e_crlc`.
    ///
    /// It used to specify the number of "relocation items", i.e. the number of entries that
    /// existed in the [`relocation pointer table`](https://www.tavi.co.uk/phobos/exeformat.html#reloctable).
    /// If there were no relocations, this field would contain 0x0000.
    /// [Source](https://stixproject.github.io/data-model/1.2/WinExecutableFileObj/DOSHeaderType/).
    ///
    /// ## On relocation items and relocation pointer table
    ///
    /// When a program is compiled, memory addresses are often hard-coded into the binary code.
    /// These addresses are usually relative to the base address where the program expects to be
    /// loaded into memory. However, when the program is loaded into memory, it might not be loaded
    /// at its preferred base address due to various reasons such as memory fragmentation or other
    /// programs already occupying that space.
    ///
    /// Relocation items, also known as fixups or relocations, are pieces of data embedded within
    /// the executable file that indicate which memory addresses need to be adjusted when the
    /// program is loaded at a different base address. These relocations specify the location and
    /// type of adjustment needed.
    ///
    /// The relocation pointer table is a data structure that contains pointers to the locations
    /// within the executable file where relocations need to be applied. It allows the operating
    /// system's loader to efficiently locate and process the relocation data during the loading
    /// process.
    ///
    /// ---
    ///
    /// Typically, this field is set to 0. [Source](https://offwhitesecurity.dev/malware-development/portable-executable-pe/dos-header/).
    #[inline]
    pub const fn relocations(&self) -> u16 {
        self.relocations.get()
    }

    /// In `winnt.h` and `pe.h`, it's `e_cparhdr`.
    ///
    /// It used to specify the size of the "executable header" in terms of "paragraphs"
    /// (16 byte chunks). It used to indicate the offset of the program's compiled/assembled and
    /// linked image (the [load module](https://www.tavi.co.uk/phobos/exeformat.html#loadmodule))
    /// within the executable file. The size of the load module could have been deduced by
    /// substructing this value (converted to bytes) from the overall size that could have been
    /// derived from combining the value of `pages_in_file` (aka `e_cp`) and the value of
    /// `bytes_on_last_page` (aka `e_cblp)`. The header used to always span an even number of
    /// paragraphs. [Source](https://stixproject.github.io/data-model/1.2/WinExecutableFileObj/DOSHeaderType/).
    ///
    /// The "executable header" in this context refers to the DOS header itself.
    ///
    /// Typically, this field is set to 4. [Source](https://offwhitesecurity.dev/malware-development/portable-executable-pe/dos-header/).
    /// This is because the modern DOS header is 64 bytes long, and 64 / 16 = 4.
    #[inline]
    pub const fn size_of_header_in_paragraphs(&self) -> u16 {
        self.size_of_header_in_paragraphs.get()
    }

    /// In `winnt.h` and `pe.h`, it's `e_minalloc`.
    ///
    /// It used to specify the minimum number of extra paragraphs needed to be allocated to begin
    /// execution. This is **in addition** to the memory required to hold the [load module](https://www.tavi.co.uk/phobos/exeformat.html#loadmodule).
    /// This value normally represented the total size of any uninitialized data and/or stack
    /// segments that were linked at the end of the program. This space was not directly included in
    /// the load module, since there were no particular initializing values, and it would simply
    /// waste disk space. [Source](https://stixproject.github.io/data-model/1.2/WinExecutableFileObj/DOSHeaderType/).
    ///
    /// If both the `minimum_extra_paragraphs_needed` (aka `e_minalloc`) and
    /// `maximum_extra_paragraphs_needed` (aka `e_maxalloc`) fields were set to 0x0000, the program
    /// would be allocated as much memory as available. [Source](https://www.tavi.co.uk/phobos/exeformat.html)
    ///
    /// Typically, this field is set to 0x10. [Source](https://offwhitesecurity.dev/malware-development/portable-executable-pe/dos-header/).
    #[inline]
    pub const fn minimum_extra_paragraphs_needed(&self) -> u16 {
        self.minimum_extra_paragraphs_needed.get()
    }

    /// In `winnt.h` and `pe.h`, it's `e_maxalloc`.
    ///
    /// It used to specify the maximum number of extra paragraphs needed to be allocated by to begin
    /// execution. This indicated **additional** memory over and above that required by the
    /// [load module](https://www.tavi.co.uk/phobos/exeformat.html#loadmodule) and the value
    /// specified in `minimum_extra_paragraphs_needed` (aka `e_minalloc`). If the request could not
    /// be satisfied, the program would be allocated as much memory as available.
    /// [Source](https://stixproject.github.io/data-model/1.2/WinExecutableFileObj/DOSHeaderType/).
    ///
    /// If both the `minimum_extra_paragraphs_needed` (aka `e_minalloc`) and
    /// `maximum_extra_paragraphs_needed` (aka `e_maxalloc`) fields were set to 0x0000, the program
    /// would be allocated as much memory as available. [Source](https://www.tavi.co.uk/phobos/exeformat.html)
    ///
    /// Typically, this field is set to 0xFFFF. [Source](https://offwhitesecurity.dev/malware-development/portable-executable-pe/dos-header/).
    #[inline]
    pub const fn maximum_extra_paragraphs_needed(&self) -> u16 {
        self.maximum_extra_paragraphs_needed.get()
    }

    /// In `winnt.h` and `pe.h`, it's `e_ss`.
    ///
    /// It used to specify the initial SS ("stack segment") value. SS value was a paragraph address
    /// of the stack segment relative to the start of the [load module](https://www.tavi.co.uk/phobos/exeformat.html#loadmodule).
    /// At load time, the value was relocated by adding the address of the start segment of the
    /// program to it, and the resulting value was placed in the SS register before the program is
    /// started. To read more about x86 memory segmentation and SS register, see the
    /// [wikipedia article](https://en.wikipedia.org/wiki/X86_memory_segmentation) on this topic. In
    /// DOS, the start segment boundary of the program was the first segment boundary in memory
    /// after [Program Segment Prefix (PSP)](https://en.wikipedia.org/wiki/Program_Segment_Prefix).
    /// [Source](https://stixproject.github.io/data-model/1.2/WinExecutableFileObj/DOSHeaderType/).
    ///
    /// The Program Segment Prefix (PSP) was a data structure used in DOS (Disk Operating System)
    /// environments. It was located at the beginning of the memory allocated for a running program,
    /// and it contained various pieces of information about the program, including command-line
    /// arguments, environment variables, and pointers to various system resources.
    ///
    /// [According to Wikipedia](https://en.wikipedia.org/wiki/Data_segment#Stack), the stack
    /// segment contains the call stack, a LIFO structure, typically located in the higher parts of
    /// memory. A "stack pointer" register tracks the top of the stack; it is adjusted each time a
    /// value is "pushed" onto the stack. The set of values pushed for one function call is termed a
    /// "stack frame".
    ///
    /// Typically, this field is set to 0. [Source](https://offwhitesecurity.dev/malware-development/portable-executable-pe/dos-header/).
    #[inline]
    pub const fn initial_relative_ss(&self) -> u16 {
        self.initial_relative_ss.get()
    }

    /// In `winnt.h` and `pe.h`, it's `e_sp`.
    ///
    /// It used to specify the initial SP ("stack pointer") value. SP value was the absolute value
    /// that must have been loaded into the SP register before the program is given control. Since
    /// the actual stack segment was determined by the loader, and this was merely a value within
    /// that segment, it didn't need to be relocated.
    ///
    /// [According to Wikipedia](https://en.wikipedia.org/wiki/Data_segment#Stack), the stack
    /// segment contains the call stack, a LIFO structure, typically located in the higher parts of
    /// memory. A "stack pointer" register tracks the top of the stack; it is adjusted each time a
    /// value is "pushed" onto the stack. The set of values pushed for one function call is termed a
    /// "stack frame".
    /// [Source](https://stixproject.github.io/data-model/1.2/WinExecutableFileObj/DOSHeaderType/).
    ///
    /// Typically, this field is set to 0xB8. [Source](https://offwhitesecurity.dev/malware-development/portable-executable-pe/dos-header/).
    #[inline]
    pub const fn initial_sp(&self) -> u16 {
        self.initial_sp.get()
    }

    /// In `winnt.h` and `pe.h`, it's `e_csum`.
    ///
    /// It used to specify the checksum of the contents of the executable file It used to ensure the
    /// integrity of the data within the file. For full details on how this checksum was calculated,
    /// see <http://www.tavi.co.uk/phobos/exeformat.html#checksum>.
    /// [Source](https://stixproject.github.io/data-model/1.2/WinExecutableFileObj/DOSHeaderType/).
    ///
    /// Typically, this field is set to 0. [Source](https://offwhitesecurity.dev/malware-development/portable-executable-pe/dos-header/).
    #[inline]
    pub const fn checksum(&self) -> u16 {
        self.checksum.get()
    }

    /// In `winnt.h` and `pe.h`, it's `e_ip`.
    ///
    /// It used to specify the initial IP ("instruction pointer") value. IP value was the absolute
    /// value that must have been loaded into the IP register in order to transfer control to the
    /// program. Since the actual code segment was determined by the loader and, and this was merely
    /// a value within that segment, it didn't need to be relocated.
    /// [Source](https://stixproject.github.io/data-model/1.2/WinExecutableFileObj/DOSHeaderType/).
    ///
    /// Typically, this field is set to 0. [Source](https://offwhitesecurity.dev/malware-development/portable-executable-pe/dos-header/).
    #[inline]
    pub const fn initial_ip(&self) -> u16 {
        self.initial_ip.get()
    }

    /// In `winnt.h` and `pe.h`, it's `e_cs`.
    ///
    /// It used to specify the pre-relocated initial CS ("code segment") value relative to the start
    /// of the [load module](https://www.tavi.co.uk/phobos/exeformat.html#loadmodule), that should
    /// have been placed in the CS register in order to transfer control to the program. At load
    /// time, this value was relocated by adding the address of the start segment of the program to
    /// it, and the resulting value was placed in the CS register when control is transferred.
    /// [Source](https://stixproject.github.io/data-model/1.2/WinExecutableFileObj/DOSHeaderType/).
    ///
    /// Typically, this field is set to 0. [Source](https://offwhitesecurity.dev/malware-development/portable-executable-pe/dos-header/).
    #[inline]
    pub const fn initial_relative_cs(&self) -> u16 {
        self.initial_relative_cs.get()
    }

    /// In `winnt.h` and `pe.h`, it's `e_lfarlc`.
    ///
    /// It used to specify the logical file address of the relocation table, or more specifically,
    /// the offset from the start of the file to the [relocation pointer table](https://www.tavi.co.uk/phobos/exeformat.html#reloctable).
    /// This value must have been used to locate the relocation table (rather than assuming a fixed
    /// location) because variable-length information pertaining to program overlays could have
    /// occurred before this table, causing its position to vary. A value of 0x40 in this field
    /// generally indicated a different kind of executable, not a DOS 'MZ' type.
    /// [Source](https://stixproject.github.io/data-model/1.2/WinExecutableFileObj/DOSHeaderType/).
    ///
    /// Typically, this field is set to 0x40. [Source](https://offwhitesecurity.dev/malware-development/portable-executable-pe/dos-header/).
    #[inline]
    pub const fn file_address_of_relocation_table(&self) -> u16 {
        self.file_address_of_relocation_table.get()
    }

    /// In `winnt.h` and `pe.h`, it's `e_ovno`.
    ///
    /// It used to specify the overlay number, which was normally set to 0x0000, because few
    /// programs actually had overlays. It changed only in files containing programs that used
    /// overlays. [Source](https://stixproject.github.io/data-model/1.2/WinExecutableFileObj/DOSHeaderType/).
    ///
    /// Overlays were sections of a program that remained on disk until the program actually
    /// required them. Different overlays could thus share the same memory area. The overlays were
    /// loaded and unloaded by special code provided by the program or its run-time library.
    ///
    /// Typically, this field is set to 0. [Source](https://offwhitesecurity.dev/malware-development/portable-executable-pe/dos-header/).
    #[inline]
    pub const fn overlay_number(&self) -> u16 {
        self.overlay_number.get()
    }

    /// In `winnt.h` and `pe.h`, it's `e_oemid`.
    ///
    /// It used to specify the identifier for the OEM ("Original Equipment Manufacturer") for
    /// `oem_info` aka `e_oeminfo`.
    /// [Source](https://stixproject.github.io/data-model/1.2/WinExecutableFileObj/DOSHeaderType/).
    ///
    /// More specifically, it used to specify the OEM of the system or hardware platform for which
    /// the executable file was created. This field was used to specify certain characteristics or
    /// requirements related to the hardware environment in which the executable was intended to
    /// run.
    ///
    /// Typically, this field is set to 0. [Source](https://offwhitesecurity.dev/malware-development/portable-executable-pe/dos-header/).
    #[inline]
    pub const fn oem_id(&self) -> u16 {
        self.oem_id.get()
    }

    /// In `winnt.h` and `pe.h`, it's `e_oeminfo`.
    ///
    /// It used to specify the extra information, the kind of which was specific to the OEM
    /// identified by `oem_id` aka `e_oemid`.
    #[inline]
    pub const fn oem_info(&self) -> u16 {
        self.oem_info.get()
    }

    /// In `winnt.h` and `pe.h`, it's `e_lfanew`.
    ///
    /// Today, it specifies the logical file address of the new exe header. In particular, it is a
    /// 4-byte offset into the file where the PE file header is located. It is necessary to use this
    /// offset to locate the PE header in the file.
    ///
    /// Typically, this field is set to 0x3c.
    #[inline]
    pub const fn pe_pointer(&self) -> u32 {
        self.pe_pointer.get()
    }
}
