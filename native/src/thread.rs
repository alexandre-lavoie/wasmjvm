use std::collections::HashSet;

use crate::{Global, Object, OpCode, Primitive, RustObject};
use wasmjvm_class::{
    AccessFlagType, Constant, Descriptor, ExceptionEntry, MethodRef, SingleType, Type,
    WithAccessFlags, WithAttributes, WithInterfaces,
};
use wasmjvm_common::WasmJVMError;

#[derive(Debug)]
pub struct Frame {
    pc: usize,
    method_ref: MethodRef,
    local_variables: Vec<Primitive>,
    operand_stack: Vec<Primitive>,
    throw: Option<Primitive>,
    throw_entries: Vec<ExceptionEntry>,
}

impl Frame {
    pub fn new(
        method_ref: MethodRef,
        local_variables: Vec<Primitive>,
    ) -> Result<Self, WasmJVMError> {
        Ok(Self {
            method_ref,
            pc: 0usize,
            local_variables,
            operand_stack: Vec::new(),
            throw: None,
            throw_entries: Vec::new(),
        })
    }

    pub fn operand_stack(self: &Self) -> &Vec<Primitive> {
        &self.operand_stack
    }

    pub fn all_mut(
        self: &mut Self,
    ) -> (
        &mut usize,
        &mut Vec<Primitive>,
        &mut Vec<Primitive>,
        &mut Option<Primitive>,
        &mut Vec<ExceptionEntry>,
    ) {
        (
            &mut self.pc,
            &mut self.operand_stack,
            &mut self.local_variables,
            &mut self.throw,
            &mut self.throw_entries,
        )
    }

    pub fn operand_stack_mut(self: &mut Self) -> &mut Vec<Primitive> {
        &mut self.operand_stack
    }

    pub fn local_variables(self: &mut Self) -> &Vec<Primitive> {
        &self.local_variables
    }

    pub fn local_variables_mut(self: &mut Self) -> &mut Vec<Primitive> {
        &mut self.local_variables
    }

    pub fn throw_mut(self: &mut Self) -> &mut Option<Primitive> {
        &mut self.throw
    }

    pub fn pc(self: &Self) -> usize {
        self.pc
    }

    pub fn pc_mut(self: &mut Self) -> &mut usize {
        &mut self.pc
    }
}

#[derive(Debug)]
pub struct Thread {
    global: Global,
    frames: Vec<Frame>,
    priority: usize,
}

pub enum ThreadResult {
    Continue,
    Stop,
    Result(Primitive),
}

impl Thread {
    pub fn new(global: Global, priority: usize) -> Self {
        Self {
            global,
            frames: Vec::new(),
            priority,
        }
    }

    pub fn new_main(global: Global) -> Result<Self, WasmJVMError> {
        let mut thread = Self {
            global,
            frames: Vec::new(),
            priority: 0,
        };

        thread.new_main_frame()?;

        Ok(thread)
    }

    fn build_frame(
        self: &mut Self,
        mut method_ref: MethodRef,
        this: Option<Primitive>,
        local_variables: Vec<Primitive>,
    ) -> Result<Frame, WasmJVMError> {
        let mut checked_this_class = this.is_none();
        loop {
            let class_index =
                if let Ok(class_index) = self.global.class_index(method_ref.class.as_str()) {
                    class_index
                } else {
                    self.global
                        .loader_mut()?
                        .load_class_name(method_ref.class.as_str())?
                };

            let metadata = self.global.class(class_index)?.metadata();

            if let Ok(method_index) = metadata.method_index(&method_ref) {
                let method = metadata.method(method_index);

                let access_flags = method.access_flags();

                if !access_flags.has_type(&AccessFlagType::Static) {
                    if this.is_none() || this.as_ref().unwrap().is_null() {
                        return Err(WasmJVMError::NullPointerException(format!(
                            "self was null when calling {:?}",
                            method_ref
                        )));
                    }
                }

                if access_flags.has_type(&AccessFlagType::Native) {
                    break;
                }

                if method.attribute(&"Code".to_string()).is_ok() {
                    break;
                }
            }

            if !checked_this_class {
                checked_this_class = true;

                let this_object = self.global.reference_p(this.as_ref().unwrap())?;
                let this_class = self
                    .global
                    .reference(*this_object.class().as_ref().unwrap())?;

                if let RustObject::Class(class) = this_class.inner() {
                    method_ref.class = class.metadata().this_class().to_string();
                } else {
                    unreachable!();
                }

                continue;
            }

            if let Some(super_class) = metadata.super_class().as_ref() {
                method_ref.class = super_class.clone();

                continue;
            }

            return Err(WasmJVMError::NoSuchMethodError(format!("{:?}", method_ref)));
        }

        let max_locals = {
            let (class_index, method_index, descriptor) = self.global.method(&method_ref)?;
            let class = self.global.class_mut(class_index)?;

            let method = class.metadata().method(method_index);

            if method.access_flags().has_type(&AccessFlagType::Native) {
                descriptor.parameters().len() + 1
            } else {
                let attribute = method.attribute(&"Code".to_string())?;
                let code = &attribute.body;

                if let wasmjvm_class::AttributeBody::Code(code) = code {
                    code.max_locals as usize
                } else {
                    return Err(WasmJVMError::TODO(27));
                }
            }
        };

        // TODO: Check if passed locals are valid.
        let mut locals = vec![Primitive::Null; max_locals];

        let mut i = if let Some(this) = this {
            locals[0] = this;
            1
        } else {
            0
        };

        for variable in local_variables {
            locals[i] = variable;
            i += 1;
        }

        Ok(Frame::new(method_ref, locals)?)
    }

    fn new_frame(
        self: &mut Self,
        method_ref: MethodRef,
        this: Option<Primitive>,
        local_variables: Vec<Primitive>,
    ) -> Result<(), WasmJVMError> {
        let frame = self.build_frame(method_ref, this, local_variables)?;

        self.frames.push(frame);

        Ok(())
    }

    fn new_static_frame(
        self: &mut Self,
        method_ref: MethodRef,
        local_variables: Vec<Primitive>,
    ) -> Result<(), WasmJVMError> {
        self.new_frame(method_ref, None, local_variables)
    }

    pub fn new_clinit_frame(self: &mut Self, class: usize) -> Result<(), WasmJVMError> {
        let class = self.global.class_mut(class)?;
        let class_name = class.metadata().this_class().clone();

        let method_ref = MethodRef::new(
            class_name.to_string(),
            "<clinit>".to_string(),
            Descriptor::void(),
        );

        if self.global.method(&method_ref).is_ok() {
            if let Ok(frame) = self.build_frame(method_ref, None, Vec::new()) {
                self.frames.push(frame);
            } else {
                unreachable!()
            }
        }

        Ok(())
    }

    pub fn new_default_init_frame(
        self: &mut Self,
        class: usize,
        this: usize,
    ) -> Result<(), WasmJVMError> {
        let class = self.global.class(class)?;
        let class_name = class.metadata().this_class().clone();

        let method_ref = MethodRef::new(
            class_name.to_string(),
            "<init>".to_string(),
            Descriptor::void(),
        );

        let frame = self.build_frame(method_ref, Some(Primitive::Reference(this)), Vec::new())?;

        self.frames.push(frame);

        Ok(())
    }

    fn new_main_frame(self: &mut Self) -> Result<(), WasmJVMError> {
        let class_index = self.global.main_class_index()?;
        let class = self.global.class(class_index)?;

        let mut method_refs = class.metadata().method_refs(&"main".to_string())?;

        let method_ref = method_refs.pop().unwrap();

        self.new_static_frame(method_ref, Vec::new())?;

        Ok(())
    }

    pub fn stack_trace(self: &mut Self) -> Result<String, WasmJVMError> {
        let frame_count = self.frames.len();

        if frame_count == 0 {
            return Ok(format!("===== Thread (Empty) ======\n"));
        }

        let frame = &self.frames[frame_count - 1];
        let (class_index, method_index, _) = self.global.method(&frame.method_ref)?;
        let class = self.global.class(class_index)?;

        let method = class.metadata().method(method_index);

        let opcode = if !method.access_flags().has_type(&AccessFlagType::Native) {
            let code = {
                let attribute = method.attribute(&"Code".to_string()).unwrap();
                attribute.body.clone()
            };

            if let wasmjvm_class::AttributeBody::Code(body) = code {
                if frame.pc() < body.code.len() {
                    let opcode = OpCode::from_u8(body.code[frame.pc()]).unwrap();
                    format!("OpCode: {:?}\n", opcode)
                } else {
                    format!("End\n")
                }
            } else {
                format!("Invalid\n")
            }
        } else {
            format!("Native\n")
        };

        let frames: Vec<String> = self
            .frames
            .iter()
            .rev()
            .map(|frame| format!("{:?}", frame))
            .collect();
        let frames: String = frames.join("\n");

        let message = format!(
            "===== Thread ======\n{}\n{}\n================\n",
            opcode, frames
        );

        Ok(message)
    }

    pub async fn tick(self: &mut Self) -> Result<ThreadResult, WasmJVMError> {
        let priority = self.global.thread_priority()?;

        if self.priority < priority {
            return Ok(ThreadResult::Continue);
        }

        if self.frames.len() == 0 {
            if priority > 0 && priority == self.priority {
                self.global.thread_unlock(priority)?;
            }

            return Ok(ThreadResult::Stop);
        }

        let frame_count = self.frames.len();
        let frame = &mut self.frames[frame_count - 1];
        let variables = frame.local_variables().clone();

        let mut out_frames: Vec<(MethodRef, Option<Primitive>, Vec<Primitive>)> = Vec::new();
        let mut out_return: Option<Primitive> = None;
        let mut out_throw = false;

        let (class_index, method_index, descriptor) = self.global.method(&frame.method_ref)?;
        let class = self.global.class(class_index)?;
        let method = class.metadata().method(method_index);

        if method.access_flags().has_type(&AccessFlagType::Native) {
            let result = self.global.native_invoke(&frame.method_ref, variables).await?;
            out_return = Some(result.into_type(descriptor.output())?);
        } else {
            let code = {
                let attribute = method.attribute(&"Code".to_string())?;
                attribute.body.clone()
            };

            if let wasmjvm_class::AttributeBody::Code(body) = code {
                let (pc, stack, locals, frame_throw, throw_entries) = frame.all_mut();

                if frame_throw.is_some() && throw_entries.len() == 0 {
                    let throw_reference = frame_throw.as_ref().unwrap();
                    let throw_object = self.global.reference_p(&throw_reference)?;
                    let throw_class = self.global.class(throw_object.class().unwrap())?;
                    let mut class_names = HashSet::new();
                    let mut class_name_queue = vec![throw_class.metadata().this_class()];
                    while class_name_queue.len() > 0 {
                        let class_name = class_name_queue.pop().unwrap();
                        let class_index = self.global.class_index(class_name)?;
                        let class = self.global.class(class_index)?;

                        class_names.insert(class_name);

                        if let Some(super_class) = class.metadata().super_class() {
                            class_name_queue.push(super_class);
                        }
                    }

                    for exception in body.exception_table.iter() {
                        if !(*pc >= exception.start_pc as usize && *pc <= exception.end_pc as usize)
                        {
                            continue;
                        }

                        if exception.catch_type as usize == 0 {
                            throw_entries.push(exception.clone());
                        } else {
                            let exception_constant =
                                class.metadata().constant(exception.catch_type as usize);
                            if let Constant::Class { name } = exception_constant {
                                if class_names.contains(name.as_str()) {
                                    throw_entries.push(exception.clone());
                                }
                            } else {
                                todo!("{:?}", exception_constant);
                            }
                        }
                    }

                    throw_entries.reverse();

                    if throw_entries.len() > 0 {
                        *pc = throw_entries[throw_entries.len() - 1].handler_pc as usize;
                        stack.push(frame_throw.as_ref().unwrap().clone());
                    } else {
                        out_throw = true;
                    }
                }

                if !out_throw {
                    let (mut new_frames, r#return, offset, throw) = Self::code_tick(
                        &mut self.global.clone(),
                        pc,
                        &body.code,
                        stack,
                        locals,
                        class.metadata(),
                    ).await?;

                    if offset >= 0 {
                        *pc += offset as usize;
                    } else {
                        *pc = (*pc as isize + offset) as usize;
                    }

                    out_frames.append(&mut new_frames);
                    if r#return.is_some() && frame_throw.is_none() {
                        if let Some(r#return) = r#return {
                            out_return = Some(r#return.into_type(descriptor.output())?);
                        }
                    } else if r#return.is_some()
                        && frame_throw.is_some()
                        && throw_entries
                            .iter()
                            .any(|entry| *pc == entry.handler_pc as usize)
                    {
                        throw_entries.pop();

                        if throw_entries.len() == 0 {
                            out_throw = true;
                        } else {
                            let exception = &throw_entries[throw_entries.len() - 1];
                            *pc = exception.handler_pc as usize;
                            stack.push(frame_throw.as_ref().unwrap().clone());
                        }
                    }

                    if let Some(throw) = throw {
                        *frame_throw = Some(throw);
                    }
                }
            } else {
                return Err(WasmJVMError::TODO(28));
            }
        }

        if let Some(r#return) = out_return {
            self.frames.pop();

            let frame_count = self.frames.len();
            if frame_count == 0 {
                return Ok(ThreadResult::Result(r#return));
            } else if !r#return.is_void() {
                let frame = &mut self.frames[frame_count - 1];
                frame.operand_stack_mut().push(r#return);
            }
        } else if out_throw {
            let mut frame = self.frames.pop().unwrap();

            if self.frames.len() == 0 {
                return Err(WasmJVMError::UnhandledException(format!(
                    "{:?}",
                    self.global.reference_p(&frame.throw.unwrap()).unwrap()
                )));
            } else {
                let mut next_frame = self.frames.pop().unwrap();
                *next_frame.throw_mut() = Some(frame.throw_mut().as_ref().unwrap().clone());
                self.frames.push(next_frame);
            }
        } else {
            for (method_ref, this, locals) in out_frames {
                self.new_frame(method_ref, this, locals)?;
            }
        }

        Ok(ThreadResult::Continue)
    }

    fn pop_locals(
        descriptor: &Descriptor,
        stack: &mut Vec<Primitive>,
    ) -> Result<Vec<Primitive>, WasmJVMError> {
        let mut locals = Vec::new();

        for r#type in descriptor.parameters().rev() {
            locals.push(stack.pop().unwrap().into_type(r#type)?);
        }
        locals.reverse();

        Ok(locals)
    }

    async fn code_tick(
        global: &mut Global,
        pc: &mut usize,
        code: &Vec<u8>,
        stack: &mut Vec<Primitive>,
        locals: &mut Vec<Primitive>,
        metadata: &wasmjvm_class::Class,
    ) -> Result<
        (
            Vec<(MethodRef, Option<Primitive>, Vec<Primitive>)>,
            Option<Primitive>,
            isize,
            Option<Primitive>,
        ),
        WasmJVMError,
    > {
        let mut frames: Vec<(MethodRef, Option<Primitive>, Vec<Primitive>)> = Vec::new();
        let mut r#return = None;
        let mut throw = None;

        let opcode_raw = code[*pc];
        let opcode = OpCode::from_u8(opcode_raw)?;

        let offset: isize = match opcode {
            OpCode::Nop => 1,
            OpCode::AconstNull => {
                stack.push(Primitive::Null);

                1
            }
            OpCode::IconstM1
            | OpCode::Iconst0
            | OpCode::Iconst1
            | OpCode::Iconst2
            | OpCode::Iconst3
            | OpCode::Iconst4
            | OpCode::Iconst5 => {
                let value = opcode_raw as i32 - OpCode::Iconst0 as i32;

                stack.push(Primitive::Int(value));

                1
            }
            OpCode::Lconst0 | OpCode::Lconst1 => {
                let value = opcode_raw as i64 - OpCode::Lconst0 as i64;

                stack.push(Primitive::Long(value));

                1
            }
            OpCode::Fconst0 | OpCode::Fconst1 | OpCode::Fconst2 => {
                let value = opcode_raw as f32 - OpCode::Fconst0 as u8 as f32;

                stack.push(Primitive::Float(value));

                1
            }
            OpCode::Dconst0 | OpCode::Dconst1 => {
                let value = opcode_raw as f64 - OpCode::Dconst0 as u8 as f64;

                stack.push(Primitive::Double(value));

                1
            }
            OpCode::BiPush => {
                let value = code[*pc + 1];

                stack.push(Primitive::Byte(value));

                2
            }
            OpCode::SiPush => {
                let value = (code[*pc + 1] as u16) << 8 | code[*pc + 2] as u16;

                stack.push(Primitive::Short(value));

                3
            }
            OpCode::Ldc => {
                // TODO: Symbolic resolution.
                let index = code[*pc + 1] as usize;
                let constant = metadata.constant(index);

                match constant {
                    Constant::String(value) => {
                        let string_ref = global.new_java_string(value.clone())?;

                        let reference = Primitive::Reference(string_ref);

                        stack.push(reference.clone());
                    }
                    _ => {
                        let primitive = Primitive::from(constant.clone());

                        stack.push(primitive);
                    }
                }

                2
            }
            OpCode::LdcW | OpCode::Ldc2W => {
                // TODO: Symbolic resolution.
                let index = (code[*pc + 1] as usize) << 8 | code[*pc + 2] as usize;
                let constant = metadata.constant(index);
                let primitive = Primitive::from(constant.clone());

                stack.push(primitive);

                3
            }
            OpCode::Iload => {
                let index = code[*pc + 1] as usize;
                let variable = locals[index].into_int()?;

                stack.push(variable);

                2
            }
            OpCode::Lload => {
                let index = code[*pc + 1] as usize;
                let variable = locals[index].into_long()?;

                stack.push(variable);

                2
            }
            OpCode::Fload => {
                let index = code[*pc + 1] as usize;
                let variable = locals[index].into_float()?;

                stack.push(variable);

                2
            }
            OpCode::Dload => {
                let index = code[*pc + 1] as usize;
                let variable = locals[index].into_double()?;

                stack.push(variable);

                2
            }
            OpCode::Aload => {
                let index = code[*pc + 1] as usize;
                let variable = locals[index].into_ref()?;

                stack.push(variable);

                2
            }
            OpCode::Iload0 | OpCode::Iload1 | OpCode::Iload2 | OpCode::Iload3 => {
                let index = opcode_raw as usize - OpCode::Iload0 as usize;
                let variable = locals[index].into_int()?;

                stack.push(variable);

                1
            }
            OpCode::Lload0 | OpCode::Lload1 | OpCode::Lload2 | OpCode::Lload3 => {
                let index = opcode_raw as usize - OpCode::Lload0 as usize;
                let variable = locals[index].into_long()?;

                stack.push(variable);

                1
            }
            OpCode::Fload0 | OpCode::Fload1 | OpCode::Fload2 | OpCode::Fload3 => {
                let index = opcode_raw as usize - OpCode::Fload0 as usize;
                let variable = locals[index].into_float()?;

                stack.push(variable);

                1
            }
            OpCode::Dload0 | OpCode::Dload1 | OpCode::Dload2 | OpCode::Dload3 => {
                let index = opcode_raw as usize - OpCode::Dload0 as usize;
                let variable = locals[index].into_double()?;

                stack.push(variable);

                1
            }
            OpCode::Aload0 | OpCode::Aload1 | OpCode::Aload2 | OpCode::Aload3 => {
                let index = opcode_raw as usize - OpCode::Aload0 as usize;
                let variable = locals[index].into_ref()?;

                stack.push(variable);

                1
            }
            OpCode::IAload
            | OpCode::LAload
            | OpCode::FAload
            | OpCode::DAload
            | OpCode::AAload
            | OpCode::BAload
            | OpCode::CAload
            | OpCode::SAload => {
                let index = stack.pop().unwrap().into_int()?;
                let reference = stack.pop().unwrap();
                let object = global.reference_p(&reference)?;

                match (object.inner(), index) {
                    (RustObject::Array(array), Primitive::Int(index)) => {
                        if index < 0 {
                            return Err(WasmJVMError::IndexOutOfBoundException(format!(
                                "{} < 0",
                                index
                            )));
                        }

                        if index >= array.len() as i32 {
                            return Err(WasmJVMError::IndexOutOfBoundException(format!(
                                "{} >= {}",
                                index,
                                array.len()
                            )));
                        }

                        let value = array[index as usize].clone();

                        match opcode {
                            OpCode::IAload => value.into_int()?,
                            OpCode::LAload => value.into_long()?,
                            OpCode::FAload => value.into_float()?,
                            OpCode::DAload => value.into_double()?,
                            OpCode::AAload => value.into_ref()?,
                            // TODO: Byte or boolean.
                            OpCode::BAload => value.into_byte()?,
                            OpCode::CAload => value.into_char()?,
                            OpCode::SAload => value.into_short()?,
                            _ => unreachable!(),
                        };

                        stack.push(value);
                    }
                    _ => unreachable!(),
                }

                1
            }
            OpCode::Istore => {
                let index = code[*pc + 1] as usize;
                let last = stack.pop().unwrap();

                locals[index] = last;

                2
            }
            OpCode::Lstore => {
                let index = code[*pc + 1] as usize;
                let last = stack.pop().unwrap();

                locals[index] = last;

                2
            }
            OpCode::Fstore => {
                let index = code[*pc + 1] as usize;
                let last = stack.pop().unwrap();

                locals[index] = last;

                2
            }
            OpCode::Dstore => {
                let index = code[*pc + 1] as usize;
                let last = stack.pop().unwrap();

                locals[index] = last;

                2
            }
            OpCode::Astore => {
                let index = code[*pc + 1] as usize;
                let last = stack.pop().unwrap();

                locals[index] = last;

                2
            }
            OpCode::Istore0 | OpCode::Istore1 | OpCode::Istore2 | OpCode::Istore3 => {
                let index = opcode_raw as usize - OpCode::Istore0 as usize;
                let last = stack.pop().unwrap();

                locals[index] = last;

                1
            }
            OpCode::Lstore0 | OpCode::Lstore1 | OpCode::Lstore2 | OpCode::Lstore3 => {
                let index = opcode_raw as usize - OpCode::Lstore0 as usize;
                let last = stack.pop().unwrap();

                locals[index] = last;

                1
            }
            OpCode::Fstore0 | OpCode::Fstore1 | OpCode::Fstore2 | OpCode::Fstore3 => {
                let index = opcode_raw as usize - OpCode::Fstore0 as usize;
                let last = stack.pop().unwrap();

                locals[index] = last;

                1
            }
            OpCode::Dstore0 | OpCode::Dstore1 | OpCode::Dstore2 | OpCode::Dstore3 => {
                let index = opcode_raw as usize - OpCode::Dstore0 as usize;
                let last = stack.pop().unwrap();

                locals[index] = last;

                1
            }
            OpCode::Astore0 | OpCode::Astore1 | OpCode::Astore2 | OpCode::Astore3 => {
                let index = opcode_raw as usize - OpCode::Astore0 as usize;
                let last = stack.pop().unwrap();

                locals[index] = last;

                1
            }
            OpCode::IAstore
            | OpCode::FAstore
            | OpCode::LAstore
            | OpCode::DAstore
            | OpCode::AAstore
            | OpCode::CAstore
            | OpCode::BAstore
            | OpCode::SAstore => {
                let value = stack.pop().unwrap();
                let index = stack.pop().unwrap().into_int()?;
                let reference = stack.pop().unwrap();

                global.array_set(reference, index, value)?;

                1
            }
            OpCode::Pop => {
                stack.pop().unwrap();

                1
            }
            OpCode::Pop2 => {
                stack.pop().unwrap();
                stack.pop().unwrap();

                1
            }
            OpCode::Dup => {
                let value = stack.pop().unwrap();

                stack.push(value.clone());
                stack.push(value);

                1
            }
            OpCode::DupX1 => {
                // TODO: Check for category 2.
                let value1 = stack.pop().unwrap();
                let value2 = stack.pop().unwrap();

                if true {
                    stack.push(value1.clone());
                    stack.push(value2);
                    stack.push(value1);
                } else {
                    let value3 = stack.pop().unwrap();

                    stack.push(value2.clone());
                    stack.push(value1.clone());
                    stack.push(value3);
                    stack.push(value2);
                    stack.push(value1);
                }

                1
            }
            OpCode::DupX2 => todo!(),
            OpCode::Dup2 => {
                let v = stack.pop().unwrap();

                match &v {
                    Primitive::Long(..) | Primitive::Double(..) => {
                        // TODO: Check if valid.
                        stack.push(v.clone());
                        stack.push(v);
                    }
                    _ => {
                        let v2 = stack.pop().unwrap();

                        stack.push(v2.clone());
                        stack.push(v.clone());
                        stack.push(v2);
                        stack.push(v);
                    }
                }

                1
            }
            OpCode::Dup2X1 => todo!(),
            OpCode::Dup2X2 => todo!(),
            OpCode::Swap => todo!(),
            OpCode::Iadd => {
                let right = stack.pop().unwrap();
                let left = stack.pop().unwrap();

                stack.push(left.into_int()?.add(&right.into_int()?)?);

                1
            }
            OpCode::Ladd => {
                let right = stack.pop().unwrap();
                let left = stack.pop().unwrap();

                stack.push(left.into_long()?.add(&right.into_long()?)?);

                1
            }
            OpCode::Fadd => {
                let right = stack.pop().unwrap();
                let left = stack.pop().unwrap();

                stack.push(left.into_float()?.add(&right.into_float()?)?);

                1
            }
            OpCode::Dadd => {
                let right = stack.pop().unwrap();
                let left = stack.pop().unwrap();

                stack.push(left.into_double()?.add(&right.into_double()?)?);

                1
            }
            OpCode::Isub => {
                let right = stack.pop().unwrap();
                let left = stack.pop().unwrap();

                stack.push(left.into_int()?.sub(&right.into_int()?)?);

                1
            }
            OpCode::Lsub => {
                let right = stack.pop().unwrap();
                let left = stack.pop().unwrap();

                stack.push(left.into_long()?.sub(&right.into_long()?)?);

                1
            }
            OpCode::Fsub => {
                let right = stack.pop().unwrap();
                let left = stack.pop().unwrap();

                stack.push(left.into_float()?.sub(&right.into_float()?)?);

                1
            }
            OpCode::Dsub => {
                let right = stack.pop().unwrap();
                let left = stack.pop().unwrap();

                stack.push(left.into_double()?.sub(&right.into_double()?)?);

                1
            }
            OpCode::Imul => {
                let right = stack.pop().unwrap();
                let left = stack.pop().unwrap();

                stack.push(left.into_int()?.mul(&right.into_int()?)?);

                1
            }
            OpCode::Lmul => {
                let right = stack.pop().unwrap();
                let left = stack.pop().unwrap();

                stack.push(left.into_long()?.mul(&right.into_long()?)?);

                1
            }
            OpCode::Fmul => {
                let right = stack.pop().unwrap();
                let left = stack.pop().unwrap();

                stack.push(left.into_float()?.mul(&right.into_float()?)?);

                1
            }
            OpCode::Dmul => {
                let right = stack.pop().unwrap();
                let left = stack.pop().unwrap();

                stack.push(left.into_double()?.mul(&right.into_double()?)?);

                1
            }
            OpCode::Idiv => {
                let right = stack.pop().unwrap();
                let left = stack.pop().unwrap();

                stack.push(left.into_int()?.div(&right.into_int()?)?);

                1
            }
            OpCode::Ldiv => {
                let right = stack.pop().unwrap();
                let left = stack.pop().unwrap();

                stack.push(left.into_long()?.div(&right.into_long()?)?);

                1
            }
            OpCode::Fdiv => {
                let right = stack.pop().unwrap();
                let left = stack.pop().unwrap();

                stack.push(left.into_float()?.div(&right.into_float()?)?);

                1
            }
            OpCode::Ddiv => {
                let right = stack.pop().unwrap();
                let left = stack.pop().unwrap();

                stack.push(left.into_double()?.div(&right.into_double()?)?);

                1
            }
            OpCode::Irem => {
                let right = stack.pop().unwrap();
                let left = stack.pop().unwrap();

                stack.push(left.into_int()?.rem(&right.into_int()?)?);

                1
            }
            OpCode::Lrem => {
                let right = stack.pop().unwrap();
                let left = stack.pop().unwrap();

                stack.push(left.into_long()?.rem(&right.into_long()?)?);

                1
            }
            OpCode::Frem => {
                let right = stack.pop().unwrap();
                let left = stack.pop().unwrap();

                stack.push(left.into_float()?.rem(&right.into_float()?)?);

                1
            }
            OpCode::Drem => {
                let right = stack.pop().unwrap();
                let left = stack.pop().unwrap();

                stack.push(left.into_double()?.rem(&right.into_double()?)?);

                1
            }
            OpCode::Ineg | OpCode::Lneg | OpCode::Fneg | OpCode::Dneg => {
                let value = stack.pop().unwrap();

                stack.push(value.neg()?);

                1
            }
            OpCode::Ishl => {
                let right = stack.pop().unwrap();
                let left = stack.pop().unwrap();

                stack.push(left.into_int()?.shl(&right.into_int()?)?);

                1
            }
            OpCode::Lshl => {
                let right = stack.pop().unwrap();
                let left = stack.pop().unwrap();

                stack.push(left.into_long()?.shl(&right.into_long()?)?);

                1
            }
            OpCode::Ishr => {
                let right = stack.pop().unwrap();
                let left = stack.pop().unwrap();

                stack.push(left.into_int()?.shr(&right.into_int()?)?);

                1
            }
            OpCode::Lshr => {
                let right = stack.pop().unwrap();
                let left = stack.pop().unwrap();

                stack.push(left.into_long()?.shr(&right.into_long()?)?);

                1
            }
            OpCode::Iushr => {
                let right = stack.pop().unwrap();
                let left = stack.pop().unwrap();

                stack.push(left.into_int()?.ushr(&right.into_int()?)?);

                1
            }
            OpCode::Lushr => {
                let right = stack.pop().unwrap();
                let left = stack.pop().unwrap();

                stack.push(left.into_long()?.ushr(&right.into_long()?)?);

                1
            }
            OpCode::Iand => {
                let right = stack.pop().unwrap();
                let left = stack.pop().unwrap();

                stack.push(left.into_int()?.and(&right.into_int()?)?);

                1
            }
            OpCode::Land => {
                let right = stack.pop().unwrap();
                let left = stack.pop().unwrap();

                stack.push(left.into_long()?.and(&right.into_long()?)?);

                1
            }
            OpCode::Ior => {
                let right = stack.pop().unwrap();
                let left = stack.pop().unwrap();

                stack.push(left.into_int()?.or(&right.into_int()?)?);

                1
            }
            OpCode::Lor => {
                let right = stack.pop().unwrap();
                let left = stack.pop().unwrap();

                stack.push(left.into_long()?.or(&right.into_long()?)?);

                1
            }
            OpCode::Ixor => {
                let right = stack.pop().unwrap();
                let left = stack.pop().unwrap();

                stack.push(left.into_int()?.xor(&right.into_int()?)?);

                1
            }
            OpCode::Lxor => {
                let right = stack.pop().unwrap();
                let left = stack.pop().unwrap();

                stack.push(left.into_long()?.xor(&right.into_long()?)?);

                1
            }
            OpCode::Iinc => {
                let index = code[*pc + 1];
                let r#const = code[*pc + 2] as i8;
                let local = locals.get_mut(index as usize).unwrap();

                if let Primitive::Int(raw) = local.into_int()? {
                    *local = Primitive::Int(raw + r#const as i32);
                } else {
                    return Err(WasmJVMError::TODO(29));
                }

                3
            }
            OpCode::I2l | OpCode::I2f | OpCode::I2d => {
                let value = stack.pop().unwrap();

                stack.push(value.into_int()?);

                1
            }
            OpCode::L2i | OpCode::L2f | OpCode::L2d => {
                let value = stack.pop().unwrap();

                stack.push(value.into_long()?);

                1
            }
            OpCode::F2i | OpCode::F2l | OpCode::F2d => {
                let value = stack.pop().unwrap();

                stack.push(value.into_float()?);

                1
            }
            OpCode::D2i | OpCode::D2l | OpCode::D2f => {
                let value = stack.pop().unwrap();

                stack.push(value.into_double()?);

                1
            }
            OpCode::I2b => {
                let value = stack.pop().unwrap();

                stack.push(value.into_byte()?);

                1
            }
            OpCode::I2c => {
                let value = stack.pop().unwrap();

                stack.push(value.into_char()?);

                1
            }
            OpCode::I2s => {
                let value = stack.pop().unwrap();

                stack.push(value.into_short()?);

                1
            }
            OpCode::Lcmp => {
                let right = stack.pop().unwrap();
                let left = stack.pop().unwrap();

                stack.push(left.into_long()?.cmp(&right.into_long()?)?);

                1
            }
            OpCode::Fcmpl => {
                let right = stack.pop().unwrap();
                let left = stack.pop().unwrap();

                stack.push(left.into_float()?.cmpl(&right.into_float()?)?);

                1
            }
            OpCode::Fcmpg => {
                let right = stack.pop().unwrap();
                let left = stack.pop().unwrap();

                stack.push(left.into_float()?.cmpg(&right.into_float()?)?);

                1
            }
            OpCode::Dcmpl => {
                let right = stack.pop().unwrap();
                let left = stack.pop().unwrap();

                stack.push(left.into_double()?.cmpl(&right.into_double()?)?);

                1
            }
            OpCode::Dcmpg => {
                let right = stack.pop().unwrap();
                let left = stack.pop().unwrap();

                stack.push(left.into_double()?.cmpg(&right.into_double()?)?);

                1
            }
            OpCode::Ifeq
            | OpCode::Ifne
            | OpCode::Iflt
            | OpCode::Ifge
            | OpCode::Ifgt
            | OpCode::Ifle => {
                let b1 = code[*pc + 1] as u16;
                let b2 = code[*pc + 2] as u16;
                let branch = (b1 << 8 | b2) as i16;

                let value = stack.pop().unwrap().into_int()?;
                let int = if let Primitive::Int(value) = value {
                    value
                } else {
                    return Err(WasmJVMError::TODO(30));
                };

                // TODO: Check if correct.
                let condition = match opcode {
                    OpCode::Ifeq => int == 0,
                    OpCode::Ifne => int != 0,
                    OpCode::Iflt => int < 0,
                    OpCode::Ifle => int <= 0,
                    OpCode::Ifgt => int > 0,
                    OpCode::Ifge => int >= 0,
                    _ => unreachable!(),
                };

                if condition {
                    branch as isize
                } else {
                    3
                }
            }
            OpCode::IfNull | OpCode::IfNonNull => {
                let b1 = code[*pc + 1] as u16;
                let b2 = code[*pc + 2] as u16;
                let branch = (b1 << 8 | b2) as i16;

                let value = stack.pop().unwrap();
                // TODO: Check if correct.
                let condition = (opcode_raw == OpCode::IfNull as u8) == value.is_null();

                if condition {
                    branch as isize
                } else {
                    3
                }
            }
            OpCode::IfIcmpeq
            | OpCode::IfIcmpne
            | OpCode::IfIcmplt
            | OpCode::IfIcmpge
            | OpCode::IfIcmpgt
            | OpCode::IfIcmple => {
                let b1 = code[*pc + 1] as u16;
                let b2 = code[*pc + 2] as u16;
                let branch = (b1 << 8 | b2) as i16;

                let right = stack.pop().unwrap();
                let left = stack.pop().unwrap();

                if let Primitive::Int(cmp) = left.into_int()?.cmp(&right.into_int()?)? {
                    let condition: bool = match opcode {
                        OpCode::IfIcmpeq => cmp == 0,
                        OpCode::IfIcmpne => cmp != 0,
                        OpCode::IfIcmple => cmp <= 0,
                        OpCode::IfIcmplt => cmp < 0,
                        OpCode::IfIcmpge => cmp >= 0,
                        OpCode::IfIcmpgt => cmp > 0,
                        _ => unreachable!(),
                    };

                    if condition {
                        branch as isize
                    } else {
                        3
                    }
                } else {
                    unreachable!();
                }
            }
            OpCode::IfAcmpeq | OpCode::IfAcmpne => {
                let b1 = code[*pc + 1] as u16;
                let b2 = code[*pc + 2] as u16;
                let branch = (b1 << 8 | b2) as i16;

                let right = stack.pop().unwrap();
                let left = stack.pop().unwrap();

                if let Primitive::Int(cmp) = left.cmp(&right)? {
                    let condition: bool = match opcode {
                        OpCode::IfAcmpeq => cmp == 0,
                        OpCode::IfAcmpne => cmp != 0,
                        _ => unreachable!(),
                    };

                    if condition {
                        branch as isize
                    } else {
                        3
                    }
                } else {
                    unreachable!();
                }
            }
            OpCode::Goto => {
                let b1 = code[*pc + 1] as u16;
                let b2 = code[*pc + 2] as u16;
                let branch = (b1 << 8 | b2) as i16;

                branch as isize
            }
            OpCode::Jsr => todo!(),
            OpCode::Ret => todo!(),
            OpCode::Tableswitch => {
                let offset = if (*pc + 1) % 4 == 0 { 1 } else { 5 - ((*pc + 1) % 4) };

                let defaultbyte1 = code[*pc + offset] as u32;
                let defaultbyte2 = code[*pc + 1 + offset] as u32;
                let defaultbyte3 = code[*pc + 2 + offset] as u32;
                let defaultbyte4 = code[*pc + 3 + offset] as u32;
                let default = ((defaultbyte1 << 24) | (defaultbyte2 << 16) | (defaultbyte3 << 8) | defaultbyte4) as i32;

                let lowbyte1 = code[*pc + 4 + offset] as u32;
                let lowbyte2 = code[*pc + 5 + offset] as u32;
                let lowbyte3 = code[*pc + 6 + offset] as u32;
                let lowbyte4 = code[*pc + 7 + offset] as u32;
                let low = ((lowbyte1 << 24) | (lowbyte2 << 16) | (lowbyte3 << 8) | lowbyte4) as i32;

                let highbyte1 = code[*pc + 8 + offset] as u32;
                let highbyte2 = code[*pc + 9 + offset] as u32;
                let highbyte3 = code[*pc + 10 + offset] as u32;
                let highbyte4 = code[*pc + 11 + offset] as u32;
                let high = ((highbyte1 << 24) | (highbyte2 << 16) | (highbyte3 << 8) | highbyte4) as i32;

                if let Primitive::Int(index) = stack.pop().unwrap().into_int()? {
                    let offset = if index < low || index > high {
                        default as isize
                    } else {
                        let jumpindex = index as isize * 4;

                        let jumpbyte1 = code[((*pc + 12) as isize + offset as isize + jumpindex) as usize] as u32;
                        let jumpbyte2 = code[((*pc + 13) as isize + offset as isize + jumpindex) as usize] as u32;
                        let jumpbyte3 = code[((*pc + 14) as isize + offset as isize + jumpindex) as usize] as u32;
                        let jumpbyte4 = code[((*pc + 15) as isize + offset as isize + jumpindex) as usize] as u32;
                        let jump = ((jumpbyte1 << 24) | (jumpbyte2 << 16) | (jumpbyte3 << 8) | jumpbyte4) as i32;

                        jump as isize
                    };

                    offset
                } else {
                    unreachable!();
                }
            },
            OpCode::Lookupswitch => {
                let offset = if (*pc + 1) % 4 == 0 { 1 } else { 5 - ((*pc + 1) % 4) };

                let defaultbyte1 = code[*pc + offset] as u32;
                let defaultbyte2 = code[*pc + 1 + offset] as u32;
                let defaultbyte3 = code[*pc + 2 + offset] as u32;
                let defaultbyte4 = code[*pc + 3 + offset] as u32;
                let default = ((defaultbyte1 << 24) | (defaultbyte2 << 16) | (defaultbyte3 << 8) | defaultbyte4) as i32;

                let npairsbyte1 = code[*pc + 4 + offset] as u32;
                let npairsbyte2 = code[*pc + 5 + offset] as u32;
                let npairsbyte3 = code[*pc + 6 + offset] as u32;
                let npairsbyte4 = code[*pc + 7 + offset] as u32;
                let npairs = ((npairsbyte1 << 24) | (npairsbyte2 << 16) | (npairsbyte3 << 8) | npairsbyte4) as i32;

                if let Primitive::Int(key) = stack.pop().unwrap().into_int()? {
                    // TODO: Binary search

                    let mut jump_offset = None;
                    for i in 0..npairs {
                        let intbyte1 = code[*pc + 8 + offset + i as usize * 8] as u32;
                        let intbyte2 = code[*pc + 9 + offset + i as usize * 8] as u32;
                        let intbyte3 = code[*pc + 10 + offset + i as usize * 8] as u32;
                        let intbyte4 = code[*pc + 11 + offset + i as usize * 8] as u32;
                        let int = ((intbyte1 << 24) | (intbyte2 << 16) | (intbyte3 << 8) | intbyte4) as i32;

                        if int == key {
                            let jump1 = code[*pc + 12 + offset + i as usize * 8] as u32;
                            let jump2 = code[*pc + 13 + offset + i as usize * 8] as u32;
                            let jump3 = code[*pc + 14 + offset + i as usize * 8] as u32;
                            let jump4 = code[*pc + 15 + offset + i as usize * 8] as u32;
                            let jump = ((jump1 << 24) | (jump2 << 16) | (jump3 << 8) | jump4) as i32;

                            jump_offset = Some(jump as isize);
                            break;
                        }
                    }

                    if let Some(jump_offset) = jump_offset {
                        jump_offset
                    } else {
                        default as isize
                    }
                } else {
                    unreachable!();
                }
            },
            OpCode::Ireturn
            | OpCode::Lreturn
            | OpCode::Freturn
            | OpCode::Dreturn
            | OpCode::Areturn => {
                let value = stack.pop().unwrap();

                r#return = Some(match opcode {
                    OpCode::Ireturn => value.into_int()?,
                    OpCode::Lreturn => value.into_long()?,
                    OpCode::Freturn => value.into_float()?,
                    OpCode::Dreturn => value.into_double()?,
                    OpCode::Areturn => value.into_ref()?,
                    _ => unreachable!(),
                });

                1
            }
            OpCode::Return => {
                r#return = Some(Primitive::Void);

                1
            }
            OpCode::GetStatic => {
                let i1 = code[*pc + 1] as u16;
                let i2 = code[*pc + 2] as u16;
                let index = (i1 << 8 | i2) as usize;

                let field_ref = metadata.constant(index);

                if let Constant::FieldRef(field_ref) = field_ref {
                    if let Ok(field) = global.static_field(field_ref) {
                        stack.push(field);

                        3
                    } else {
                        0
                    }
                } else {
                    return Err(WasmJVMError::TODO(32));
                }
            }
            OpCode::PutStatic => {
                let i1 = code[*pc + 1] as u16;
                let i2 = code[*pc + 2] as u16;
                let index = (i1 << 8 | i2) as usize;

                let field_ref = metadata.constant(index);

                if let Constant::FieldRef(field_ref) = field_ref {
                    let value = stack.pop().unwrap();

                    global.static_field_set(field_ref, value)?;
                } else {
                    return Err(WasmJVMError::TODO(33));
                }

                3
            }
            OpCode::GetField => {
                let i1 = code[*pc + 1] as u16;
                let i2 = code[*pc + 2] as u16;
                let index = (i1 << 8 | i2) as usize;

                let field_ref = metadata.constant(index);
                let reference = stack.pop().unwrap();
                let object = global.reference_p(&reference)?;

                if let Constant::FieldRef(field_ref) = field_ref {
                    let field = object.fields.get(&field_ref.name);

                    if let Some(field) = field {
                        stack.push(field.clone());
                    } else {
                        return Err(WasmJVMError::TODO(34));
                    }
                } else {
                    return Err(WasmJVMError::TODO(35));
                }

                3
            }
            OpCode::PutField => {
                let i1 = code[*pc + 1] as u16;
                let i2 = code[*pc + 2] as u16;
                let index = (i1 << 8 | i2) as usize;

                let field_ref = metadata.constant(index);
                let value = stack.pop().unwrap();
                let object_ref = stack.pop().unwrap();

                if let Constant::FieldRef(field_ref) = field_ref {
                    global.field_set(object_ref, field_ref, value)?;
                } else {
                    return Err(WasmJVMError::TODO(36));
                }

                3
            }
            OpCode::InvokeSpecial | OpCode::InvokeVirtual => {
                // TODO: Check InvokeVirtual

                let i1 = code[*pc + 1] as u16;
                let i2 = code[*pc + 2] as u16;
                let index = (i1 << 8 | i2) as usize;

                let method_ref = metadata.constant(index);

                if let Constant::MethodRef(method_ref) = method_ref {
                    let locals = Self::pop_locals(&method_ref.descriptor, stack)?;
                    let this = stack.pop().unwrap();

                    frames.push((method_ref.clone(), Some(this), locals));
                } else {
                    panic!("Expecting a method.");
                }

                3
            }
            OpCode::InvokeStatic => {
                let i1 = code[*pc + 1] as u16;
                let i2 = code[*pc + 2] as u16;
                let index = (i1 << 8 | i2) as usize;

                let constant = metadata.constant(index);

                if let Constant::MethodRef(method_ref) = constant {
                    let locals = Self::pop_locals(&method_ref.descriptor, stack)?;

                    frames.push((method_ref.clone(), None, locals));
                } else {
                    panic!("Expecting a method.");
                }

                3
            }
            OpCode::InvokeInterface => {
                let i1 = code[*pc + 1] as u16;
                let i2 = code[*pc + 2] as u16;
                let index = (i1 << 8 | i2) as usize;
                let _count = code[*pc + 3] as u8;
                let _zero = code[*pc + 4] as u8;

                let constant = metadata.constant(index);

                if let Constant::InterfaceMethodRef(method_ref) = constant {
                    let locals = Self::pop_locals(&method_ref.descriptor, stack)?;
                    let this = stack.pop().unwrap();

                    frames.push((method_ref.clone(), Some(this), locals));
                } else {
                    panic!("Expected interface method.");
                }

                5
            }
            OpCode::InvokeDynamic => todo!(),
            OpCode::New => {
                let i1 = code[*pc + 1] as u16;
                let i2 = code[*pc + 2] as u16;
                let index = (i1 << 8 | i2) as usize;

                let class_ref = metadata.constant(index);

                let instance_ref = if let Constant::Class { name, .. } = &class_ref {
                    global.new_instance(name)?
                } else {
                    panic!("Expecting a class.");
                };

                stack.push(Primitive::Reference(instance_ref));

                3
            }
            OpCode::NewArray => {
                // TODO: Check constant pool.
                let _type = code[*pc + 1] as usize;
                let count = stack.pop().unwrap().into_int()?;

                if let Primitive::Int(count) = count {
                    let index = global.new_object(Object::new_empty_array(count as usize)?)?;
                    stack.push(Primitive::Reference(index));
                } else {
                    return Err(WasmJVMError::TODO(37));
                }

                2
            }
            OpCode::ANewArray => {
                // TODO: Check constant pool.
                let _index = (code[*pc + 1] as usize) << 8 | code[*pc + 2] as usize;
                let count = stack.pop().unwrap().into_int()?;

                if let Primitive::Int(count) = count {
                    let index = global.new_object(Object::new_empty_array(count as usize)?)?;
                    stack.push(Primitive::Reference(index));
                } else {
                    return Err(WasmJVMError::TODO(38));
                }

                3
            }
            OpCode::ArrayLength => {
                let array_ref = stack.pop().unwrap();
                let object = global.reference_p(&array_ref)?;

                if let RustObject::Array(raw) = object.inner() {
                    stack.push(Primitive::Int(raw.len() as i32));
                } else {
                    return Err(WasmJVMError::TODO(39));
                }

                1
            }
            OpCode::Athrow => {
                let object_ref = stack.pop().unwrap();

                throw = Some(object_ref);

                1
            }
            OpCode::Instanceof | OpCode::CheckCast => {
                let i1 = code[*pc + 1] as u16;
                let i2 = code[*pc + 2] as u16;
                let index = (i1 << 8 | i2) as usize;
                let class_ref = metadata.constant(index);
                let object_ref = stack.pop().unwrap();

                let mut instanceof = true;
                if let Constant::Class { name } = class_ref {
                    let r#type: Type;
                    if name.starts_with("[") {
                        let descriptor = Descriptor::from_str(name)?;
                        r#type = descriptor.output().clone();
                    } else {
                        r#type = Type::Single(SingleType::Object(name.clone()));
                    }

                    let single_type: SingleType;
                    let array_size: usize;
                    match r#type {
                        Type::Single(r#type) => {
                            single_type = r#type;
                            array_size = 0;
                        }
                        Type::Array(r#type, size) => {
                            single_type = r#type;
                            array_size = size;
                        }
                    }

                    let class_type: String;
                    if let SingleType::Object(class) = single_type {
                        class_type = class;
                    } else {
                        todo!("{:?}", name);
                    }

                    let mut object_array_size = 0;
                    let mut array_queue: Vec<&Primitive> = vec![&object_ref];
                    let mut next_array_queue: Vec<&Primitive> = Vec::new();

                    while array_queue.len() > 0 {
                        while array_queue.len() > 0 {
                            let next_primitive = array_queue.pop().unwrap();

                            match next_primitive {
                                Primitive::Reference(index) => {
                                    let next_object = global.reference(*index)?;

                                    match next_object.inner() {
                                        RustObject::Array(primitives) => {
                                            for primitive in primitives.iter() {
                                                next_array_queue.push(primitive);
                                            }
                                        }
                                        _ => {
                                            let mut class_index = next_object.class().unwrap();

                                            loop {
                                                let class = global.class(class_index)?;
                                                let class_metadata = class.metadata();

                                                if class_metadata.this_class() == &class_type {
                                                    break;
                                                }

                                                if class_metadata.interface(&class_type).is_ok() {
                                                    break;
                                                }

                                                if let Some(super_class) =
                                                    class_metadata.super_class()
                                                {
                                                    if super_class == &class_type {
                                                        break;
                                                    }

                                                    class_index =
                                                        global.class_index(super_class)?;
                                                } else {
                                                    instanceof = false;
                                                    break;
                                                }
                                            }
                                        }
                                    }
                                }
                                _ => instanceof = false,
                            };

                            if !instanceof {
                                break;
                            }
                        }

                        if !instanceof || next_array_queue.len() == 0 {
                            break;
                        }

                        object_array_size += 1;
                        if object_array_size > array_size {
                            instanceof = false;
                            break;
                        }

                        array_queue.append(&mut next_array_queue);
                    }
                } else {
                    return Err(WasmJVMError::TODO(40));
                }

                match opcode {
                    OpCode::Instanceof => {
                        stack.push(Primitive::Int(instanceof as i32));
                    }
                    OpCode::CheckCast => {
                        if instanceof {
                            stack.push(object_ref);
                        } else {
                            // TODO: Better output.
                            return Err(WasmJVMError::ClassCastException(String::new()));
                        }
                    }
                    _ => unreachable!(),
                }

                3
            }
            OpCode::MonitorEnter => todo!(),
            OpCode::MonitorExit => todo!(),
            OpCode::Wide => todo!(),
            OpCode::MultiANewArray => {
                // TODO: Check constant pool.
                let _index = (code[*pc + 1] as usize) << 8 | code[*pc + 2] as usize;
                let dimensions = code[*pc + 3] as u8;
                let mut counts = Vec::new();

                for _ in 0..dimensions {
                    if let Primitive::Int(int) = stack.pop().unwrap() {
                        counts.push(int as usize);
                    } else {
                        todo!();
                    }
                }
                counts.reverse();

                stack.push(Object::new_deep_array(global, &counts, 0)?);

                4
            }
            OpCode::GotoW => todo!(),
            OpCode::JsrW => todo!(),
            OpCode::Breakpoint => todo!(),
            OpCode::Impdep1 => todo!(),
            OpCode::Impdep2 => todo!(),
        };

        Ok((frames, r#return, offset, throw))
    }
}
