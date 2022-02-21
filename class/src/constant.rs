use crate::{
    ClassError, ClassFile, ClassResolvable, Descriptor, Parsable, SourceStream, Streamable,
};

#[derive(Debug)]
pub enum ConstantTag {
    Utf8,
    Integer,
    Float,
    Long,
    Double,
    Class,
    String,
    FieldRef,
    MethodRef,
    InterfaceMethodRef,
    NameAndType,
    MethodHandle,
    MethodType,
    InvokeDynamic,
}

impl ConstantTag {
    fn from_u8(tag: u8) -> Result<ConstantTag, ClassError> {
        match tag {
            1 => Ok(ConstantTag::Utf8),
            3 => Ok(ConstantTag::Integer),
            4 => Ok(ConstantTag::Float),
            5 => Ok(ConstantTag::Long),
            6 => Ok(ConstantTag::Double),
            7 => Ok(ConstantTag::Class),
            8 => Ok(ConstantTag::String),
            9 => Ok(ConstantTag::FieldRef),
            10 => Ok(ConstantTag::MethodRef),
            11 => Ok(ConstantTag::InterfaceMethodRef),
            12 => Ok(ConstantTag::NameAndType),
            15 => Ok(ConstantTag::MethodHandle),
            16 => Ok(ConstantTag::MethodType),
            18 => Ok(ConstantTag::InvokeDynamic),
            _ => Err(ClassError::InvalidConstant(tag)),
        }
    }
}

#[derive(Debug, Clone)]
pub enum ConstantInfo {
    Utf8(String),
    Integer(u32),
    Float(u32),
    Long(u32, u32),
    Double(u32, u32),
    Class {
        name_index: u16,
    },
    String(String),
    FieldRef {
        class_index: u16,
        name_and_type_index: u16,
    },
    MethodRef {
        class_index: u16,
        name_and_type_index: u16,
    },
    InterfaceMethodRef {
        class_index: u16,
        name_and_type_index: u16,
    },
    NameAndType {
        name_index: u16,
        descriptor_index: u16,
    },
    MethodHandle {
        reference_kind: u8,
        reference_index: u16,
    },
    MethodType {
        descriptor_index: u16,
    },
    InvokeDynamic {
        bootstrap_method_attr_index: u16,
        name_and_type_index: u16,
    },
}

#[derive(Debug, Clone)]
pub enum Constant {
    Utf8(String),
    Integer(u32),
    Float(u32),
    Long(u32, u32),
    Double(u32, u32),
    Class {
        name: String,
    },
    String(String),
    FieldRef {
        class: String,
        name: String,
        descriptor: Descriptor,
    },
    MethodRef {
        class: String,
        name: String,
        descriptor: Descriptor,
    },
    InterfaceMethodRef {
        class: String,
        name: String,
        descriptor: Descriptor,
    },
    NameAndType {
        name: String,
        descriptor: Descriptor,
    },
    MethodHandle {
        reference_kind: u8,
        reference_index: u16,
    },
    MethodType {
        descriptor_index: u16,
    },
    InvokeDynamic {
        bootstrap_method_attr_index: u16,
        name_and_type_index: u16,
    },
}

impl Constant {
    pub fn to_descriptor(self: &Self) -> Result<Descriptor, ClassError> {
        Descriptor::from_constant(self)
    }

    pub fn to_string(self: &Self) -> Result<String, ClassError> {
        match self {
            Constant::Utf8(string) | Constant::String(string) => Ok(string.clone()),
            Constant::Class { name } => Ok(name.clone()),
            _ => Err(ClassError::NotStringConstant),
        }
    }
}

impl ClassResolvable<Constant> for ConstantInfo {
    fn resolve(self: &Self, class_file: &ClassFile) -> Result<Constant, ClassError> {
        match self {
            ConstantInfo::Utf8(string) => Ok(Constant::Utf8(string.clone())),
            ConstantInfo::Integer(b0) => Ok(Constant::Integer(b0.clone())),
            ConstantInfo::Float(b0) => Ok(Constant::Float(b0.clone())),
            ConstantInfo::Long(b0, b1) => Ok(Constant::Long(b0.clone(), b1.clone())),
            ConstantInfo::Double(b0, b1) => Ok(Constant::Double(b0.clone(), b1.clone())),
            ConstantInfo::Class { name_index } => {
                let name = class_file
                    .constant(name_index.clone() as usize)?
                    .to_string()?;
                Ok(Constant::Class { name })
            }
            ConstantInfo::String(string) => Ok(Constant::String(string.clone())),
            ConstantInfo::NameAndType {
                name_index,
                descriptor_index,
            } => {
                let name = class_file
                    .constant(name_index.clone() as usize)?
                    .to_string()?;
                let descriptor = class_file
                    .constant(descriptor_index.clone() as usize)?
                    .to_descriptor()?;

                Ok(Constant::NameAndType { name, descriptor })
            }
            ConstantInfo::MethodRef {
                class_index,
                name_and_type_index,
            }
            | ConstantInfo::FieldRef {
                class_index,
                name_and_type_index,
            }
            | ConstantInfo::InterfaceMethodRef {
                class_index,
                name_and_type_index,
            } => {
                let class = class_file
                    .constant(class_index.clone() as usize)?
                    .to_string()?;

                let name_and_type_constant =
                    class_file.constant(name_and_type_index.clone() as usize)?;
                let (name, descriptor) = (match name_and_type_constant {
                    Constant::NameAndType { name, descriptor } => Ok((name, descriptor)),
                    _ => Err(ClassError::UnexpectedConstant(self.clone())),
                })?;

                Ok(match self {
                    ConstantInfo::MethodRef { .. } => Constant::MethodRef {
                        class,
                        name,
                        descriptor,
                    },
                    ConstantInfo::FieldRef { .. } => Constant::FieldRef {
                        class,
                        name,
                        descriptor,
                    },
                    ConstantInfo::InterfaceMethodRef { .. } => Constant::InterfaceMethodRef {
                        class,
                        name,
                        descriptor,
                    },
                    _ => todo!(),
                })
            }
            ConstantInfo::MethodHandle {
                reference_kind,
                reference_index,
            } => Ok(Constant::MethodHandle {
                reference_kind: reference_kind.clone(),
                reference_index: reference_index.clone(),
            }),
            ConstantInfo::MethodType { descriptor_index } => Ok(Constant::MethodType {
                descriptor_index: descriptor_index.clone(),
            }),
            ConstantInfo::InvokeDynamic {
                bootstrap_method_attr_index,
                name_and_type_index,
            } => Ok(Constant::InvokeDynamic {
                bootstrap_method_attr_index: bootstrap_method_attr_index.clone(),
                name_and_type_index: name_and_type_index.clone(),
            }),
        }
    }
}

impl Streamable<ConstantInfo> for ConstantInfo {
    fn from_stream(stream: &mut SourceStream) -> Result<ConstantInfo, ClassError> {
        let raw_tag = stream.parse()?;
        let tag = ConstantTag::from_u8(raw_tag)?;
        match tag {
            ConstantTag::Utf8 => {
                let count: u16 = stream.parse()?;
                let u8_str = stream.parse_vec(count as usize)?;

                if let Ok(string) = String::from_utf8(u8_str) {
                    Ok(ConstantInfo::Utf8(string))
                } else {
                    Err(ClassError::InvalidString)
                }
            }
            ConstantTag::Integer | ConstantTag::Float => {
                let bytes = stream.parse()?;

                match tag {
                    ConstantTag::Integer => Ok(ConstantInfo::Integer(bytes)),
                    ConstantTag::Float => Ok(ConstantInfo::Float(bytes)),
                    _ => todo!(),
                }
            }
            ConstantTag::Long | ConstantTag::Double => {
                let high_bytes = stream.parse()?;
                let low_bytes = stream.parse()?;

                match tag {
                    ConstantTag::Long => Ok(ConstantInfo::Long(high_bytes, low_bytes)),
                    ConstantTag::Double => Ok(ConstantInfo::Double(high_bytes, low_bytes)),
                    _ => todo!(),
                }
            }
            ConstantTag::Class => {
                let name_index = stream.parse()?;

                Ok(ConstantInfo::Class { name_index })
            }
            ConstantTag::String => {
                let count: u16 = stream.parse()?;
                let u8_str = stream.parse_vec(count as usize)?;

                if let Ok(string) = String::from_utf8(u8_str) {
                    Ok(ConstantInfo::String(string))
                } else {
                    Err(ClassError::InvalidString)
                }
            }
            ConstantTag::MethodRef | ConstantTag::FieldRef | ConstantTag::InterfaceMethodRef => {
                let class_index = stream.parse()?;
                let name_and_type_index = stream.parse()?;

                match tag {
                    ConstantTag::MethodRef => Ok(ConstantInfo::MethodRef {
                        class_index,
                        name_and_type_index,
                    }),
                    ConstantTag::FieldRef => Ok(ConstantInfo::FieldRef {
                        class_index,
                        name_and_type_index,
                    }),
                    ConstantTag::InterfaceMethodRef => Ok(ConstantInfo::InterfaceMethodRef {
                        class_index,
                        name_and_type_index,
                    }),
                    _ => todo!(),
                }
            }
            ConstantTag::NameAndType => {
                let name_index = stream.parse()?;
                let descriptor_index = stream.parse()?;

                Ok(ConstantInfo::NameAndType {
                    name_index,
                    descriptor_index,
                })
            }
            ConstantTag::MethodHandle => {
                let reference_kind = stream.parse()?;
                let reference_index = stream.parse()?;

                Ok(ConstantInfo::MethodHandle {
                    reference_kind,
                    reference_index,
                })
            }
            ConstantTag::MethodType => {
                let descriptor_index = stream.parse()?;

                Ok(ConstantInfo::MethodType { descriptor_index })
            }
            ConstantTag::InvokeDynamic => {
                let bootstrap_method_attr_index = stream.parse()?;
                let name_and_type_index = stream.parse()?;

                Ok(ConstantInfo::InvokeDynamic {
                    bootstrap_method_attr_index,
                    name_and_type_index,
                })
            }
        }
    }
}
