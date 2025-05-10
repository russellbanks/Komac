use std::{borrow::Cow, collections::HashMap};

use super::PredefinedVar;
use crate::installers::nsis::version::NsisVersion;

/// The VAR constants have 20 integer constants before the strings: 0-9 and R0-9
const NUM_REGISTERS: usize = 20;

const NUM_INTERNAL_VARS: usize = NUM_REGISTERS + PredefinedVar::num_vars();

pub struct NsVar;

impl NsVar {
    pub fn resolve(
        buf: &mut String,
        index: usize,
        variables: &HashMap<usize, Cow<str>>,
        nsis_version: NsisVersion,
    ) {
        if let NUM_REGISTERS..NUM_INTERNAL_VARS = index {
            let mut offset = index - NUM_REGISTERS;
            if nsis_version == NsisVersion(2, 2, 5) && offset >= PredefinedVar::ExePath as usize {
                offset += size_of::<u16>();
            }
            if let Ok(var) = PredefinedVar::try_from(offset) {
                buf.push_str(var.as_str());
            }
        } else if let Some(var) = variables.get(&index) {
            buf.push_str(var);
        }
    }
}
