use serde::Deserialize;

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BundleCacheType {
    // Wix v4+
    //
    // https://docs.firegiant.com/wix/api/wixtoolsetdata/bundlecachetype/
    // https://github.com/wixtoolset/wix/blob/main/src/api/wix/WixToolset.Data/BundleCacheType.cs#L8
    #[default]
    Keep,
    Remove,
    Force,

    // Wix v3
    //
    // https://github.com/wixtoolset/wix3/blob/master/src/libs/balutil/inc/balinfo.h#L21
    No,
    Yes, // Default in Wix v3
    Always,
}
