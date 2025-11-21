use std::fmt;

/// Enumerated resource types.
///
/// See <https://learn.microsoft.com/windows/win32/menurc/resource-types>
#[derive(Copy, Clone, Eq, PartialEq)]
pub enum ResourceType {
    /// Hardware-dependent cursor resource.
    Cursor = 1,

    /// Bitmap resource.
    Bitmap = 2,

    /// Hardware-dependent icon resource.
    Icon = 3,

    /// Menu resource.
    Menu = 4,

    /// Dialog box.
    Dialog = 5,

    /// String-table entry.
    String = 6,

    /// Font directory resource.
    FontDirectory = 7,

    /// Font resource.
    Font = 8,

    /// Accelerator table.
    Accelerator = 9,

    /// Application-defined resource (raw data).
    RCData = 10,

    /// Message-table entry.
    MessageTable = 11,

    /// Hardware-independent cursor resource.
    GroupCursor = 12,

    /// Hardware-independent icon resource.
    GroupIcon = 14,

    /// Version resource.
    Version = 16,

    /// Allows a resource editing tool to associate a string with an .rc file. Typically, the string
    /// is the name of the header file that provides symbolic names. The resource compiler parses
    /// the string but otherwise ignores the value. For example, `1 DLGINCLUDE "MyFile.h"`.
    DialogInclude = 17,

    /// Plug and Play resource.
    PlugPlay = 19,

    /// VXD.
    Vxd = 20,

    /// Animated cursor.
    AnimatedCursor = 21,

    /// Animated icon.
    AnimatedIcon = 22,

    /// HTML resource.
    Html = 23,

    /// Side-by-Side Assembly Manifest.
    Manifest = 24,
}

impl ResourceType {
    /// Returns the resource type's ID.
    #[must_use]
    #[inline]
    pub const fn id(self) -> u32 {
        self as u32
    }
}

impl fmt::Debug for ResourceType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self}({})", self.id())
    }
}

impl fmt::Display for ResourceType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Cursor => f.write_str("RT_CURSOR"),
            Self::Bitmap => f.write_str("RT_BITMAP"),
            Self::Icon => f.write_str("RT_ICON"),
            Self::Menu => f.write_str("RT_MENU"),
            Self::Dialog => f.write_str("RT_DIALOG"),
            Self::String => f.write_str("RT_STRING"),
            Self::FontDirectory => f.write_str("RT_FONTDIR"),
            Self::Font => f.write_str("RT_FONT"),
            Self::Accelerator => f.write_str("RT_ACCELERATOR"),
            Self::RCData => f.write_str("RT_RCDATA"),
            Self::MessageTable => f.write_str("RT_MESSAGETABLE"),
            Self::GroupCursor => f.write_str("RT_GROUP_CURSOR"),
            Self::GroupIcon => f.write_str("RT_GROUP_ICON"),
            Self::Version => f.write_str("RT_VERSION"),
            Self::DialogInclude => f.write_str("RT_DLGINCLUDE"),
            Self::PlugPlay => f.write_str("RT_PLUGPLAY"),
            Self::Vxd => f.write_str("RT_VXD"),
            Self::AnimatedCursor => f.write_str("RT_ANICURSOR"),
            Self::AnimatedIcon => f.write_str("RT_ANIICON"),
            Self::Html => f.write_str("RT_HTML"),
            Self::Manifest => f.write_str("RT_MANIFEST"),
        }
    }
}

impl From<ResourceType> for u32 {
    #[inline]
    fn from(resource_type: ResourceType) -> Self {
        resource_type.id()
    }
}
