#ifndef TYPES_HPP
#define TYPES_HPP

#include <string>

namespace geometry {

class Point {
public:
    int x;
    int y;

    Point(int x, int y);
    int distance_to(const Point& other) const;
};

class Rectangle {
public:
    Point origin;
    int width;
    int height;

    Rectangle(Point origin, int width, int height);
    int area() const;
};

struct Size {
    int width;
    int height;
};

enum class Color {
    Red,
    Green,
    Blue
};

}  // namespace geometry

namespace utils {

int helper(int value);

}  // namespace utils

typedef int (*Callback)(int);

#endif
