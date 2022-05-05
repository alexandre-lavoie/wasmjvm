use wasmjvm_common::WasmJVMError;
use wasmjvm_native::{
    Global, Jar, Loader, NativeInterface, Primitive, RegisterFn, RustObject, Thread, ThreadResult,
    JAVA_LOADER, JAVA_NATIVE, JAVA_THREAD,
};

pub struct VM {
    global: Global,
    natives: Vec<RegisterFn>,
    loader: Option<Loader>,
    booted: bool
}

impl VM {
    pub fn new() -> Self {
        let global = Global::new();
        Self {
            global: global.clone(),
            natives: Vec::new(),
            loader: Some(Loader::new(global)),
            booted: false
        }
    }

    pub fn boot(self: &mut Self) -> Result<(), WasmJVMError> {
        self.booted = true;

        let mut loader: Loader = self
            .loader
            .replace(Loader::new(self.global.clone()))
            .unwrap();

        loader.boot()?;
        loader.load_class_name(JAVA_NATIVE)?;

        self.global
            .new_rust_instance(JAVA_LOADER, RustObject::Loader(loader))?;

        let native = NativeInterface::new();
        self.global
            .new_rust_instance(JAVA_NATIVE, RustObject::Native(native))?;

        self.register_natives()?;

        Ok(())
    }

    pub fn load_jar<F: 'static + std::io::Read + std::io::Seek>(
        self: &mut Self,
        jar: Jar<F>,
    ) -> Result<(), WasmJVMError> {
        let loader = self.loader.as_mut().unwrap();

        loader.load_jar(jar)?;

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

    pub async fn tick(self: &mut Self) -> Result<Option<Primitive>, WasmJVMError> {
        let mut result = Primitive::Void;
        let mut stop = true;

        for thread_index in self.global.threads().clone().iter() {
            match self.global.thread_tick(*thread_index).await {
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
            Ok(Some(result))
        } else {
            Ok(None)
        }
    }

    pub async fn run(self: &mut Self) -> Result<Primitive, WasmJVMError> {
        if !self.booted {
            self.boot()?
        }

        let main_thread = Thread::new_main(self.global.clone())?;
        self.global
            .new_rust_instance(&JAVA_THREAD.to_string(), RustObject::Thread(main_thread))?;

        loop {
            if let Some(data) = self.tick().await? {
                return Ok(data);
            }
        }
    }
}
