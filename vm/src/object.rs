#[derive(Debug)]
pub enum ObjectRef {
    Byte(u8),
    Char(u8),
    Double(f64),
    Float(f32),
    Int(i32),
    Long(i64),
    Object(String),
    Short(u8),
    Boolean(u8),
    Void,
}
