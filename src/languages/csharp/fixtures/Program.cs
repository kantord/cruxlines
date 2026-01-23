namespace MyApp
{
    public class Program
    {
        public static void Main(string[] args)
        {
            var user = new User("Alice", 30);
            var calculator = new Calculator();

            int result = calculator.Add(1, 2);
            Console.WriteLine($"Hello {user.Name}, result is {result}");

            var order = new Order { Id = 1, Status = OrderStatus.Active };
            IRepository<User> repo = new UserRepository();
        }
    }
}
