package java.lang;

public class Object {
    private static long count = 0;
    private long index;

    public Object() {
        this.index = count++;
    }
 
    public final native Class getClass();

    public long getIndex() {
        return this.index;
    }

    public String toString() {
        return new StringBuilder().append(this.getClass().getName()).append("(index=").append(this.index).append(")").toString();
    }

    public boolean equals(Object other) {
        return this == other;
    }
}
