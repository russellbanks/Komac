use super::ParsedCall;

pub trait Call {
    fn call(&mut self, call: &ParsedCall) -> bool;
}
