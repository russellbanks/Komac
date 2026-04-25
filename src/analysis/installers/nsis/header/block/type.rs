use strum::{EnumCount, EnumIter};

#[derive(Copy, Clone, EnumCount, EnumIter)]
pub enum BlockType {
    Pages,
    Sections,
    Entries,
    Strings,
    LangTables,
    CtlColors,
    BgFont,
    Data,
}

impl BlockType {
    /// Returns the block type as a static string.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Pages => "Pages",
            Self::Sections => "Sections",
            Self::Entries => "Entries",
            Self::Strings => "Strings",
            Self::LangTables => "LangTables",
            Self::CtlColors => "CtlColors",
            Self::BgFont => "BgFont",
            Self::Data => "Data",
        }
    }
}
