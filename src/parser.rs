use crate::*;
use std::path::PathBuf;
use thiserror::Error;

use nom::*;
use nom::bytes::complete::tag;
use nom::character::complete::line_ending;
use nom::number::complete::{double, le_i8};

#[derive(Error, Debug)]
pub enum MshError {
    #[error("invalid header (expected {expected:?}, found {found:?})")]
    InvalidHeader {
        expected: String,
        found: String,
    },
}

pub type MshResult<T> = std::result::Result<T, MshError>;

impl Msh2 {
    fn from_file(file: &PathBuf) -> MshResult<Msh2> {
        let filestr = std::fs::read_to_string(&file).expect("IO error");
        let input = parse_header(&filestr).unwrap();
        std::dbg!(input);
        todo!();
    }
}

#[derive(Debug, Copy, Clone)]
pub struct MshHeader {
    pub version: MshFormat,
    pub storage: MshStorage,
    pub size_t: MshSizeT,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum MshFormat { V22, V41 }
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum MshStorage { Ascii, BinaryLe, BinaryBe }
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum MshSizeT { FourBytes, EightBytes }

named!(pub sp<char>, char!(' '));
// add many spaces terminated by \r\n
named!(pub crlf, tag!("\r\n"));

named!(pub msh_header<MshHeader>,
do_parse!(
    tag!("$MeshFormat") >>
    crlf >>
    version: alt!(tag!("2.2") | tag!("4.1")) >>
    sp >>
    binary: one_of!("01") >>
    sp >>
    size_t: one_of!("48") >>
    endian: opt!(
        do_parse!(
            sp >>
            int: le_i8 >>
            (Some(int))
        )
    ) >>
    crlf >>
    tag!("$EndMeshFormat") >>
    crlf >> (
        MshHeader {
            version: match version {
                b"2.2" => MshFormat::V22,
                b"4.1" => MshFormat::V41,
                _ => panic!(format!("bad version in mesh header: {}", std::str::from_utf8(version).unwrap())),
            },
            storage: match binary {
                '0' => MshStorage::Ascii,
                '1' => match endian.unwrap() {
                    None => panic!("binary header missing endianness"),
                    Some(1) => MshStorage::BinaryLe,
                    Some(_) => MshStorage::BinaryBe,
                }
                _ => panic!(format!("bad storage flag {}, expected 0 (ascii) or 1 (binary)", binary)),
            },
            size_t: match size_t {
                '4' => MshSizeT::FourBytes,
                '8' => MshSizeT::EightBytes,
                _ => panic!(format!("bad size_t value: {}", size_t)),
            },
        }
    )
)
);

fn parse_header(input: &str) -> IResult<&[u8], MshHeader> {
    let (input, header) = msh_header(input.as_bytes())?;
    std::dbg!(header);
    Ok((input, header))
}

#[cfg(test)]
mod tests {
    use super::*;
    use insta::assert_debug_snapshot;

    #[test]
    fn msh2_ascii_header() {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("props");
        path.push("v2");
        path.push("empty.msh");
        assert_debug_snapshot!(parse_header(&std::fs::read_to_string(&path).unwrap()).unwrap().1);
    }

    #[test]
    fn msh2_binary_header() {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("props");
        path.push("v2");
        path.push("empty-bin.msh");
        assert_debug_snapshot!(parse_header(&std::fs::read_to_string(&path).unwrap()).unwrap().1);
    }

    //#[test]
    //fn empty_mesh() {
    //    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    //    path.push("props");
    //    path.push("empty.msh");
    //    Msh2::from_file(&path).unwrap();
    //}
}
