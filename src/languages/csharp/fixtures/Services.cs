namespace MyApp
{
    public class Calculator
    {
        public int Add(int a, int b)
        {
            return a + b;
        }

        public int Multiply(int a, int b)
        {
            return a * b;
        }
    }

    public class UserRepository : IRepository<User>
    {
        public User GetById(int id)
        {
            return new User("Unknown", 0);
        }

        public void Save(User entity)
        {
            // Save logic
        }
    }

    public struct Point
    {
        public int X;
        public int Y;
    }

    public delegate void EventCallback(string message);
}
