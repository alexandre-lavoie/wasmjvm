export default class RustInterface {
    private static pendingStream: Map<number, ((str: string) => void)> = new Map();
    private static bufferStream: Map<number, string> = new Map();

    public static file_bind(pointer: number, path: string): void {
        self.postMessage({ name: "bind", payload: {
            pointer,
            path
        }});
    }

    public static async file_read(pointer: number): Promise<number> {
        if(RustInterface.bufferStream.has(pointer)) {
            let buffer: string = RustInterface.bufferStream.get(pointer);

            if(buffer != null && buffer.length > 0) {
                let byte = buffer.charCodeAt(0);
                RustInterface.bufferStream.set(pointer, buffer.substring(1));
                return byte;
            }
        }

        let buffer = await new Promise<string>(resolve => {
            this.pendingStream.set(pointer, resolve);

            self.postMessage({ name: "input", payload: {
                pointer
            }});
        });

        this.pendingStream.delete(pointer);
        let byte = buffer.charCodeAt(0);
        RustInterface.bufferStream.set(pointer, buffer.substring(1));

        return byte;
    }

    public static async file_write(pointer: number, value: number): Promise<void> {
        self.postMessage({ name: "output", payload: {
            pointer,
            buffer: String.fromCharCode(value)
        }});
    }

    private static readonly MAX_INT: number = Math.pow(2, 53)-1;
    public static random(): BigInt {
        return BigInt(Math.floor(Math.random() * RustInterface.MAX_INT));
    }

    public static error(message: string): void {
        console.error(message);
    }

    public static resolveStream(pointer: number, buffer: string) {
        this.pendingStream.get(pointer)(buffer);
    }
}
