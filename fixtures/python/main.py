from utils import Counter, add, PI
from models import User, Status


def greet(name: str) -> str:
    return f"Hello, {name}"


def main() -> None:
    user = User(name="Ada", status=Status.ACTIVE)
    total = add(2, 3)
    counter = Counter(start=1)
    counter.inc()
    print(greet(user.name))
    print(f"total={total}, pi={PI}, counter={counter.value}")


if __name__ == "__main__":
    main()
