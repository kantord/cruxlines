public class Main {
    public static void main(String[] args) {
        User user = new User("Ada");
        System.out.println(user.getName());
        Status status = Status.ACTIVE;
        int total = Utils.add(1, 2);
        System.out.println(status);
        System.out.println(total);
    }
}
