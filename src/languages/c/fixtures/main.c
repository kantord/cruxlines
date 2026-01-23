#include "types.h"
#include <stdio.h>

int global_count = 0;

int add(int a, int b) {
    return a + b;
}

Point create_point(int x, int y) {
    Point p;
    p.x = x;
    p.y = y;
    return p;
}

int main(void) {
    Point p = create_point(10, 20);
    int result = add(p.x, p.y);

    struct Rectangle rect;
    rect.origin = p;
    rect.width = 100;
    rect.height = 50;

    enum Color c = RED;

    printf("Result: %d\n", result);
    return 0;
}
