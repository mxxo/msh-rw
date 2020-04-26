pub type Tag = usize;

#[derive(Debug, Copy, Clone)]
pub struct Dim {
    dim: u8,
}

impl Dim {
    pub fn new(d: u8) -> Option<Dim> {
        match d {
            0..=3 => Some(Dim { dim: d }),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Msh2 {
    pub nodes: Vec<Point>,
    pub elts: Vec<MeshElt>,
    pub physical_groups: Option<Vec<(Dim, Tag)>>,
}

#[derive(Debug, Copy, Clone)]
pub struct Point {
    pub tag: Tag,
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

#[derive(Debug, Clone)]
pub struct MeshElt {
    pub tag: Tag,
    pub ty: MeshShape,
    pub nodes: Vec<Tag>,
    pub physical_groups: Option<Vec<Tag>>,
}

#[derive(Debug, Copy, Clone)]
pub enum MeshShape {
    Node,
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
