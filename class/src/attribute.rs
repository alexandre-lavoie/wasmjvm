use crate::{ClassFile, ClassResolvable, SourceStream};

use wasmjvm_common::{WasmJVMError, Streamable, Parsable, FromData};
use std::slice::Iter;

#[derive(Debug)]
pub struct AttributeInfo {
    attribute_name_index: u16,
    info: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct Attribute {
    name: String,
    pub body: AttributeBody,
}

#[derive(Debug, Clone)]
pub struct ExceptionEntry {
    pub start_pc: u16,
    pub end_pc: u16,
    pub handler_pc: u16,
    pub catch_type: u16,
}

#[derive(Debug, Clone)]
pub struct LineNumberEntry {
    pub start_pc: u16,
    pub line_number: u16,
}

#[derive(Debug, Clone)]
pub struct CodeBody {
    pub max_stack: u16,
    pub max_locals: u16,
    pub code: Vec<u8>,
    pub exception_table: Vec<ExceptionEntry>,
    pub attributes: Vec<Attribute>,
}

#[derive(Debug, Clone)]
pub enum AttributeBody {
    Code(CodeBody),
    LineNumberTable {
        line_number_table: Vec<LineNumberEntry>,
    },
    SourceFile {
        sourcefile: String,
    },
    Signature {
        signature_index: u16
    },
    User {
        info: Vec<u8>,
    },
}

impl Attribute {
    pub fn name(self: &Self) -> &str {
        self.name.as_str()
    }
}

impl ClassResolvable<Attribute> for AttributeInfo {
    fn resolve(self: &Self, class_file: &ClassFile) -> Result<Attribute, WasmJVMError> {
        let name = class_file
            .constant(self.attribute_name_index as usize)?
            .to_string()?;
        let mut source = SourceStream::from_vec(self.info.clone());

        let body = match name.as_str() {
            "Code" => {
                let max_stack = source.parse()?;
                let max_locals = source.parse()?;

                let code_length: u32 = source.parse()?;
                let code = source.parse_vec(code_length as usize)?;

                let exception_table_length: u16 = source.parse()?;
                let exception_table = source.parse_vec(exception_table_length as usize)?;

                let attribute_count: u16 = source.parse()?;
                let attribute_infos: Vec<AttributeInfo> =
                    source.parse_vec(attribute_count as usize)?;
                let attributes = class_file.resolve_vec(&attribute_infos)?;

                AttributeBody::Code(CodeBody {
                    max_stack,
                    max_locals,
                    code,
                    exception_table,
                    attributes,
                })
            }
            "LineNumberTable" => {
                let line_number_table_length: u16 = source.parse()?;
                let line_number_table = source.parse_vec(line_number_table_length as usize)?;

                AttributeBody::LineNumberTable { line_number_table }
            }
            "SourceFile" => {
                let sourcefile_index: u16 = source.parse()?;
                let sourcefile = class_file
                    .constant(sourcefile_index as usize)?
                    .to_string()?;

                AttributeBody::SourceFile { sourcefile }
            }
            "Signature" => {
                let signature_index: u16 = source.parse()?;

                AttributeBody::Signature { signature_index }
            }
            _ => AttributeBody::User {
                info: self.info.clone(),
            },
        };

        Ok(Attribute { name, body })
    }
}

impl Streamable<SourceStream, ExceptionEntry> for ExceptionEntry {
    fn from_stream(stream: &mut SourceStream) -> Result<ExceptionEntry, WasmJVMError> {
        let start_pc = stream.parse()?;
        let end_pc = stream.parse()?;
        let handler_pc = stream.parse()?;
        let catch_type = stream.parse()?;

        Ok(ExceptionEntry {
            start_pc,
            end_pc,
            handler_pc,
            catch_type,
        })
    }
}

impl Streamable<SourceStream, LineNumberEntry> for LineNumberEntry {
    fn from_stream(stream: &mut SourceStream) -> Result<LineNumberEntry, WasmJVMError> {
        let start_pc = stream.parse()?;
        let line_number = stream.parse()?;

        Ok(LineNumberEntry {
            start_pc,
            line_number,
        })
    }
}

impl Streamable<SourceStream, AttributeInfo> for AttributeInfo {
    fn from_stream(stream: &mut SourceStream) -> Result<AttributeInfo, WasmJVMError> {
        let attribute_name_index = stream.parse()?;
        let attribute_length: u32 = stream.parse()?;
        let info = stream.parse_vec(attribute_length as usize)?;

        Ok(AttributeInfo {
            attribute_name_index,
            info,
        })
    }
}

pub trait WithAttributes {
    fn attributes(self: &Self) -> Option<Iter<Attribute>>;

    fn attribute(self: &Self, name: &str) -> Result<&Attribute, WasmJVMError> {
        if let Some(attributes) = self.attributes() {
            for attribute in attributes {
                if attribute.name() == name {
                    return Ok(attribute);
                }
            }
        }

        Err(WasmJVMError::NoSuchFieldError(format!("Attribute {}", name)))
    }
}

impl WithAttributes for Attribute {
    fn attributes(self: &Self) -> Option<Iter<Attribute>> {
        match &self.body {
            AttributeBody::Code(code) => Some(code.attributes.iter()),
            _ => None,
        }
    }
}
