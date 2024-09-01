use itoa::Integer;

pub struct LangCode;

impl LangCode {
    pub fn resolve<I: Integer>(buf: &mut String, code: I) {
        buf.push_str("$(LSTR_");
        buf.push_str(itoa::Buffer::new().format(code));
        buf.push(')');
    }
}
