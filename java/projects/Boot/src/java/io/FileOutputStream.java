package java.io;

public class FileOutputStream extends PrintStream {
    private String path;

    public FileOutputStream(String path) {
        this.path = path;
        this.nativeBind();
    }

    private native void nativeBind();
    private native void nativeWrite(int value);

    @Override
    public void write(int value) {
        nativeWrite(value);
    }

    @Override
    public void print(String string) {
        super.print(string);
    }
}
