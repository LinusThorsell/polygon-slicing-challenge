# Convex Polygon Splitting Challenge

This is a coding challenge that follows the following rules:
- Define a convex polygon by a list of points (x, y).
- Define a list of lines (x1, y1, x2, y2).
- Given the polygon and the lines, take the first line and split the polygon on the interception point. Take the next line, split the two new polygons with the next line to create (potentially) double the amount of polygons. Continue doing this until all lines have been used to cut the polygons.
- Calculate the largest area of the polygons.

### Example

Given this polygon.
```rust
let polygon_points = vec![
    Point { x: 0.0, y: 0.0 },
    Point { x: 1.0, y: 0.0 },
    Point { x: 1.0, y: 1.0 },
    Point { x: 0.0, y: 1.0 },
];
```

And these lines:
```rust
    let lines = vec![
        Line {
            p1: Point { x: 0.0, y: 0.0 },
            p2: Point { x: 1.0, y: 1.0 },
        },
        Line {
            p1: Point { x: 0.5, y: 0.0 },
            p2: Point { x: 0.5, y: 1.0 },
        },
    ];
```

Perform the challenge yourself! The result should be `0.375`! Good luck! There are more testcases in the `main.rs` file.
