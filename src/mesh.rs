use crate::Node;

// pub trait MeshElt?

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Copy, Clone)]
pub struct Point {
    point: Node,
}

impl Point {
    pub fn from_points(p: Node) -> Point {
        Point { point: p }
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Copy, Clone)]
pub struct Line {
    points: (Node, Node),
}

impl Line {
    pub fn from_points(a: Node, b: Node) -> Line {
        Line { points: (a, b) }
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Copy, Clone)]
pub struct Tri {
    points: (Node, Node, Node),
}

impl Tri {
    pub fn from_points(a: Node, b: Node, c: Node) -> Tri {
        Tri { points: (a, b, c) }
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Copy, Clone)]
pub struct Quad {
    points: (Node, Node, Node, Node),
}

impl Quad {
    pub fn from_points(a: Node, b: Node, c: Node, d: Node) -> Option<Quad> {
        todo!()
    }

    /// Not all point configurations are correct.
    pub fn from_points_unchecked(a: Node, b: Node, c: Node, d: Node) -> Quad {
        Quad { points: (a, b, c, d) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
}
