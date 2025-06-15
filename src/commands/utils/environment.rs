use std::{env, sync::LazyLock};

pub static CI: LazyLock<bool> =
    LazyLock::new(|| env::var("CI").is_ok_and(|ci| ci.parse() == Ok(true)));

pub static VHS_RECORD: LazyLock<bool> =
    LazyLock::new(|| env::var("VHS_RECORD").is_ok_and(|vhs| vhs.parse() == Ok(true)));
