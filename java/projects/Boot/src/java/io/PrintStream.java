package java.io;

public class PrintStream extends OutputStream {
    private OutputStream outputStream;

    public PrintStream(OutputStream outputStream) {
        this.outputStream = outputStream;
    }

    public PrintStream(String path) {
        this.outputStream = new FileOutputStream(path);
    }

    @Override
    public void write(int value) {
        outputStream.write(value);
    }

    public void print(String string) {
        for(byte b : string.getBytes()) {
            this.write(b);
        }
    }

    public void println(String string) {
        this.print(string);
        this.write('\n');
    }

    public void println() {
        this.write('\n');
    }
}
