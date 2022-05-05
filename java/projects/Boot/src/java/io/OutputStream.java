package java.io;

public abstract class OutputStream {
    public abstract void write(int value);

    public void write(byte value) {
        this.write((int)value);
    }

    public void write(char value) {
        this.write((int)value);
    }
}
