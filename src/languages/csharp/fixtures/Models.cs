namespace MyApp
{
    public class User
    {
        public string Name { get; set; }
        public int Age { get; set; }

        public User(string name, int age)
        {
            Name = name;
            Age = age;
        }
    }

    public class Order
    {
        public int Id { get; set; }
        public OrderStatus Status { get; set; }
    }

    public enum OrderStatus
    {
        Pending,
        Active,
        Completed
    }

    public interface IRepository<T>
    {
        T GetById(int id);
        void Save(T entity);
    }

    public record Person(string FirstName, string LastName);
}
