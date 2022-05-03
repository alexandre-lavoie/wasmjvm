use wasmjvm_common::WasmJVMError;
use wasmjvm_native::{
    Global, Loader, NativeInterface, Primitive, RegisterFn, RustObject, Thread, ThreadResult,
    JAVA_LOADER, JAVA_NATIVE, JAVA_THREAD,
};

pub struct VM {
    global: Global,
    natives: Vec<RegisterFn>
}

impl VM {
    pub fn new() -> Self {
        let global = Global::new();
        Self {
            global,
            natives: Vec::new()
        }
    }

    pub fn boot<F: std::io::Read + std::io::Seek>(
        self: &mut Self,
        jar: F,
    ) -> Result<(), WasmJVMError> {
        let mut loader = Loader::new(self.global.clone());

        loader.load_boot_jar(jar)?;

        self.global
            .new_rust_instance(&JAVA_LOADER.to_string(), RustObject::Loader(loader))?;

        let native = NativeInterface::new();
        self.global
            .new_rust_instance(&JAVA_NATIVE.to_string(), RustObject::Native(native))?;

        Ok(())
    }

    pub fn load_jars<F: std::io::Read + std::io::Seek>(
        self: &mut Self,
        jars: Vec<F>
    ) -> Result<(), WasmJVMError> {
        let loader = self.global.loader_mut()?;

        for jar in jars {
            loader.load_jar(jar)?;
        }

        Ok(())
    }

    pub fn register_native(self: &mut Self, r#fn: RegisterFn) -> Result<(), WasmJVMError> {
        self.natives.push(r#fn);

        Ok(())
    }

    pub fn register_natives(self: &mut Self) -> Result<(), WasmJVMError> {
        while self.natives.len() > 0 {
            let r#fn = self.natives.pop().unwrap();
            self.global.register_native(r#fn)?;
        }

        Ok(())
    }

    pub fn stack_trace(self: &mut Self) -> Result<String, WasmJVMError> {
        let mut buffer = Vec::new();

        for thread_index in self.global.threads().clone().iter() {
            buffer.push(self.global.thread_mut(*thread_index)?.stack_trace()?);
        }

        Ok(buffer.join("\n"))
    }

    pub fn heap_trace(self: &mut Self) -> Result<String, WasmJVMError> {
        self.global.heap_trace()
    }

    pub fn run(self: &mut Self) -> Result<Primitive, WasmJVMError> {
        let mut result = Primitive::Void;

        let main_thread = Thread::new_main(self.global.clone())?;
        self.global
            .new_rust_instance(&JAVA_THREAD.to_string(), RustObject::Thread(main_thread))?;

        loop {
            let mut stop = true;

            for thread_index in self.global.threads().clone().iter() {
                match self.global.thread_tick(*thread_index) {
                    Ok(ThreadResult::Continue) => {
                        stop = false;
                    }
                    Ok(ThreadResult::Stop) => {}
                    Ok(ThreadResult::Result(value)) => {
                        result = value;
                    }
                    Err(err) => return Err(err),
                }
            }

            if stop {
                break;
            }
        }

        Ok(result)
    }
}
