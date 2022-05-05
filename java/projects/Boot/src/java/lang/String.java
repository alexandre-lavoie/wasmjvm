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

    public int length() {
        return this.getBytes().length;
    }

    public char charAt(int index) {
        return (char)getInternal()[index];
    }

    @Override
    public String toString() {
        return this;
    }

    @Override
    public boolean equals(Object other) {
        if (!(other instanceof String)) return false;

        String otherString = (String)other;

        if(this.length() != otherString.length()) return false;

        byte[] thisBytes = this.getBytes();
        byte[] otherBytes = otherString.getBytes();

        for(int i = 0; i < thisBytes.length; i++) {
            if (thisBytes[i] != otherBytes[i]) return false;
        }

        return true;
    }
}
