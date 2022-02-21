use crate::{
    AccessFlags, Attribute, AttributeInfo, ClassError, ClassFile, ClassResolvable, Constant,
    Descriptor, Parsable, SourceStream, Streamable, WithAttributes, WithAccessFlags, WithDescriptor,
};

use std::{result::Result, slice::Iter};

#[derive(Debug)]
pub struct FieldInfo {
    access_flags: AccessFlags,
    name_index: u16,
    descriptor_index: u16,
    attributes: Vec<AttributeInfo>,
}

#[derive(Debug)]
pub struct Field {
    access_flags: AccessFlags,
    name: String,
    descriptor: Descriptor,
    attributes: Vec<Attribute>,
}

impl Field {
    pub fn name(self: &Self) -> &String {
        return &self.name;
    }
}

pub trait WithFields {
    fn fields(self: &Self) -> Option<Iter<Field>>;

    fn field(self: &Self, name: &String) -> Result<&Field, ClassError> {
        if let Some(fields) = self.fields() {
            for field in fields {
                if field.name() == name {
                    return Ok(field);
                }
            }
        }

        Err(ClassError::FieldNotFound)
    }
}

impl WithAttributes for Field {
    fn attributes(self: &Self) -> Option<Iter<Attribute>> {
        Some(self.attributes.iter())
    }
}

impl WithAccessFlags for Field {
    fn access_flags(self: &Self) -> &AccessFlags {
        &self.access_flags
    }
}

impl WithDescriptor for Field {
    fn descriptor(self: &Self) -> &Descriptor {
        &self.descriptor
    }
}

impl ClassResolvable<Field> for FieldInfo {
    fn resolve(self: &Self, class_file: &ClassFile) -> Result<Field, ClassError> {
        let access_flags = self.access_flags.clone();
        let name_constant = class_file.constant(self.name_index as usize)?;

        let name = (match name_constant {
            Constant::Utf8(string) | Constant::String(string) => Ok(string),
            _ => Err(ClassError::InvalidField),
        })?;

        let descriptor = class_file
            .constant(self.descriptor_index as usize)?
            .to_descriptor()?;
        let attributes = class_file.resolve_vec(&self.attributes)?;
        Ok(Field {
            access_flags,
            name,
            descriptor,
            attributes,
        })
    }
}

impl Streamable<FieldInfo> for FieldInfo {
    fn from_stream(stream: &mut SourceStream) -> Result<FieldInfo, ClassError> {
        let access_flags = stream.parse()?;
        let name_index = stream.parse()?;
        let descriptor_index = stream.parse()?;

        let attribute_count: u16 = stream.parse()?;
        let attributes = stream.parse_vec(attribute_count as usize)?;

        Ok(FieldInfo {
            access_flags,
            name_index,
            descriptor_index,
            attributes,
        })
    }
}
