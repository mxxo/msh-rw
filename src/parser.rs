use crate::*;

use thiserror::Error;

use nom::*;
use nom::bytes::complete::tag;
use nom::combinator::map_res;
use nom::character::complete::{line_ending, digit1};
use nom::multi::many_till;
use nom::number::complete::double;
use nom::error::context;

use std::path::Path;
use std::str::FromStr;

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

pub fn tab(input: &str) -> IResult<&str, &str> {
    tag("\t")(input)
}

pub fn whitespace(input: &str) -> IResult<&str, &str> {
    use nom::branch::alt;
    alt((sp, tab))(input)
}

// TODO: add many spaces terminated by eol
pub fn end_of_line(input: &str) -> IResult<&str, &str> {
    if input.is_empty() {
        Ok((input, input))
    } else {
        let (input, (_, eol)) = many_till(whitespace, line_ending)(input)?;
        Ok((input, eol))
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

//fn line(input: &str, term: &str) -> IResult<&str, &str> {
//    terminated(tag(term), end_of_line)(input)
//}

//pub fn parse_nodes_msh2(input: &str) -> MshResult<Vec<Point>> {
//    context("mesh header", tag("$MeshFormat"))(input)
//}
//

fn parse_node_section_msh2(input: &str) -> IResult<&str, Vec<Point>> {
    use nom::sequence::terminated;
    let (input, _) = terminated(tag("$Nodes"), end_of_line)(input)?;
    let (input, num_nodes) = terminated(parse_u64, end_of_line)(input)?;
    let (input, (nodes, _)) = many_till(parse_node_msh2, terminated(tag("$EndNodes"), end_of_line))(input)?;
    if num_nodes != nodes.len() as u64 {
        // we don't really care if the number doesn't line up for msh2
        eprintln!("warning: node header says {} nodes, but parsed {}", num_nodes, nodes.len());
    }
    Ok((input, nodes))
}

fn parse_node_msh2(input: &str) -> IResult<&str, Point> {
    do_parse!(input,
        tag: parse_u64 >> sp >>
        x: double >> sp >>
        y: double >> sp >>
        z: double >> end_of_line >>
        (
            Point { tag, x, y, z }
        )
    )
}

fn parse_u64(input: &str) -> IResult<&str, u64> {
    map_res(digit1, u64::from_str)(input)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use insta::{assert_debug_snapshot, assert_display_snapshot};

    #[test]
    fn trailing_spaces() {
        let inp = "101 0 1 100.0      \t\n";
        assert_debug_snapshot!(parse_node_msh2(inp).unwrap().1);
    }

mod msh2 {

    use super::*;

    #[test]
    fn node_msh2() {
        let inp = "1201 0 0. 1.";
        assert_debug_snapshot!(parse_node_msh2(inp).unwrap().1);
    }

    #[test]
    fn some_nodes() {
        let inp = "$Nodes\n3\n1 0. 0. 1.\n2 1. 1 1\n100 1 1 1\n$EndNodes\n";
        assert_debug_snapshot!(parse_node_section_msh2(inp).unwrap().1);
    }

    #[test]
    fn nodes_len_mismatch() {
        let inp = "$Nodes\n0\n1 0. 0. 1.\n2 1. 1 1\n100 1 1 1\n$EndNodes\n";
        assert_debug_snapshot!(parse_node_section_msh2(inp).unwrap().1);
    }

    #[test]
    fn empty_nodes() {
        let inp = "$Nodes\n0\n$EndNodes\n";
        assert_debug_snapshot!(parse_node_section_msh2(inp).unwrap().1);
    }

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
}

mod msh4 {
    use super::*;

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
    //#[test]
    //fn empty_mesh() {
    //    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    //    path.push("props");
    //    path.push("empty.msh");
    //    Msh::from_file(&path).unwrap();
    //}
}
}
