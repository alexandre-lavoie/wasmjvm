use std::collections::HashMap;

use wasmjvm_class::{Class, WithFields};
use wasmjvm_common::WasmJVMError;

use crate::{
    ClassInstance, Global, Object, RustObject, Thread, JAVA_CLASS, JAVA_LOADER, JAVA_OBJECT,
    JAVA_THREAD,
};

#[derive(Debug)]
pub struct Loader {
    global: Global,
    clinit_thread: usize,
    init_thread: usize,
}

impl Loader {
    pub fn threads(self: &Self) -> (usize, usize) {
        (self.clinit_thread, self.init_thread)
    }

    pub fn new(global: Global, cores: Vec<Class>) -> Result<Loader, WasmJVMError> {
        let mut loader = Loader {
            global,
            clinit_thread: 0,
            init_thread: 0,
        };

        let mut core_map = HashMap::new();

        for core in cores {
            core_map.insert(core.this_class().clone(), core);
        }

        loader.inner_init(core_map)?;

        Ok(loader)
    }

    fn inner_init(self: &mut Self, mut cores: HashMap<String, Class>) -> Result<(), WasmJVMError> {
        if !(cores.contains_key(&JAVA_OBJECT.to_string())
            && cores.contains_key(&JAVA_CLASS.to_string())
            && cores.contains_key(&JAVA_LOADER.to_string())
            && cores.contains_key(&JAVA_THREAD.to_string()))
        {
            return Err(WasmJVMError::LinkageError(format!(
                "Missing core classes {:?}",
                cores
            )));
        }

        let object_class = cores.remove(&JAVA_OBJECT.to_string()).unwrap();
        let object_fields = object_class.field_names();

        let mut clinits = Vec::new();
        let mut inits = Vec::new();

        let object = Object::new(
            self.global.index(),
            object_fields.clone(),
            RustObject::Class(ClassInstance::new(object_class)),
        )?;
        let object_index = self.global.new_object(object)?;
        clinits.push(object_index);
        inits.push((object_index, object_index));

        let class_class = cores.remove(&JAVA_CLASS.to_string()).unwrap();
        let mut class_fields = class_class.field_names();
        class_fields.append(&mut object_fields.clone());

        let class = Object::new(
            object_index.clone(),
            object_fields,
            RustObject::Class(ClassInstance::new(class_class)),
        )?;
        let class_index = self.global.new_object(class)?;
        clinits.push(class_index);
        inits.push((object_index, class_index));

        let loader_class = cores.remove(&JAVA_LOADER.to_string()).unwrap();
        let loader = Object::new(
            class_index.clone(),
            class_fields.clone(),
            RustObject::Class(ClassInstance::new(loader_class)),
        )?;
        let loader_index = self.global.new_object(loader)?;
        clinits.push(loader_index);
        inits.push((class_index, loader_index));

        let thread_class = cores.remove(&JAVA_THREAD.to_string()).unwrap();
        let thread_class = Object::new(
            class_index.clone(),
            class_fields.clone(),
            RustObject::Class(ClassInstance::new(thread_class)),
        )?;
        let thread_class_index = self.global.new_object(thread_class)?;
        clinits.push(thread_class_index);
        inits.push((class_index, thread_class_index));

        inits.push((thread_class_index, inits.len()));
        inits.push((thread_class_index, inits.len() + 1));

        let thread_fields = self.global.resolve_fields(thread_class_index)?;

        let mut init_thread = Thread::new(self.global.clone());
        for (class, this) in inits.iter().rev() {
            init_thread.new_default_init_frame(*class, *this)?;
        }
        let init_thread = Object::new(
            thread_class_index,
            thread_fields.clone(),
            RustObject::Thread(init_thread),
        )?;
        self.init_thread = self.global.new_object(init_thread)?;

        let mut clinit_thread = Thread::new(self.global.clone());
        for class in clinits.iter().rev() {
            clinit_thread.new_clinit_frame(*class)?;
        }
        let clinit_thread = Object::new(
            thread_class_index,
            thread_fields,
            RustObject::Thread(clinit_thread),
        )?;
        self.clinit_thread = self.global.new_object(clinit_thread)?;

        Ok(())
    }

    pub fn clinit(self: &mut Self, class: usize) -> Result<(), WasmJVMError> {
        let thread = self.global.thread_mut(self.clinit_thread)?;

        thread.new_clinit_frame(class)?;

        Ok(())
    }

    pub fn default_init(self: &mut Self, class: usize, this: usize) -> Result<(), WasmJVMError> {
        let thread = self.global.thread_mut(self.init_thread)?;

        thread.new_default_init_frame(class, this)?;

        Ok(())
    }

    pub fn new_class(self: &mut Self, metadata: Class) -> Result<usize, WasmJVMError> {
        let class_index = self.global.class_index(&JAVA_CLASS.to_string())?;

        let inner = RustObject::Class(ClassInstance::new(metadata));
        let class = Object::new(class_index, self.global.resolve_fields(class_index)?, inner)?;

        let object_index = self.global.new_object(class)?;

        self.clinit(object_index)?;
        self.default_init(class_index, object_index)?;

        Ok(object_index)
    }

    pub fn load_class(self: &mut Self, class: Class) -> Result<usize, WasmJVMError> {
        self.new_class(class)
    }
}
