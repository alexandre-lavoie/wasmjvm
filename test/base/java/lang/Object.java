package java.lang;

public class Object {
    private static long count = 0;
    private long address;

    public Object() {
        this.address = count++;
    }

    public String toString() {
        return this.getClass().getName().append(" (").append(new String(this.address)).append(")");
    }

    public native Class getClass();
}
