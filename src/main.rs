const EPSILON: f64 = 1e-10;
#[derive(Debug, Clone, Copy, PartialEq)]
struct Point {
    x: f64,
    y: f64,
}

#[derive(Debug, Clone)]
struct Line {
    p1: Point,
    p2: Point,
}

/// Line segment Line segment collision check
/// https://en.wikipedia.org/wiki/Line%E2%80%93line_intersection
fn line_segment_intersection(line1: &Line, line2: &Line) -> Option<Point> {
    let (x1, y1) = (line1.p1.x, line1.p1.y);
    let (x2, y2) = (line1.p2.x, line1.p2.y);
    let (x3, y3) = (line2.p1.x, line2.p1.y);
    let (x4, y4) = (line2.p2.x, line2.p2.y);

    let dx1 = x2 - x1;
    let dy1 = y2 - y1;
    let dx2 = x4 - x3;
    let dy2 = y4 - y3;

    let denom = dx1 * dy2 - dy1 * dx2;

    // If denominator is "zero", lines are parallel or coincident
    if denom.abs() < EPSILON {
        return None;
    }

    let dx3 = x3 - x1;
    let dy3 = y3 - y1;

    let t = (dx3 * dy2 - dy3 * dx2) / denom;
    let u = (dx3 * dy1 - dy3 * dx1) / denom;

    // If t and u are in the [0, 1] range with some tolerance we have an intersection.
    if t >= -EPSILON && t <= 1.0 + EPSILON && u >= -EPSILON && u <= 1.0 + EPSILON {
        Some(Point {
            x: x1 + t * dx1,
            y: y1 + t * dy1,
        })
    } else {
        None
    }
}

/// Fins all intersections between a polygon and a line
/// then removes duplicates within +-EPSILON floating point marginal.
fn find_intersections(polygon_points: &Vec<Point>, line: &Line) -> Vec<Point> {
    let mut intersection_points = Vec::new();

    for i in 0..polygon_points.len() {
        let current = &polygon_points[i];
        let next = &polygon_points[(i + 1) % polygon_points.len()];

        let edge_line = Line {
            p1: *current,
            p2: *next,
        };

        if let Some(intersection_point) = line_segment_intersection(line, &edge_line) {
            intersection_points.push(intersection_point);
        }
    }

    // Remove duplicates
    intersection_points.sort_by(|a, b| {
        let by_x = a.x.partial_cmp(&b.x).unwrap_or(std::cmp::Ordering::Equal);
        if by_x == std::cmp::Ordering::Equal {
            a.y.partial_cmp(&b.y).unwrap_or(std::cmp::Ordering::Equal)
        } else {
            by_x
        }
    });
    intersection_points.dedup_by(|a, b| (a.x - b.x).abs() < EPSILON && (a.y - b.y).abs() < EPSILON);

    intersection_points
}

/// Utility to decide which polygon a point belongs to
/// after being split by a given line.
fn point_line_side(line: &Line, p: &Point) -> f64 {
    let Line { p1, p2 } = line;
    let dx = p2.x - p1.x;
    let dy = p2.y - p1.y;
    (p.x - p1.x) * dy - (p.y - p1.y) * dx
}

/// Splits a polygon and returns vector of polygons.
/// Returns None if no valid cut can be found/made.
fn split_polygon(polygon_points: &Vec<Point>, line: &Line) -> Option<Vec<Vec<Point>>> {
    let intersections: Vec<Point> = find_intersections(polygon_points, line);

    if intersections.len() != 2 {
        return None;
    }

    // We'll build two new polygons: one on each side of the line.
    let mut polygon_a = Vec::new();
    let mut polygon_b = Vec::new();

    let mut inserted_first_intersection = false;
    let mut inserted_second_intersection = false;

    for i in 0..polygon_points.len() {
        let current = &polygon_points[i];
        let next = &polygon_points[(i + 1) % polygon_points.len()];

        // Add current point to the appropriate polygon(s)
        let side = point_line_side(line, current);
        if side >= -EPSILON {
            polygon_a.push(current.clone());
        }
        if side <= EPSILON {
            polygon_b.push(current.clone());
        }

        // Check if the edge from current to next is intersected by the line
        let edge_line = Line {
            p1: *current,
            p2: *next,
        };

        if let Some(intercept_point) = line_segment_intersection(line, &edge_line) {
            if !intersections.is_empty() {
                if intersections[0].x == intercept_point.x
                    && intersections[0].y == intercept_point.y
                    && !inserted_first_intersection
                {
                    polygon_a.push(intercept_point.clone());
                    polygon_b.push(intercept_point.clone());
                    inserted_first_intersection = true;
                } else if intersections.len() > 1
                    && intersections[1].x == intercept_point.x
                    && intersections[1].y == intercept_point.y
                    && !inserted_second_intersection
                {
                    polygon_a.push(intercept_point.clone());
                    polygon_b.push(intercept_point.clone());
                    inserted_second_intersection = true;
                }
            }
        }
    }

    Some(vec![polygon_a, polygon_b])
}

/// https://en.wikipedia.org/wiki/Shoelace_formula
fn polygon_area(points: &Vec<Point>) -> f64 {
    let point_count = points.len();
    if point_count < 3 {
        return 0.0; // Not a polygon by definition.
    }

    let mut area = 0.0;
    for point_index in 0..point_count {
        // Next point index (wrapping around to zero with % n)
        let next_point_index = (point_index + 1) % point_count;

        // Apply shoelace formula component
        area += points[point_index].x * points[next_point_index].y
            - points[next_point_index].x * points[point_index].y;
    }

    ((area / 2.0).abs() * 10_000_000.0).round() / 10_000_000.0
}

/// Splits polygon into smaller polygons by a list of lines.
/// Returns the area of the largest polygon found.
fn get_largest_polygon_area(polygon_points: &Vec<Point>, lines: &Vec<Line>) -> f64 {
    let mut polygons: Vec<Vec<Point>> = vec![polygon_points.clone()];

    let mut new_polygons = Vec::new();
    for line in lines {
        new_polygons.clear();

        for poly in &polygons {
            match split_polygon(poly, line) {
                Some(mut split_result) => {
                    new_polygons.append(&mut split_result);
                }
                None => {
                    new_polygons.push(poly.clone());
                }
            }
        }

        // Discard cut/consumed polygons
        polygons = new_polygons.clone();
    }

    // find largest polygon area
    let mut largest_area = 0.0;
    for poly in &polygons {
        let area = polygon_area(&poly);
        if area > largest_area {
            largest_area = area;
        }
    }

    largest_area
}

// Main function is the same as the
// sample testcase from the test module.
fn main() {
    // Polygon defined as a vector of points.
    let polygon_points = vec![
        Point { x: 0.0, y: 0.0 },
        Point { x: 1.0, y: 0.0 },
        Point { x: 1.0, y: 1.0 },
        Point { x: 0.0, y: 1.0 },
    ];

    // Lines defined as two points.
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

    // Program will cut the original polygons by the lines one by one,
    // in order, and return the area of the largest polygon found.

    println!("{}", get_largest_polygon_area(&polygon_points, &lines));
}

#[cfg(test)]
mod tests {
    use crate::{get_largest_polygon_area, Line, Point};

    fn round_f64(value: f64) -> f64 {
        (value * 1_000_000.0).round() / 1_000_000.0
    }

    #[test]
    fn sample() {
        let polygon_points = vec![
            Point { x: 0.0, y: 0.0 },
            Point { x: 1.0, y: 0.0 },
            Point { x: 1.0, y: 1.0 },
            Point { x: 0.0, y: 1.0 },
        ];
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

        assert_eq!(
            round_f64(get_largest_polygon_area(&polygon_points, &lines)),
            0.375
        );
    }

    #[test]
    fn extra_no_cut() {
        let polygon_points = vec![
            Point { x: 0.0, y: 0.0 },
            Point { x: 1.0, y: 0.0 },
            Point { x: 1.0, y: 1.0 },
            Point { x: 0.0, y: 1.0 },
        ];
        let lines = vec![Line {
            p1: Point { x: 2.0, y: 2.0 },
            p2: Point { x: 3.0, y: 3.0 },
        }];

        assert_eq!(
            round_f64(get_largest_polygon_area(&polygon_points, &lines)),
            1.0
        );
    }

    #[test]
    fn extra_intersect_one_vertex() {
        let polygon_points = vec![
            Point { x: 0.0, y: 0.0 },
            Point { x: 1.0, y: 0.0 },
            Point { x: 1.0, y: 1.0 },
            Point { x: 0.0, y: 1.0 },
        ];
        let lines = vec![Line {
            p1: Point { x: 2.0, y: 2.0 },
            p2: Point { x: 0.5, y: 0.5 },
        }];

        assert_eq!(
            round_f64(get_largest_polygon_area(&polygon_points, &lines)),
            1.0
        );
    }

    #[test]
    fn extra_simple_1() {
        let polygon_points = vec![
            Point { x: 0.0, y: 0.0 },
            Point { x: 1.0, y: 0.0 },
            Point { x: 1.0, y: 1.0 },
            Point { x: 0.0, y: 1.0 },
        ];
        let lines = vec![Line {
            p1: Point { x: 0.5, y: 0.0 },
            p2: Point { x: 0.5, y: 1.0 },
        }];

        assert_eq!(
            round_f64(get_largest_polygon_area(&polygon_points, &lines)),
            0.5
        );
    }

    #[test]
    fn extra_simple_2() {
        let polygon_points = vec![
            Point { x: 0.0, y: 0.0 },
            Point { x: 1.0, y: 0.0 },
            Point { x: 1.0, y: 1.0 },
            Point { x: 0.0, y: 1.0 },
        ];
        let lines = vec![
            Line {
                p1: Point { x: 0.5, y: 0.0 },
                p2: Point { x: 0.5, y: 1.0 },
            },
            Line {
                p1: Point { x: 0.0, y: 0.5 },
                p2: Point { x: 1.0, y: 0.5 },
            },
        ];

        assert_eq!(
            round_f64(get_largest_polygon_area(&polygon_points, &lines)),
            0.25
        );
    }

    #[test]
    fn extra_two_double_intersects_diagonal() {
        let polygon_points = vec![
            Point { x: 0.0, y: 0.0 },
            Point { x: 1.0, y: 0.0 },
            Point { x: 1.0, y: 1.0 },
            Point { x: 0.0, y: 1.0 },
        ];
        let lines = vec![Line {
            p1: Point { x: 0.0, y: 0.0 },
            p2: Point { x: 1.0, y: 1.0 },
        }];

        assert_eq!(
            round_f64(get_largest_polygon_area(&polygon_points, &lines)),
            0.5
        );
    }

    #[test]
    fn extra_intersect_partly_diagonal() {
        let polygon_points = vec![
            Point { x: 0.0, y: 0.0 },
            Point { x: 1.0, y: 0.0 },
            Point { x: 1.0, y: 1.0 },
            Point { x: 0.0, y: 1.0 },
        ];
        let lines = vec![Line {
            p1: Point { x: 0.1, y: 0.0 },
            p2: Point { x: 1.0, y: 1.0 },
        }];

        assert_eq!(
            round_f64(get_largest_polygon_area(&polygon_points, &lines)),
            0.55
        );
    }

    #[test]
    fn extra_simple_6_digits_of_accuracy() {
        let polygon_points = vec![
            Point { x: 0.0, y: 0.0 },
            Point { x: 1.0, y: 0.0 },
            Point { x: 1.0, y: 1.0 },
            Point { x: 0.0, y: 1.0 },
        ];
        let lines = vec![Line {
            p1: Point {
                x: 0.123456789,
                y: 0.0,
            },
            p2: Point {
                x: 0.123456789,
                y: 1.0,
            },
        }];

        assert_eq!(
            round_f64(get_largest_polygon_area(&polygon_points, &lines)),
            0.876543
        );
    }
}
