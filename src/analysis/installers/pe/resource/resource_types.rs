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

    /// Returns the resource type as a static string.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Cursor => "RT_CURSOR",
            Self::Bitmap => "RT_BITMAP",
            Self::Icon => "RT_ICON",
            Self::Menu => "RT_MENU",
            Self::Dialog => "RT_DIALOG",
            Self::String => "RT_STRING",
            Self::FontDirectory => "RT_FONTDIR",
            Self::Font => "RT_FONT",
            Self::Accelerator => "RT_ACCELERATOR",
            Self::RCData => "RT_RCDATA",
            Self::MessageTable => "RT_MESSAGETABLE",
            Self::GroupCursor => "RT_GROUP_CURSOR",
            Self::GroupIcon => "RT_GROUP_ICON",
            Self::Version => "RT_VERSION",
            Self::DialogInclude => "RT_DLGINCLUDE",
            Self::PlugPlay => "RT_PLUGPLAY",
            Self::Vxd => "RT_VXD",
            Self::AnimatedCursor => "RT_ANICURSOR",
            Self::AnimatedIcon => "RT_ANIICON",
            Self::Html => "RT_HTML",
            Self::Manifest => "RT_MANIFEST",
        }
    }
}

impl fmt::Debug for ResourceType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self}({})", self.id())
    }
}

impl fmt::Display for ResourceType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_str().fmt(f)
    }
}

impl From<ResourceType> for u32 {
    #[inline]
    fn from(resource_type: ResourceType) -> Self {
        resource_type.id()
    }
}

impl TryFrom<u32> for ResourceType {
    type Error = &'static str;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Self::Cursor),
            2 => Ok(Self::Bitmap),
            3 => Ok(Self::Icon),
            4 => Ok(Self::Menu),
            5 => Ok(Self::Dialog),
            6 => Ok(Self::String),
            7 => Ok(Self::FontDirectory),
            8 => Ok(Self::Font),
            9 => Ok(Self::Accelerator),
            10 => Ok(Self::RCData),
            11 => Ok(Self::MessageTable),
            12 => Ok(Self::GroupCursor),
            14 => Ok(Self::GroupIcon),
            16 => Ok(Self::Version),
            17 => Ok(Self::DialogInclude),
            19 => Ok(Self::PlugPlay),
            20 => Ok(Self::Vxd),
            21 => Ok(Self::AnimatedCursor),
            22 => Ok(Self::AnimatedIcon),
            23 => Ok(Self::Html),
            24 => Ok(Self::Manifest),
            _ => Err("Unknown resource type"),
        }
    }
}
