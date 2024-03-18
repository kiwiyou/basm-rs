use basm::platform::io::{Reader, ReaderTrait, Writer, Print};
pub fn main() {
    let mut reader: Reader = Default::default();
    let mut writer: Writer = Default::default();
    let a = reader.i64();
    let b = reader.i64();
    writer.println(a + b);
}
