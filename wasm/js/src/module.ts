export async function asyncLoad(): Promise<any> {
    return new Promise<any>((resolve, reject) => {
        require("wasmjvm").then(module => resolve(module));
    });
}
