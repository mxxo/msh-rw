use crate::Point;

// pub trait MeshElt?

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Copy, Clone)]
pub struct Node {
    point: Point,
}

impl Node {
    pub fn from_points(p: Point) -> Node {
        Node { point: p }
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Copy, Clone)]
pub struct Line {
    points: (Point, Point),
}

impl Line {
    pub fn from_points(a: Point, b: Point) -> Line {
        Line { points: (a, b) }
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Copy, Clone)]
pub struct Tri {
    points: (Point, Point, Point),
}

impl Tri {
    pub fn from_points(a: Point, b: Point, c: Point) -> Tri {
        Tri { points: (a, b, c) }
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Copy, Clone)]
pub struct Quad {
    points: (Point, Point, Point, Point),
}

impl Quad {
    pub fn from_points(a: Point, b: Point, c: Point, d: Point) -> Option<Quad> {
        todo!()
    }

    /// Not all point configurations are correct.
    pub fn from_points_unchecked(a: Point, b: Point, c: Point, d: Point) -> Quad {
        Quad { points: (a, b, c, d) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

}
