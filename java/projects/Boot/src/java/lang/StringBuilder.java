package java.lang;

public class StringBuilder {
    private byte[] buffer;
    private int pointer;

    public StringBuilder() {
        this.buffer = new byte[1024];
        this.pointer = 0;
    }

    public String toString() {
        byte[] bufferCopy = new byte[this.pointer];

        for(int i = 0; i < bufferCopy.length; i++) {
            bufferCopy[i] = this.buffer[i];
        }

        return new String(bufferCopy);
    }

    public StringBuilder append(boolean value) {
        return this.append(value ? "True" : "False");
    }

    public StringBuilder append(String value) {
        return this.append(value.getBytes());
    }

    public StringBuilder append(byte[] bytes) {
        for(int i = 0; i < bytes.length; i++) {
            buffer[this.pointer++] = bytes[i];
        }

        return this;
    }

    public StringBuilder append(char value) {
        this.buffer[this.pointer++] = (byte)value;

        return this;
    }

    public StringBuilder append(int value) {
        return this.append((long)value);
    }

    public StringBuilder append(long value) {
        byte[] buffer = new byte[21];
        boolean positive = true;

        if(value < 0) {
            positive = false;
            value *= -1;
        }
        
        int i = buffer.length - 1;
        for(; i > 0; i--) {
            buffer[i] = (byte)('0' + value % 10);
            value /= 10;
            if(value == 0) {
                break;
            }
        }

        if (!positive) this.buffer[this.pointer++] = '-';

        for(int j = i; j < buffer.length; j++) {
            this.buffer[j - i + this.pointer] = buffer[j];
        }

        this.pointer += buffer.length - i;

        return this;
    }
}
