#ifndef TYPES_H
#define TYPES_H

typedef struct {
    int x;
    int y;
} Point;

typedef int (*Comparator)(const void*, const void*);

struct Rectangle {
    Point origin;
    int width;
    int height;
};

enum Color {
    RED,
    GREEN,
    BLUE
};

union Value {
    int i;
    float f;
    char c;
};

extern int global_count;

int add(int a, int b);
Point create_point(int x, int y);

#endif
