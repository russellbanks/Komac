use crate::installers::nsis::version::NsisVersion;
use std::borrow::Cow;
use std::collections::HashMap;

const STRINGS: [&str; 12] = [
    "CMDLINE",
    "INSTDIR",
    "OUTDIR",
    "EXEDIR",
    "LANGUAGE",
    "TEMP",
    "PLUGINSDIR",
    "EXEPATH",
    "EXEFILE",
    "HWNDPARENT",
    "_CLICK",
    "_OUTDIR",
];

/// The VAR constants have 20 integer constants before the strings: 0-9 and R0-9
const NUM_REGISTERS: usize = 20;

const NUM_INTERNAL_VARS: usize = NUM_REGISTERS + STRINGS.len();

const VAR_EXE_PATH: usize = 27;

pub struct NsVar;

impl NsVar {
    pub fn resolve(
        buf: &mut String,
        mut index: usize,
        variables: &HashMap<usize, Cow<str>>,
        nsis_version: NsisVersion,
    ) {
        if let NUM_REGISTERS..NUM_INTERNAL_VARS = index {
            if nsis_version == NsisVersion(2, 2, 5) && index >= VAR_EXE_PATH {
                index += size_of::<u16>();
            }
            if let Some(var_string) = STRINGS.get(index - NUM_REGISTERS) {
                buf.push('$');
                buf.push_str(var_string);
            }
        } else {
            if let Some(var) = variables.get(&index) {
                buf.push_str(var);
            }
        }
    }
}
