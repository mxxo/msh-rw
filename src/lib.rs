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
    pub nodes: Vec<Point>,
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

    pub fn write_msh2<W: Write>(&self, sink: &mut W, storage: MshStorage) -> io::Result<()> {
        write!(sink, "{}", MshHeader::v2(storage))?;
        writeln!(sink, "$Nodes")?;
        writeln!(sink, "{}", self.nodes.len())?;
        for node in &self.nodes {
            match storage {
                MshStorage::Ascii => writeln!(sink, "{} {} {} {}", node.tag, node.x, node.y, node.z)?,
                MshStorage::BinaryLe => {
                    sink.write_all(&node.tag.to_le_bytes())?;
                    sink.write_all(&node.x.to_le_bytes())?;
                    sink.write_all(&node.y.to_le_bytes())?;
                    sink.write_all(&node.z.to_le_bytes())?
                },
                MshStorage::BinaryBe => {
                    sink.write_all(&node.tag.to_be_bytes())?;
                    sink.write_all(&node.x.to_be_bytes())?;
                    sink.write_all(&node.y.to_be_bytes())?;
                    sink.write_all(&node.z.to_be_bytes())?
                },
            }
        }
        writeln!(sink, "$EndNodes")?;
        Ok(())
    }

    pub fn write_msh4<W: Write>(&self, sink: &mut W, storage: MshStorage) -> io::Result<()> {
        write!(sink, "{}", MshHeader::v4(storage))?;
        Ok(())
    }
}

#[derive(Debug, Copy, Clone)]
pub struct MshHeader {
    pub version: MshFormat,
    pub storage: MshStorage,
    pub size_t: MshSizeT,
}

impl MshHeader {
    pub fn v2(storage: MshStorage) -> Self {
        MshHeader {
            storage,
            version: MshFormat::V22,
            size_t: MshSizeT::EightBytes,
        }
    }
    pub fn v4(storage: MshStorage) -> Self {
        MshHeader {
            storage,
            version: MshFormat::V41,
            size_t: MshSizeT::EightBytes,
        }
    }
}

impl std::fmt::Display for MshHeader {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        writeln!(fmt, "$MeshFormat")?;
        writeln!(fmt, "{} {} {}", self.version, self.storage, self.size_t)?;
        match self.storage {
            MshStorage::BinaryBe => writeln!(fmt, "\u{0}\u{0}\u{0}\u{1}")?,
            MshStorage::BinaryLe => writeln!(fmt, "\u{1}\u{0}\u{0}\u{0}")?,
            MshStorage::Ascii => (/* skip */),
        };
        writeln!(fmt, "$EndMeshFormat")
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum MshStorage { Ascii, BinaryLe, BinaryBe }
impl std::fmt::Display for MshStorage {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            MshStorage::Ascii => write!(fmt, "0"),
            MshStorage::BinaryLe | MshStorage::BinaryBe => write!(fmt, "1"),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum MshFormat { V22, V41 }
impl std::fmt::Display for MshFormat {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            MshFormat::V22 => write!(fmt, "2.2"),
            MshFormat::V41 => write!(fmt, "4.1"),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum MshSizeT { FourBytes, EightBytes }
impl std::fmt::Display for MshSizeT {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            MshSizeT::FourBytes => write!(fmt, "4"),
            MshSizeT::EightBytes => write!(fmt, "8"),
        }
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Copy, Clone)]
pub struct Point {
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
    pub physical_groups: Option<Vec<Tag>>,
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Copy, Clone)]
pub enum MeshShape {
    Node,
    Line,
    Triangle,
    Quad,
}

#[cfg(test)]
mod tests {
    use super::*;
    use insta::{assert_debug_snapshot, assert_display_snapshot};

    #[test]
    fn write_empty_msh2_ascii() {
        let mut buffer = Vec::new();
        let msh = Msh::new();
        msh.write_msh2(&mut buffer, MshStorage::Ascii).unwrap();
        assert_debug_snapshot!(String::from_utf8(buffer).unwrap());
    }

    #[test]
    fn write_empty_msh4_ascii() {
        let mut buffer = Vec::new();
        let msh = Msh::new();
        msh.write_msh4(&mut buffer, MshStorage::Ascii).unwrap();
        assert_debug_snapshot!(String::from_utf8(buffer).unwrap());
    }

    #[test]
    fn basic_msh2_ascii() {
        let mut msh = Msh::new();
        msh.nodes = vec![
            Point { tag: 1, x: 0.0, y: 0.0, z: 0.0, },
            Point { tag: 2, x: 1.0, y: 0.0, z: 0.0, },
        ];
        msh.elts = vec![MeshElt {
                tag: 1,
                ty: MeshShape::Line,
                nodes: vec![1, 2],
                physical_groups: None,
            }
        ];
        let mut buffer = Vec::new();
        msh.write_msh2(&mut buffer, MshStorage::Ascii).unwrap();
        assert_debug_snapshot!(String::from_utf8(buffer).unwrap());
    }

    #[cfg(feature = "serde")]
    #[test]
    fn msh_json() {
        let mut msh = Msh::new();
        msh.nodes = vec![
            Point { tag: 1, x: 0.0, y: 0.0, z: 0.0, },
            Point { tag: 2, x: 1.0, y: 0.0, z: 0.0, },
        ];
        msh.elts = vec![MeshElt {
                tag: 1,
                ty: MeshShape::Line,
                nodes: vec![1, 2],
                physical_groups: None,
            }
        ];
        assert_display_snapshot!(serde_json::to_string(&msh).unwrap());
    }
}
