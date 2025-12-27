PI = 3.14159


def add(a: int, b: int) -> int:
    return a + b


class Counter:
    def __init__(self, start: int = 0) -> None:
        self.value = start

    def inc(self) -> int:
        self.value += 1
        return self.value
