package java.lang;

public class String extends Object {
    public String() {}

    public String(byte[] raw) {
        this.setInternal(raw);
    }

    public String(long value) {
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

        byte[] raw = new byte[buffer.length - i + 2];
        for(int j = i; j < buffer.length; j++) {
            raw[j - i + 1] = buffer[j];
        }

        raw[0] = (byte)(positive ? '+' : '-');

        this.setInternal(raw);
    }

    public byte[] getBytes() {
        return this.getInternal();
    }

    public String append(String string) {
        byte[] left = this.getInternal();
        byte[] right = string.getBytes();
        byte[] buffer = new byte[left.length + right.length];

        for (int i = 0; i < buffer.length; i++) {
            if (i >= left.length) {
                buffer[i] = right[i - left.length];
            } else {
                buffer[i] = left[i];
            }
        }

        return new String(buffer);
    }

    private native byte[] getInternal();
    private native void setInternal(byte[] value);
}
