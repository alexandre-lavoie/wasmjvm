use crate::{ClassError, ClassFile, ClassResolvable, Parsable, SourceStream, Streamable};

use std::slice::Iter;

#[derive(Debug, Default)]
pub struct InterfaceInfo {
    name_index: u16,
}

#[derive(Debug)]
pub struct Interface {
    name: String,
}

impl Interface {
    fn name(self: &Self) -> &String {
        &self.name
    }
}

pub trait WithInterfaces {
    fn interfaces(self: &Self) -> Option<Iter<Interface>>;

    fn interface(self: &Self, name: &String) -> Result<&Interface, ClassError> {
        if let Some(interfaces) = self.interfaces() {
            for interface in interfaces {
                if interface.name() == name {
                    return Ok(interface);
                }
            }
        }

        Err(ClassError::InterfaceNotFound)
    }
}

impl ClassResolvable<Interface> for InterfaceInfo {
    fn resolve(self: &Self, class_file: &ClassFile) -> Result<Interface, ClassError> {
        let name = class_file.constant(self.name_index as usize)?.to_string()?;

        Ok(Interface { name })
    }
}

impl Streamable<InterfaceInfo> for InterfaceInfo {
    fn from_stream(stream: &mut SourceStream) -> Result<InterfaceInfo, ClassError> {
        let name_index = stream.parse()?;

        Ok(InterfaceInfo { name_index })
    }
}
