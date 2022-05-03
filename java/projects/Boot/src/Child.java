public class Child extends Parent {
    private String name;

    public Child(String name, int age) {
        super(age);
        this.name = name;
    }

    public String getName() {
        return this.name;
    }
}
