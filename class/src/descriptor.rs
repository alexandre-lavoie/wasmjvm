use crate::{ClassError, Constant};

use std::{result::Result, slice::Iter};

#[derive(Debug, Clone)]
pub enum Type {
    Single(SingleType),
    Array(SingleType, usize),
}

#[derive(Debug, Clone)]
pub enum SingleType {
    Byte,
    Char,
    Double,
    Float,
    Int,
    Long,
    Object(String),
    Short,
    Boolean,
    Void,
}

#[derive(Debug, Clone)]
pub struct Descriptor {
    parameters: Vec<Type>,
    output: Type,
}

impl Descriptor {
    pub fn parameters(self: &Self) -> Iter<Type> {
        self.parameters.iter()
    }

    pub fn output(self: &Self) -> &Type {
        &self.output
    }

    pub fn from_constant(constant: &Constant) -> Result<Descriptor, ClassError> {
        match constant {
            Constant::Utf8(string) | Constant::String(string) => Self::from_string(&string),
            _ => Err(ClassError::InvalidDescriptor),
        }
    }

    fn parse_type(string: &[u8], mut offset: usize) -> Result<(Type, usize), ClassError> {
        let tag = string[offset];
        offset += 1;

        match tag {
            b'B' => Ok((Type::Single(SingleType::Byte), offset)),
            b'C' => Ok((Type::Single(SingleType::Char), offset)),
            b'D' => Ok((Type::Single(SingleType::Double), offset)),
            b'F' => Ok((Type::Single(SingleType::Float), offset)),
            b'I' => Ok((Type::Single(SingleType::Int), offset)),
            b'J' => Ok((Type::Single(SingleType::Long), offset)),
            b'L' => {
                let mut vec_string = Vec::new();

                while string[offset] as char != ';' {
                    vec_string.push(string[offset]);
                    offset += 1;
                }
                offset += 1;

                let result = String::from_utf8(vec_string);

                if let Ok(utf8_string) = result {
                    Ok((Type::Single(SingleType::Object(utf8_string)), offset))
                } else {
                    Err(ClassError::InvalidString)
                }
            },
            b'S' => Ok((Type::Single(SingleType::Short), offset)),
            b'Z' => Ok((Type::Single(SingleType::Boolean), offset)),
            b'V' => Ok((Type::Single(SingleType::Void), offset)),
            b'[' => {
                let mut array_size: usize = 1;
                while string[offset] as char == '[' {
                    array_size += 1;
                    offset += 1;
                }

                let (t, new_offset) = Self::parse_type(string, offset)?;

                match t {
                    Type::Single(single) => Ok((Type::Array(single, array_size), new_offset)),
                    _ => unreachable!(),
                }
            }
            _ => Err(ClassError::InvalidType),
        }
    }

    pub fn from_string(string: &String) -> Result<Descriptor, ClassError> {
        let string_bytes = string.as_bytes();

        let mut parameters = Vec::new();
        let mut offset = 0;

        if string_bytes[offset] as char == '(' {
            offset += 1;

            while string_bytes[offset] as char != ')' {
                let (t, new_offset) = Self::parse_type(string_bytes, offset)?;
                parameters.push(t);
                offset = new_offset;
            }

            offset += 1;
        }

        let (output, _) = Self::parse_type(string_bytes, offset)?;

        Ok(Descriptor { parameters, output })
    }
}

pub trait WithDescriptor {
    fn descriptor(self: &Self) -> &Descriptor;
}
