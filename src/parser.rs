use crate::*;
use std::path::Path;
use thiserror::Error;

use nom::*;
use nom::bytes::complete::tag;
use nom::character::complete::line_ending;
use nom::error::context;

#[derive(Error, Debug)]
pub enum MshError {
    #[error("IO error ({source})")]
    Io {
        #[from]
        source: std::io::Error,
    },
    #[error("header parse error:\n{source}")]
    Parse {
        #[from]
        source: nom::Err<(String, nom::error::ErrorKind)>,
    }
}

pub type MshResult<T> = std::result::Result<T, MshError>;

impl Msh {
    fn from_file<P: AsRef<Path>>(path: P) -> MshResult<Msh> {
        let header = parse_header(&first_four_lines(path)?)?;
        todo!()
    }
}

fn first_four_lines<P: AsRef<Path>>(path: P) -> std::io::Result<String> {
    use std::io::BufRead;
    // examine first 3-4 lines, instead of reading the whole file
    // because binary files aren't utf8
    let file = std::fs::File::open(path)?;
    let mut reader = std::io::BufReader::new(file);
    let mut buffer = String::new();
    for _ in 0..=3 {
        reader.read_line(&mut buffer)?;
    }
    Ok(buffer)
}

// -- helper parsers

pub fn sp(input: &str) -> IResult<&str, &str> {
    tag(" ")(input)
}

// TODO: add many spaces terminated by eol
pub fn end_of_line(input: &str) -> IResult<&str, &str> {
    if input.is_empty() {
        Ok((input, input))
    } else {
        line_ending(input)
    }
}

pub fn file_header(input: &str) -> IResult<&str, &str> {
    context("mesh header", tag("$MeshFormat"))(input)
}

/// Parse a `msh` file header.
pub fn mesh_header(input: &str) -> IResult<&str, MshHeader> {
    do_parse!(input,
        file_header >>
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

fn parse_header(input: &str) -> MshResult<MshHeader> {
    match mesh_header(input) {
        Ok((_extra_text, header)) => Ok(header),
        Err(e) => Err(e.to_owned().into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use insta::{assert_debug_snapshot, assert_display_snapshot};

    #[test]
    fn msh2_ascii_header() {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("props");
        path.push("v2");
        path.push("empty.msh");
        assert_debug_snapshot!(parse_header(&first_four_lines(path).unwrap()).unwrap());
    }

    #[test]
    fn unix_line_endings() {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("props");
        path.push("v2");
        path.push("unix-empty.msh");
        assert_debug_snapshot!(parse_header(&first_four_lines(path).unwrap()).unwrap());
    }

    #[test]
    fn msh2_binary_header() {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("props");
        path.push("v2");
        path.push("empty-bin.msh");
        assert_debug_snapshot!(parse_header(&first_four_lines(path).unwrap()).unwrap());
    }

    #[test]
    fn msh4_ascii_header() {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("props");
        path.push("v4");
        path.push("empty.msh");
        assert_debug_snapshot!(parse_header(&first_four_lines(path).unwrap()).unwrap());
    }

    #[test]
    fn msh4_binary_header() {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("props");
        path.push("v4");
        path.push("empty-bin.msh");
        assert_debug_snapshot!(parse_header(&first_four_lines(path).unwrap()).unwrap());
    }

    #[test]
    fn bad_header() {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("props");
        path.push("v2");
        path.push("bad-header.msh");
        let res = parse_header(&first_four_lines(path).unwrap());
        assert!(res.is_err());
        if let Err(trace) = res {
            assert_display_snapshot!(trace);
        }
    }

    //#[test]
    //fn empty_mesh() {
    //    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    //    path.push("props");
    //    path.push("empty.msh");
    //    Msh::from_file(&path).unwrap();
    //}
}
