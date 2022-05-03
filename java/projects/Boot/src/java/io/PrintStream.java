package java.io;

public abstract class PrintStream {
    public abstract void write(int value);

    public void write(byte value) {
        this.write((int)value);
    }

    public void write(char value) {
        this.write((int)value);
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
}
