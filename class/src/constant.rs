use crate::{ClassFile, ClassResolvable, Descriptor, SourceStream};

use wasmjvm_common::{Parsable, Streamable, WasmJVMError};

#[derive(Debug)]
pub enum ConstantTag {
    Empty,
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
    fn from_u8(tag: u8) -> Result<ConstantTag, WasmJVMError> {
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
            _ => Err(WasmJVMError::ConstantInvalid),
        }
    }
}

#[derive(Debug, Clone)]
pub enum ConstantInfo {
    Empty,
    Utf8(Vec<u8>),
    Integer(u32),
    Float(u32),
    Long(u32, u32),
    Double(u32, u32),
    Class {
        name_index: u16,
    },
    String {
        string_index: u16,
    },
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MethodRef {
    pub class: String,
    pub name: String,
    pub descriptor: Descriptor,
}

impl MethodRef {
    pub fn string_init() -> Self {
        MethodRef {
            class: "java/lang/String".to_string(),
            name: "<init>".to_string(),
            descriptor: Descriptor::void(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FieldRef {
    pub class: String,
    pub name: String,
    pub descriptor: Descriptor,
}

#[derive(Debug, Clone)]
pub enum Constant {
    Empty,
    Utf8(String),
    Integer(i32),
    Float(f32),
    Long(i64),
    Double(f64),
    Class {
        name: String,
    },
    String(String),
    FieldRef(FieldRef),
    MethodRef(MethodRef),
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
        descriptor: Descriptor,
    },
    InvokeDynamic {
        name: String,
        descriptor: Descriptor,
        bootstrap_method_attr_index: u16,
    },
}

impl Constant {
    pub fn to_descriptor(self: &Self) -> Result<Descriptor, WasmJVMError> {
        Descriptor::from_constant(self)
    }

    pub fn to_name_descritor(self: &Self) -> Result<(String, Descriptor), WasmJVMError> {
        match self {
            Constant::NameAndType { name, descriptor } => Ok((name.clone(), descriptor.clone())),
            _ => Err(WasmJVMError::NameDescriptorInvalid),
        }
    }

    pub fn to_string(self: &Self) -> Result<String, WasmJVMError> {
        match self {
            Constant::Utf8(string) | Constant::String(string) => Ok(string.clone()),
            Constant::Class { name } => Ok(name.clone()),
            _ => Err(WasmJVMError::StringInvalid),
        }
    }
}

impl ClassResolvable<Constant> for ConstantInfo {
    fn resolve(self: &Self, class_file: &ClassFile) -> Result<Constant, WasmJVMError> {
        match self {
            ConstantInfo::Empty => Ok(Constant::Empty),
            ConstantInfo::Utf8(u8_str) => {
                let result = String::from_utf8(u8_str.to_vec());

                if let Ok(string) = result {
                    Ok(Constant::Utf8(string))
                } else {
                    Err(WasmJVMError::StringInvalid)
                }
            }
            ConstantInfo::Integer(b0) => Ok(Constant::Integer(b0.clone() as i32)),
            ConstantInfo::Float(b0) => Ok(Constant::Float(f32::from_bits(b0.clone()))),
            ConstantInfo::Long(b0, b1) => Ok(Constant::Long(
                ((b0.clone() as u64) << 32) as i64 + b1.clone() as i64,
            )),
            ConstantInfo::Double(b0, b1) => Ok(Constant::Double(f64::from_bits(
                ((b0.clone() as u64) << 32) | (b1.clone() as u64),
            ))),
            ConstantInfo::Class { name_index } => {
                let name = class_file
                    .constant(name_index.clone() as usize)?
                    .to_string()?;
                Ok(Constant::Class { name })
            }
            ConstantInfo::String { string_index } => {
                let string = class_file
                    .constant(string_index.clone() as usize)?
                    .to_string()?;

                Ok(Constant::String(string))
            }
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

                let (name, descriptor) = class_file
                    .constant(name_and_type_index.clone() as usize)?
                    .to_name_descritor()?;

                Ok(match self {
                    ConstantInfo::MethodRef { .. } => Constant::MethodRef(MethodRef {
                        class,
                        name,
                        descriptor,
                    }),
                    ConstantInfo::FieldRef { .. } => Constant::FieldRef(FieldRef {
                        class,
                        name,
                        descriptor,
                    }),
                    ConstantInfo::InterfaceMethodRef { .. } => Constant::InterfaceMethodRef {
                        class,
                        name,
                        descriptor,
                    },
                    _ => unreachable!(),
                })
            }
            ConstantInfo::MethodHandle {
                reference_kind,
                reference_index,
            } => Ok(Constant::MethodHandle {
                reference_kind: reference_kind.clone(),
                reference_index: reference_index.clone(),
            }),
            ConstantInfo::MethodType { descriptor_index } => {
                let descriptor = class_file
                    .constant(descriptor_index.clone() as usize)?
                    .to_descriptor()?;

                Ok(Constant::MethodType { descriptor })
            }
            ConstantInfo::InvokeDynamic {
                bootstrap_method_attr_index,
                name_and_type_index,
            } => {
                let (name, descriptor) = class_file
                    .constant(name_and_type_index.clone() as usize)?
                    .to_name_descritor()?;

                Ok(Constant::InvokeDynamic {
                    name,
                    descriptor,
                    bootstrap_method_attr_index: bootstrap_method_attr_index.clone(),
                })
            }
        }
    }
}

impl Streamable<SourceStream, ConstantInfo> for ConstantInfo {
    fn from_stream(stream: &mut SourceStream) -> Result<ConstantInfo, WasmJVMError> {
        let raw_tag = stream.parse()?;
        let tag = ConstantTag::from_u8(raw_tag)?;
        match tag {
            ConstantTag::Empty => Ok(ConstantInfo::Empty),
            ConstantTag::Utf8 => {
                let count: u16 = stream.parse()?;
                let u8_str = stream.parse_vec(count as usize)?;

                Ok(ConstantInfo::Utf8(u8_str))
            }
            ConstantTag::Integer | ConstantTag::Float => {
                let bytes = stream.parse()?;

                match tag {
                    ConstantTag::Integer => Ok(ConstantInfo::Integer(bytes)),
                    ConstantTag::Float => Ok(ConstantInfo::Float(bytes)),
                    _ => unreachable!(),
                }
            }
            ConstantTag::Long | ConstantTag::Double => {
                let high_bytes = stream.parse()?;
                let low_bytes = stream.parse()?;

                match tag {
                    ConstantTag::Long => Ok(ConstantInfo::Long(high_bytes, low_bytes)),
                    ConstantTag::Double => Ok(ConstantInfo::Double(high_bytes, low_bytes)),
                    _ => unreachable!(),
                }
            }
            ConstantTag::Class => {
                let name_index = stream.parse()?;

                Ok(ConstantInfo::Class { name_index })
            }
            ConstantTag::String => {
                let string_index: u16 = stream.parse()?;

                Ok(ConstantInfo::String { string_index })
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
                    _ => unreachable!(),
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
