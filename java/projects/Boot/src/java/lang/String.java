package java.lang;

public class String {
    public String(byte[] raw) {
        this.setInternal(raw);
    }

    private native byte[] getInternal();
    private native void setInternal(byte[] value);

    public byte[] getBytes() {
        return this.getInternal();
    }
}
