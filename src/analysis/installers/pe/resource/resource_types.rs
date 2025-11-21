use std::fmt;

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum ResourceType {
    Cursor = 1,
    Bitmap = 2,
    Icon = 3,
    Menu = 4,
    Dialog = 5,
    String = 6,
    FontDirectory = 7,
    Font = 8,
    Accelerator = 9,
    RCData = 10,
    MessageTable = 11,
    GroupCursor = 12,
    GroupIcon = 14,
    Version = 16,
    DialogInclude = 17,
    PlugPlay = 19,
    Vxd = 20,
    AnimatedCursor = 21,
    AnimatedIcon = 22,
    Html = 23,
    Manifest = 24,
}

impl ResourceType {
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
            Self::Cursor => f.write_str("Cursor"),
            Self::Bitmap => f.write_str("Bitmap"),
            Self::Icon => f.write_str("Icon"),
            Self::Menu => f.write_str("Menu"),
            Self::Dialog => f.write_str("Dialog"),
            Self::String => f.write_str("String"),
            Self::FontDirectory => f.write_str("FontDirectory"),
            Self::Font => f.write_str("Font"),
            Self::Accelerator => f.write_str("Accelerator"),
            Self::RCData => f.write_str("RCData"),
            Self::MessageTable => f.write_str("MessageTable"),
            Self::GroupCursor => f.write_str("GroupCursor"),
            Self::GroupIcon => f.write_str("GroupIcon"),
            Self::Version => f.write_str("Version"),
            Self::DialogInclude => f.write_str("DialogInclude"),
            Self::PlugPlay => f.write_str("PlugPlay"),
            Self::Vxd => f.write_str("Vxd"),
            Self::AnimatedCursor => f.write_str("AnimatedCursor"),
            Self::AnimatedIcon => f.write_str("AnimatedIcon"),
            Self::Html => f.write_str("Html"),
            Self::Manifest => f.write_str("Manifest"),
        }
    }
}

impl From<ResourceType> for u32 {
    #[inline]
    fn from(resource_type: ResourceType) -> Self {
        resource_type.id()
    }
}
