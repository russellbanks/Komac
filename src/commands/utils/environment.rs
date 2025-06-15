use std::{env, sync::LazyLock};

pub static CI: LazyLock<bool> =
    LazyLock::new(|| env::var("CI").is_ok_and(|ci| ci.parse() == Ok(true)));

pub static VHS: LazyLock<bool> =
    LazyLock::new(|| env::var("VHS").is_ok_and(|vhs| vhs.parse() == Ok(true)));
