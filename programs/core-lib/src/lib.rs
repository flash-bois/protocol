pub struct Point {
    pub x: i32,
    pub y: i32,
}

pub fn add_points(left: Point, right: Point) -> Point {
    Point {
        x: left.x + right.x,
        y: left.y + right.y,
    }
}

pub fn sum(Point { x, y }: Point) -> i32 {
    x + y
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_points() {
        let left = Point { x: 1, y: 2 };
        let right = Point { x: 3, y: 4 };
        let result = add_points(left, right);
        assert_eq!(result.x, 4);
        assert_eq!(result.y, 6);
    }
}
