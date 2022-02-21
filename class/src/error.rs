use crate::{Constant, ConstantInfo};

#[derive(Debug)]
pub enum ClassError {
    OutOfBound,
    FileError,
    BadMagic,
    InvalidConstant(u8),
    InvalidString,
    UnexpectedConstant(ConstantInfo),
    InvalidConstantConversion(Constant),
    InvalidNameDescriptor,
    InvalidDescriptor,
    InvalidField,
    InvalidType,
    NotStringConstant,
    MethodNotFound,
    FieldNotFound,
    AttributeNotFound,
    InterfaceNotFound,
}
