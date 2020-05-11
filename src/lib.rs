//! Gmsh file parser.
pub type Tag = u64; // always write out 8-byte tags

pub mod parser;
pub mod mesh;

use std::io::{self, Write};

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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
    pub fn from_u8_unchecked(dim: u8) -> Dim {
        Dim { dim }
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone)]
pub struct PhysicalGroup {
    pub dim: Dim,
    pub tag: Tag,
    pub name: String,
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Default)]
pub struct Msh {
    pub nodes: Vec<Node>,
    pub elts: Vec<MeshElt>,
    pub physical_groups: Vec<PhysicalGroup>,
    // entity blocs -- need to match to nodes
}

impl Msh {
    pub fn new() -> Msh {
        Msh {
            nodes: Vec::new(),
            elts: Vec::new(),
            physical_groups: Vec::new(),
        }
    }

    pub fn write_msh2<W: Write>(&self, sink: &mut W, storage: Storage) -> io::Result<()> {
        write!(sink, "{}", MshHeader { version: Version::V22, storage })?;
        writeln!(sink, "$Nodes")?;
        writeln!(sink, "{}", self.nodes.len())?;
        for node in &self.nodes {
            match storage {
                Storage::Ascii => writeln!(sink, "{} {} {} {}", node.tag, node.x, node.y, node.z)?,
                Storage::BinaryLe => {
                    sink.write_all(&node.tag.to_le_bytes())?;
                    sink.write_all(&node.x.to_le_bytes())?;
                    sink.write_all(&node.y.to_le_bytes())?;
                    sink.write_all(&node.z.to_le_bytes())?
                },
            }
        }
        writeln!(sink, "$EndNodes")?;
        Ok(())
    }

    pub fn write_msh4<W: Write>(&self, sink: &mut W, storage: Storage) -> io::Result<()> {
        write!(sink, "{}", MshHeader{ version: Version::V41, storage })?;
        Ok(())
    }

    pub fn section(&mut self, s: impl MshSection) {
        todo!();
    }
}

pub trait MshSection {
    fn add_section(self, mesh: &mut Msh);
}

/// Only `size_t` of 8 bytes is supported (like Gmsh itself).
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct MshHeader {
    pub version: Version,
    pub storage: Storage,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Version { V22, V41 }

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Storage { Ascii, BinaryLe }

impl std::fmt::Display for MshHeader {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        use Storage::*;
        use Version::*;
        writeln!(fmt, "$MeshFormat")?;
        let vers = match self.version {
            V22 => "2.2",
            V41 => "4.1",
        };
        let storage = match self.storage {
            Ascii => "0",
            BinaryLe => "1",
        };
        writeln!(fmt, "{} {} 8", vers, storage)?;
        match self.storage {
            Ascii => (/* skip */),
            BinaryLe => writeln!(fmt, "\u{1}\u{0}\u{0}\u{0}")?,
        };
        writeln!(fmt, "$EndMeshFormat")
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Copy, Clone)]
pub struct Node {
    pub tag: Tag,
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone)]
pub struct MeshElt {
    pub tag: Tag,
    pub ty: MeshShape,
    pub nodes: Vec<Tag>,
    /// The element's physical group.
    pub physical_group: Option<Tag>,
    /// The geometry this element comes from.
    pub geometry: Option<Tag>,
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Copy, Clone)]
pub enum MeshShape {
    Point,
    Line,
    Triangle,
    Quad,
    Tetrahedron,
}

impl MeshShape {
    pub fn from_gmsh_label(label: &str) -> Option<MeshShape> {
        match label {
            "1" => Some(MeshShape::Line),
            "2" => Some(MeshShape::Triangle),
            "3" => Some(MeshShape::Quad),
            "4" => Some(MeshShape::Tetrahedron),
            "15" => Some(MeshShape::Point),
            other => {
                None
            }
        }
    }

    pub fn num_nodes(self) -> u8 {
        match self {
            MeshShape::Point => 1,
            MeshShape::Line => 2,
            MeshShape::Triangle => 3,
            MeshShape::Quad => 4,
            MeshShape::Tetrahedron => 4,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use insta::{assert_debug_snapshot, assert_display_snapshot};

    #[test]
    fn write_empty_msh2_ascii() {
        let mut buffer = Vec::new();
        let msh = Msh::new();
        msh.write_msh2(&mut buffer, Storage::Ascii).unwrap();
        assert_debug_snapshot!(String::from_utf8(buffer).unwrap());
    }

    #[test]
    fn write_empty_msh4_ascii() {
        let mut buffer = Vec::new();
        let msh = Msh::new();
        msh.write_msh4(&mut buffer, Storage::Ascii).unwrap();
        assert_debug_snapshot!(String::from_utf8(buffer).unwrap());
    }

    #[test]
    fn basic_msh2_ascii() {
        let mut msh = Msh::new();
        msh.nodes = vec![
            Node { tag: 1, x: 0.0, y: 0.0, z: 0.0, },
            Node { tag: 2, x: 1.0, y: 0.0, z: 0.0, },
        ];
        msh.elts = vec![MeshElt {
                tag: 1,
                ty: MeshShape::Line,
                nodes: vec![1, 2],
                physical_group: None,
                geometry: None,
            }
        ];
        let mut buffer = Vec::new();
        msh.write_msh2(&mut buffer, Storage::Ascii).unwrap();
        assert_debug_snapshot!(String::from_utf8(buffer).unwrap());
    }

    #[cfg(feature = "serde")]
    #[test]
    fn msh_json() {
        let mut msh = Msh::new();
        msh.nodes = vec![
            Node { tag: 1, x: 0.0, y: 0.0, z: 0.0, },
            Node { tag: 2, x: 1.0, y: 0.0, z: 0.0, },
        ];
        msh.elts = vec![MeshElt {
                tag: 1,
                ty: MeshShape::Line,
                nodes: vec![1, 2],
                physical_group: None,
                geometry: None,
            }
        ];
        assert_display_snapshot!(serde_json::to_string(&msh).unwrap());
    }
}
