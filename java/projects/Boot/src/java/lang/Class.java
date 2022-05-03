package java.lang;

public class Class {
    public native String getName();

    public String toString() {
        return new StringBuilder().append("Class(name=").append(getName()).append(")").toString();
    }
}
