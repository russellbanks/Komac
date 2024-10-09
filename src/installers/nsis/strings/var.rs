use crate::installers::nsis::version::NsisVersion;

const STRINGS: [&str; 12] = [
    // INST_0 through INST_9
    // INST_R0 through INST_R9
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

/// There are 10 constants named R0 through to R9
const NUM_R_INT_VARS: u32 = 10;

/// The VAR constants have 20 integer constants before the strings: 0-9 and R0-9
const TOTAL_INT_VARS: u32 = NUM_R_INT_VARS * 2;

#[expect(clippy::cast_possible_truncation)]
const NUM_INTERNAL_VARS: u32 = TOTAL_INT_VARS + STRINGS.len() as u32;

const VAR_EXE_PATH: u32 = 27;

pub struct NsVar;

impl NsVar {
    pub fn resolve(buf: &mut String, mut index: u32, nsis_version: NsisVersion) {
        buf.push('$');
        if index < TOTAL_INT_VARS {
            if index >= NUM_R_INT_VARS {
                buf.push('R');
                index -= NUM_R_INT_VARS;
            }
            buf.push_str(itoa::Buffer::new().format(index));
        } else if index < NUM_INTERNAL_VARS {
            if nsis_version == NsisVersion(2, 2, 5) && index >= VAR_EXE_PATH {
                index += 2;
            }
            if let Some(var_string) = STRINGS.get((index - TOTAL_INT_VARS) as usize) {
                buf.push_str(var_string);
            }
        } else {
            buf.push('_');
            buf.push_str(itoa::Buffer::new().format(index));
            buf.push('_');
        }
    }
}
