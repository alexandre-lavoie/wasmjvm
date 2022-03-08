use wasm_bindgen::prelude::*;

#[wasm_bindgen(module="wasmjvm_interface")]
extern "C" {
    #[wasm_bindgen(js_name="default")]
    pub type JS;

    #[wasm_bindgen(static_method_of=JS, js_class="default")]
    pub fn log(message: String);

    #[wasm_bindgen(static_method_of=JS, js_class="default")]
    pub fn error(message: String);

    #[wasm_bindgen(static_method_of=JS, js_class="default")]
    pub fn prompt() -> String;
}
