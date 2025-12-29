public class InteropCall {
    public static void main(String[] args) {
        System.out.println(kotlinGreet());
        JavaUser user = new JavaUser("Ada");
        System.out.println(user.getName());
    }
}
