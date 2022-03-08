use std::collections::HashMap;

use wasmjvm_class::{
    AccessFlagType, Class, Constant, SingleType, Type, WithAccessFlags, WithDescriptor, WithFields,
};
use wasmjvm_common::WasmJVMError;

use crate::{ClassInstance, Loader, NativeInterface, Thread};

#[derive(Debug)]
pub enum RustObject {
    Class(ClassInstance),
    String(String),
    Array(Vec<Primitive>),
    Thread(Thread),
    Loader(Loader),
    Native(NativeInterface),
    Null,
}

#[derive(Debug)]
pub struct Object {
    class: Option<usize>,
    inner: RustObject,
    pub fields: HashMap<String, Primitive>,
}

impl Object {
    pub fn new(
        class_index: usize,
        fields: Vec<String>,
        inner: RustObject,
    ) -> Result<Self, WasmJVMError> {
        let mut fields_map = HashMap::new();

        for field in fields {
            fields_map.insert(field, Primitive::Null);
        }

        Ok(Self {
            class: Some(class_index),
            inner,
            fields: fields_map,
        })
    }

    pub fn new_array(raw: Vec<Primitive>) -> Result<Self, WasmJVMError> {
        Ok(Self {
            class: None,
            inner: RustObject::Array(raw),
            fields: HashMap::new(),
        })
    }

    pub fn new_empty_array(size: usize) -> Result<Self, WasmJVMError> {
        // TODO: Use type default value.
        Self::new_array(vec![Primitive::Null; size])
    }

    pub fn class(self: &Self) -> Option<usize> {
        self.class
    }

    pub fn inner(self: &Self) -> &RustObject {
        &self.inner
    }

    pub fn inner_mut(self: &mut Self) -> &mut RustObject {
        &mut self.inner
    }
}

#[derive(Debug, Clone)]
pub enum Primitive {
    Void,
    Null,
    Boolean(bool),
    Byte(u8),
    Char(u8),
    Short(u16),
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    Reference(usize),
}

macro_rules! primitive_into {
    ($name:ident, $primitive:ident, $type:ident) => {
        pub fn $name(self: &Self) -> Result<Self, WasmJVMError> {
            match self {
                Primitive::Boolean(value) => Ok(Primitive::$primitive(*value as u8 as $type)),
                Primitive::Byte(value) | Primitive::Char(value) => {
                    Ok(Primitive::$primitive(*value as $type))
                }
                Primitive::Short(value) => Ok(Primitive::$primitive(*value as $type)),
                Primitive::Int(value) => Ok(Primitive::$primitive(*value as $type)),
                Primitive::Long(value) => Ok(Primitive::$primitive(*value as $type)),
                Primitive::Float(value) => Ok(Primitive::$primitive(*value as $type)),
                Primitive::Double(value) => Ok(Primitive::$primitive(*value as $type)),
                Primitive::Null => Ok(Primitive::$primitive(0 as $type)),
                _ => panic!("Failed to cast {:?} to {}.", self, stringify!($type)),
            }
        }
    };
}

macro_rules! primitive_op {
    ($name:ident, $op:tt) => {
        pub fn $name(self: &Self, other: &Self) -> Result<Self, WasmJVMError> {
            match (self, other) {
                (Primitive::Int(left), Primitive::Int(right)) => Ok(Primitive::Int(left $op right)),
                (Primitive::Long(left), Primitive::Long(right)) => Ok(Primitive::Long(left $op right)),
                (Primitive::Float(left), Primitive::Float(right)) => Ok(Primitive::Float(left $op right)),
                (Primitive::Double(left), Primitive::Double(right)) => Ok(Primitive::Double(left $op right)),
                _ => unreachable!()
            }
        }
    }
}

impl Primitive {
    primitive_into!(into_float, Float, f32);
    primitive_into!(into_double, Double, f64);
    primitive_into!(into_int, Int, i32);
    primitive_into!(into_long, Long, i64);
    primitive_into!(into_byte, Byte, u8);
    primitive_into!(into_char, Char, u8);
    primitive_into!(into_short, Short, u16);

    pub fn into_void(self: &Self) -> Result<Self, WasmJVMError> {
        match self {
            Self::Void => Ok(Self::Void),
            _ => unreachable!(),
        }
    }

    pub fn into_type(self: &Self, r#type: &Type) -> Result<Self, WasmJVMError> {
        match r#type {
            Type::Array(..) => self.into_ref(),
            Type::Single(single) => match single {
                SingleType::Boolean => todo!(),
                SingleType::Byte => self.into_byte(),
                SingleType::Char => self.into_char(),
                SingleType::Double => self.into_double(),
                SingleType::Float => self.into_float(),
                SingleType::Int => self.into_int(),
                SingleType::Long => self.into_long(),
                SingleType::Short => self.into_short(),
                SingleType::Object(..) => self.into_ref(),
                SingleType::Void => self.into_void(),
                _ => unreachable!(),
            },
            _ => unreachable!(),
        }
    }

    primitive_op!(add, +);
    primitive_op!(sub, -);
    primitive_op!(mul, *);
    primitive_op!(div, /);
    primitive_op!(rem, %);

    pub fn is_void(self: &Self) -> bool {
        match self {
            Primitive::Void => true,
            _ => false,
        }
    }

    pub fn is_null(self: &Self) -> bool {
        match self {
            Primitive::Null => true,
            _ => false,
        }
    }

    pub fn into_ref(self: &Self) -> Result<Self, WasmJVMError> {
        match self {
            Primitive::Null => Ok(Primitive::Null),
            Primitive::Reference(value) => Ok(Primitive::Reference(*value)),
            _ => panic!("Invalid reference: {:?}.", self),
        }
    }

    pub fn neg(self: &Self) -> Result<Self, WasmJVMError> {
        match self {
            Primitive::Int(value) => Ok(Primitive::Int(-value)),
            Primitive::Long(value) => Ok(Primitive::Long(-value)),
            Primitive::Float(value) => Ok(Primitive::Float(-value)),
            Primitive::Double(value) => Ok(Primitive::Double(-value)),
            _ => todo!(),
        }
    }

    pub fn cmpg(self: &Self, other: &Self) -> Result<Self, WasmJVMError> {
        if self.is_null() || other.is_null() {
            Ok(Primitive::Int(1))
        } else {
            self.cmp(other)
        }
    }

    pub fn cmpl(self: &Self, other: &Self) -> Result<Self, WasmJVMError> {
        if self.is_null() || other.is_null() {
            Ok(Primitive::Int(-1))
        } else {
            self.cmp(other)
        }
    }

    pub fn cmp(self: &Self, other: &Self) -> Result<Self, WasmJVMError> {
        let (gt, eq) = match (self, other) {
            (Primitive::Int(left), Primitive::Int(right)) => (left > right, left == right),
            (Primitive::Long(left), Primitive::Long(right)) => (left > right, left == right),
            (Primitive::Float(left), Primitive::Float(right)) => (left > right, left == right),
            (Primitive::Double(left), Primitive::Double(right)) => (left > right, left == right),
            _ => unreachable!(),
        };

        if gt {
            Ok(Primitive::Int(1))
        } else if eq {
            Ok(Primitive::Int(0))
        } else {
            Ok(Primitive::Int(-1))
        }
    }
}

impl From<Constant> for Primitive {
    fn from(constant: Constant) -> Self {
        match constant {
            Constant::Integer(value) => Primitive::Int(value),
            Constant::Float(value) => Primitive::Float(value),
            Constant::Long(value) => Primitive::Long(value),
            Constant::Double(value) => Primitive::Double(value),
            _ => unreachable!(),
        }
    }
}
