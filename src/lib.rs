#[derive(Debug, Clone)]
pub struct Msh2 {
    nodes: Vec<Point>,
    elts: Vec<MeshElt>,
}

#[derive(Debug, Copy, Clone)]
pub struct Point {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

#[derive(Debug, Clone)]
pub struct MeshElt {
    pub ty: MeshShape,
    // pub physical_groups: Vec<
}

#[derive(Debug, Copy, Clone)]
pub enum MeshShape {
    Line,
    Triangle,
    Quad,
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
