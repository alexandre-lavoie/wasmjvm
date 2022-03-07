package java.lang;

public class Object {
    private static long count = 1;
    private long id = 1;

    public Object() {
        this.id = count++;
    }

    public long getId() {
        return this.id;
    }
}
