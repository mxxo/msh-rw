use crate::*;
use std::path::PathBuf;
use thiserror::Error;

use nom::*;
use nom::bytes::complete::tag;
use nom::character::complete::line_ending;
//use nom::number::complete::{double, le_i32};

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

// -- helper parsers

// pub sp<char>, char!(' ')

pub fn sp(input: &str) -> IResult<&str, &str> {
    tag(" ")(input)
    //nom::character::complete::char(' ')(input)
}

// TODO: add many spaces terminated by eol
pub fn end_of_line(input: &str) -> IResult<&str, &str> {
    if input.is_empty() {
        Ok((input, input))
    } else {
        line_ending(input)
    }
}

// named!(pub crlf, do_parse!(opt!(tag("\r")) >> (tag!("\n"))));
pub fn mesh_header(input: &str) -> IResult<&str, MshHeader> {
    do_parse!(input,
        tag!("$MeshFormat") >>
        end_of_line >>
        version: alt!(tag!("2.2") | tag!("4.1")) >>
        sp >>
        binary: one_of!("01") >>
        sp >>
        size_t: one_of!("48") >>
        endian: opt!(
            do_parse!(
                end_of_line >>
                // gmsh docs say ascii int = 1 => I assume this means 4 bytes?
                endianness: take!(4) >>
                (Some(endianness))
            )
        ) >>
        (MshHeader {
            version: match version {
               "2.2" => MshFormat::V22,
               "4.1" => MshFormat::V41,
               _ => panic!(format!("bad version in mesh header: {}", version)),
            },
            storage: match binary {
                '0' => MshStorage::Ascii,
                '1' => match endian.unwrap() {
                    None => panic!("binary header missing endianness"),
                    Some("\u{1}\u{0}\u{0}\u{0}") => MshStorage::BinaryLe,
                    Some(_) => {MshStorage::BinaryBe},
                }
                _ => panic!(format!("bad storage flag {}, expected 0 (ascii) or 1 (binary)", binary)),
            },
            size_t: match size_t {
                '4' => MshSizeT::FourBytes,
                '8' => MshSizeT::EightBytes,
                _ => panic!(format!("bad size_t value: {}", size_t)),
            },
        })
    )
}

//named!(pub msh_header<&str, MshHeader>,
//do_parse!(
//    tag("$MeshFormat") >>
//    end_of_line >>
//    version: alt(tag("2.2") | tag("4.1")) >>
//    sp >>
//    binary: one_of("01") >>
//    sp >>
//    size_t: one_of("48") >>
//    endian: opt(
//        do_parse(
//            sp >>
//            int: le_i32 >>
//            (Some(int))
//        )
//    ) >>
//    end_of_line >>
//    tag!("$EndMeshFormat") >>
//    end_of_line >> (
//        MshHeader {
//            version: match version {
//                "2.2" => MshFormat::V22,
//                "4.1" => MshFormat::V41,
//                _ => panic!(format!("bad version in mesh header: {}", version)),
//            },
//            storage: match binary {
//                '0' => MshStorage::Ascii,
//                '1' => match endian.unwrap() {
//                    None => panic!("binary header missing endianness"),
//                    Some(1) => MshStorage::BinaryLe,
//                    Some(_) => MshStorage::BinaryBe,
//                }
//                _ => panic!(format!("bad storage flag {}, expected 0 (ascii) or 1 (binary)", binary)),
//            },
//            size_t: match size_t {
//                '4' => MshSizeT::FourBytes,
//                '8' => MshSizeT::EightBytes,
//                _ => panic!(format!("bad size_t value: {}", size_t)),
//            },
//        }
//    )
//)
//);

fn parse_header(input: &str) -> IResult<&str, MshHeader> {
    let (input, /*(incomplete,*/ header) = mesh_header(input)?;
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
    fn unix_line_endings() {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("props");
        path.push("v2");
        path.push("unix-empty.msh");
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

    #[test]
    fn msh4_ascii_header() {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("props");
        path.push("v4");
        path.push("empty.msh");
        assert_debug_snapshot!(parse_header(&std::fs::read_to_string(&path).unwrap()).unwrap().1);
    }

    #[test]
    fn msh4_binary_header() {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("props");
        path.push("v4");
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
