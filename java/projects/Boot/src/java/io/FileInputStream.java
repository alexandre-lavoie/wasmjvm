package java.io;

public class FileInputStream extends InputStream {
    private String path;

    public FileInputStream(String path) {
        this.path = path;
        this.nativeBind();
    }

    private native void nativeBind();
    private native int nativeRead();

    @Override
    public int read() {
        return nativeRead();
    }
}
