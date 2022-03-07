use std::{
    sync::{Arc, Mutex, MutexGuard},
};

use wasmjvm_class::{Object, ClassInstance, Primitive};
use wasmjvm_common::WasmJVMError;
use wasmjvm_native::Global;

use crate::{Thread, ThreadResult};

pub struct VM {
    global: Arc<Mutex<Global>>,
    threads: Vec<Thread>,
    loader: Thread,
}

impl VM {
    pub fn new() -> Self {
        let global = Arc::new(Mutex::new(Global::default()));
        let loader = Thread::new(&global);

        Self {
            threads: Vec::new(),
            global,
            loader,
        }
    }

    pub fn global_mut(self: &mut Self) -> MutexGuard<Global> {
        self.global.lock().unwrap()
    }

    fn new_thread(self: &mut Self) -> usize {
        let index = self.threads.len();
        let thread = Thread::new(&self.global);
        self.threads.push(thread);
        index
    }

    pub fn load_class_file_path(self: &mut Self, path: &String) -> Result<(), WasmJVMError> {
        let class = wasmjvm_class::Class::from_file(path)?;

        self.load_class(class)?;

        Ok(())
    }

    pub fn load_class(self: &mut Self, class: wasmjvm_class::Class) -> Result<(), WasmJVMError> {
        let class_name = class.this_class().clone();
        let object = Object::Class(ClassInstance::new(class));

        match self.global.lock() {
            Ok(mut global) => {
                global.alloc(object)?;
            }
            Err(_) => return Err(WasmJVMError::ClassInvalid),
        }

        self.loader.new_clinit_frame(&class_name);

        Ok(())
    }

    pub fn run(self: &mut Self) -> Result<Primitive, WasmJVMError> {
        let main_index = self.new_thread();
        self.threads[main_index].init_main()?;

        let mut result = Primitive::Void;

        let mut index = 0;
        while self.threads.len() > 0 {
            match self.loader.tick() {
                Ok(ThreadResult::Continue) => continue,
                Ok(ThreadResult::Result(..)) => continue,
                Ok(ThreadResult::Stop) => {},
                Err(err) => {
                    panic!("Error: {:?}", err);
                }
            }

            let thread = &mut self.threads[index];
            // thread.unwind();

            match thread.tick() {
                Ok(ThreadResult::Continue) => {
                    index = (index + 1) % self.threads.len();
                }
                Ok(ThreadResult::Stop) => {
                    self.threads.remove(index);
                }
                Ok(ThreadResult::Result(value)) => {
                    result = value;
                    self.threads.remove(index);
                }
                Err(err) => {
                    eprintln!("==== Global ====\n{:?}\n================", self.global);
                    thread.unwind();
                    eprintln!("Error: {:?}", err);
                    self.threads.remove(index);
                }
            }
        }

        Ok(result)
    }
}
