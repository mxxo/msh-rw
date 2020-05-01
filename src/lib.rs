//! Gmsh **`msh`** file parser.
pub mod parser;

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

#[derive(Debug, Clone, Default)]
pub struct Msh {
    pub nodes: Vec<Point>,
    pub elts: Vec<MeshElt>,
    pub physical_groups: Vec<(Dim, Tag)>,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum MshStorage { Ascii, BinaryLe, BinaryBe }

impl Msh {

    pub fn new() -> Msh {
        Msh {
            nodes: Vec::new(),
            elts: Vec::new(),
            physical_groups: Vec::new(),
        }
    }

    pub fn write_msh2(&self, storage: MshStorage) -> std::io::Result<()> {
        todo!();
    }

    pub fn write_msh4(&self, storage: MshStorage) -> std::io::Result<()> {
        todo!();
    }
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

}
