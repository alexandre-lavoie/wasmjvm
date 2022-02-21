use std::collections::HashMap;

use wasmjvm_class::{
    Attribute, AttributeBody, SourceStream, WithAccessFlags, WithAttributes, WithMethods, Class,
};
use wasmjvm_common::{FromData, Parsable, Stream, WasmJVMError};

use crate::{ObjectRef, OpCode};

#[derive(Default, Debug)]
pub struct VM {
    classfiles: HashMap<String, wasmjvm_class::Class>,
    main_class: Option<String>,
}

impl VM {
    pub fn new() -> VM {
        VM {
            ..Default::default()
        }
    }

    pub fn classfile(self: &Self, name: &String) -> Result<&wasmjvm_class::Class, WasmJVMError> {
        let result = self.classfiles.get(name);

        if let Some(classfile) = result {
            Ok(classfile)
        } else {
            Err(WasmJVMError::ClassNotFound)
        }
    }

    pub fn load_class_file(self: &mut Self, classfile: Class) -> Result<(), WasmJVMError> {
        let classname = classfile.this_class().clone();

        if self.main_class.is_none() {
            self.main_class = Some(classname.clone());
        }

        if self.classfiles.contains_key(&classname) {
            return Err(WasmJVMError::ClassInvalid);
        }

        self.classfiles.insert(classname, classfile);

        Ok(())
    }

    pub fn load_class_file_path(self: &mut Self, path: &String) -> Result<(), WasmJVMError> {
        let result = wasmjvm_class::Class::from_string(path);

        if let Ok(classfile) = result {
            self.load_class_file(classfile)
        } else {
            Err(WasmJVMError::ClassInvalid)
        }
    }

    pub fn run(self: &mut Self) -> Result<Option<ObjectRef>, WasmJVMError> {
        if let Some(main_class) = self.main_class.clone() {
            Ok(self.run_method(&main_class, &"main".to_string(), Vec::new(), None)?)
        } else {
            Err(WasmJVMError::ClassNotFound)
        }
    }

    fn run_method(
        self: &mut Self,
        class_name: &String,
        method_name: &String,
        mut params: Vec<ObjectRef>,
        this: Option<ObjectRef>,
    ) -> Result<Option<ObjectRef>, WasmJVMError> {
        let method = (if let Some(main_class) = self.classfiles.get(class_name) {
            if let Ok(method) = main_class.method(method_name) {
                Ok(method.clone())
            } else {
                Err(WasmJVMError::MethodNotFound)
            }
        } else {
            Err(WasmJVMError::ClassNotFound)
        })?;

        if method
            .access_flags()
            .has_type(&wasmjvm_class::AccessFlagType::Native)
        {
            todo!();
        } else if let Ok(Attribute {
            body:
                AttributeBody::Code {
                    max_stack,
                    max_locals,
                    code,
                    ..
                },
            ..
        }) = method.attribute(&"Code".to_string())
        {
            let mut stack: Vec<ObjectRef> = Vec::with_capacity(max_stack.clone() as usize);
            let mut locals: Vec<ObjectRef> = Vec::with_capacity(max_locals.clone() as usize);

            locals.append(&mut params);

            let mut source = SourceStream::from_vec(code);

            while !source.is_empty() {
                let opcode_raw = source.parse()?;
                let opcode = OpCode::from_u8(opcode_raw)?;

                match opcode {
                    OpCode::IConstM1
                    | OpCode::IConst0
                    | OpCode::IConst1
                    | OpCode::IConst2
                    | OpCode::IConst3
                    | OpCode::IConst4
                    | OpCode::IConst5 => {
                        let value = opcode_raw as i32 - OpCode::IConst0 as i32;
                        stack.push(ObjectRef::Int(value))
                    }
                    OpCode::Ireturn => return Ok(stack.pop()),
                    _ => todo!("Not Implemented: OpCode {:?}", opcode),
                }
            }

            Ok(Some(ObjectRef::Void))
        } else {
            Err(WasmJVMError::MethodInvalid)
        }
    }
}
