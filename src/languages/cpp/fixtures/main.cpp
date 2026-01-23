#include "types.hpp"
#include <iostream>

namespace geometry {

Point::Point(int x, int y) : x(x), y(y) {}

int Point::distance_to(const Point& other) const {
    int dx = x - other.x;
    int dy = y - other.y;
    return dx * dx + dy * dy;
}

Rectangle::Rectangle(Point origin, int width, int height)
    : origin(origin), width(width), height(height) {}

int Rectangle::area() const {
    return width * height;
}

}  // namespace geometry

namespace utils {

int helper(int value) {
    return value * 2;
}

}  // namespace utils

int main() {
    geometry::Point p1(0, 0);
    geometry::Point p2(3, 4);

    int dist = p1.distance_to(p2);

    geometry::Rectangle rect(p1, 100, 50);
    int a = rect.area();

    geometry::Color c = geometry::Color::Red;
    geometry::Size s{10, 20};

    int result = utils::helper(42);

    std::cout << "Distance: " << dist << ", Area: " << a << std::endl;
    return 0;
}
