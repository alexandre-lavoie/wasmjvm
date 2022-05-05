use wasm_bindgen::prelude::*;

#[wasm_bindgen(module="wasmjvm_interface")]
extern "C" {
    #[wasm_bindgen(js_name="RustInterface")]
    pub type JS;

    #[wasm_bindgen(static_method_of=JS, js_class="RustInterface")]
    pub fn file_bind(pointer: usize, path: String);

    #[wasm_bindgen(static_method_of=JS, js_class="RustInterface")]
    pub fn file_write(pointer: usize, value: i32);

    #[wasm_bindgen(static_method_of=JS, js_class="RustInterface")]
    pub fn error(message: String);

    #[wasm_bindgen(static_method_of=JS, js_class="RustInterface")]
    pub async fn file_read(pointer: usize) -> JsValue;

    #[wasm_bindgen(static_method_of=JS, js_class="RustInterface")]
    pub fn random() -> i64;
}
