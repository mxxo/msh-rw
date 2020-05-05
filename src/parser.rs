use crate::*;

use thiserror::Error;

use nom::*;
use nom::branch::alt;
use nom::bytes::complete::{tag, take_until};
use nom::combinator::map_res;
use nom::character::complete::{anychar, char, line_ending, digit1, one_of};
use nom::multi::{count, many_till};
use nom::number::complete::double;
use nom::error::context;
use nom::sequence::{delimited, preceded, terminated};

use std::path::Path;
use std::str::FromStr;

#[derive(Error, Debug)]
pub enum MshError {
    #[error("IO error ({source})")]
    Io {
        #[from]
        source: std::io::Error,
    },
    #[error("parse error:\n{source}")]
    Parse {
        #[from]
        source: nom::Err<(String, nom::error::ErrorKind)>,
    }
}

pub type MshResult<T> = std::result::Result<T, MshError>;

impl Msh {
    fn from_file<P: AsRef<Path>>(path: P) -> MshResult<Msh> {
        let (input, header) = parse_header(&first_four_lines(path)?)?;
        //match header {
        //    todo!();
        //}
        todo!()
    }
}

fn parse_msh2() -> MshResult<Msh> {
    todo!()
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
    use nom::character::complete::multispace0;
    multispace0(input)
    //let (input, nodes) = count(parse_u64_sp, num_info as usize)(input)?;
    //tag(" ")(input)
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

pub fn format_header(input: &str) -> IResult<&str, &str> {
    terminated(tag("$MeshFormat"), end_of_line)(input)
}

pub fn format_footer(input: &str) -> IResult<&str, &str> {
    terminated(tag("$EndMeshFormat"), end_of_line)(input)
}

/// Parse a `msh` file header.
pub fn mesh_header(input: &str) -> IResult<&str, MshHeader> {
    do_parse!(input,
        format_header >>
        version: alt!(tag!("2.2") | tag!("4.1")) >>
        sp >>
        binary: one_of!("01") >>
        sp >>
        size_t: one_of!("48") >>
        end_of_line >>
        endian: opt!(
            do_parse!(
                // gmsh docs say ascii int = 1 => I assume this means 4 bytes?
                endianness: take!(4) >>
                end_of_line >>
                (Some(endianness))
            )
        ) >>
        format_footer >>
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

fn parse_header(input: &str) -> MshResult<(&str, MshHeader)> {
    match mesh_header(input) {
        Ok(res) => Ok(res),
        Err(e) => Err(e.to_owned().into()),
    }
}

fn parse_node_section_msh2(input: &str) -> IResult<&str, Vec<Node>> {
    let (input, _) = terminated(tag("$Nodes"), end_of_line)(input)?;
    let (input, num_nodes) = terminated(parse_u64, end_of_line)(input)?;
    let (input, (nodes, _)) = many_till(parse_node_msh2, terminated(tag("$EndNodes"), end_of_line))(input)?;
    if num_nodes != nodes.len() as u64 {
        // we don't really care if the number doesn't line up for msh2
        eprintln!("warning: node header says {} nodes, but parsed {}", num_nodes, nodes.len());
    }
    Ok((input, nodes))
}

fn parse_node_msh2(input: &str) -> IResult<&str, Node> {
    do_parse!(input,
        tag: parse_u64 >> sp >>
        x: double >> sp >>
        y: double >> sp >>
        z: double >> sp >>
        ( Node { tag, x, y, z } )
    )
}

fn parse_physical_groups_msh2(input: &str) -> IResult<&str, Vec<PhysicalGroup>> {
    let (input, _) = terminated(tag("$PhysicalNames"), end_of_line)(input)?;
    let (input, num_groups) = terminated(parse_u64, end_of_line)(input)?;
    let (input, (groups, _)) = many_till(parse_physical_group_msh2, terminated(tag("$EndPhysicalNames"), end_of_line))(input)?;
    if num_groups != groups.len() as u64 {
        // we don't really care if the number doesn't line up for msh2
        eprintln!("warning: header says {} physical groups, but read {}", num_groups, groups.len());
    }
    Ok((input, groups))
}

fn parse_physical_group_msh2(input: &str) -> IResult<&str, PhysicalGroup> {
    let (input, dim) = parse_dimension(input)?;
    let (input, _) = sp(input)?;
    let (input, tag) = parse_u64(input)?;
    let (input, _) = sp(input)?;
    let (input, name) = delimited(char('"'), take_until("\""), char('"'))(input)?;
    let (input, _) = end_of_line(input)?;
    Ok((input, PhysicalGroup { dim, tag, name: name.to_string() }))
}

fn parse_dimension(input: &str) -> IResult<&str, Dim> {
    let (input, dim) = one_of("0123")(input)?;
    Ok((input, Dim::from_u8_unchecked(u8::from_str(&dim.to_string()).unwrap())))
}

fn parse_u64(input: &str) -> IResult<&str, u64> {
    map_res(digit1, u64::from_str)(input)
}

fn parse_u64_sp(input: &str) ->  IResult<&str, u64> {
    terminated(map_res(digit1, u64::from_str), sp)(input)
}

fn parse_elements_section_msh2(input: &str) -> IResult<&str, Vec<MeshElt>> {
    let (input, _) = terminated(tag("$Elements"), end_of_line)(input)?;
    let (input, num_elts) = terminated(parse_u64, end_of_line)(input)?;
    let (input, (elts, _)) = many_till(parse_element_msh2, terminated(tag("$EndElements"), end_of_line))(input)?;
    if num_elts != elts.len() as u64 {
        // we don't really care if the number doesn't line up for msh2
        eprintln!("warning: header says {} elements, but read {}", num_elts, elts.len());
    }
    Ok((input, elts))
}

fn parse_element_msh2(input: &str) -> IResult<&str, MeshElt> {
    let (input, tag) = terminated(parse_u64, sp)(input)?;
    let (input, label) = terminated(digit1, sp)(input)?;
    let elt_type = match MeshShape::from_gmsh_label(label) {
        Some(ty) => ty,
        None => panic!(format!("unknown mesh element type: {}", label)),
    };

    let (input, elt_info) = parse_elt_info(input)?;
    let (input, nodes) = count(parse_u64_sp, elt_type.num_nodes() as usize)(input)?;

    let uint_to_tag = |uint| if uint != 0 { Some(uint) } else { None };
    Ok((input, MeshElt {
        tag,
        ty: elt_type,
        nodes,
        // multiple physical groups are handled by duplicate shapes
        physical_group: uint_to_tag(elt_info.physical_group),
        geometry: uint_to_tag(elt_info.geometry),
    }))
}

struct EltInfo {
    pub physical_group: Tag,
    pub geometry: Tag,
    // lots of options here:
    // https://gitlab.onelab.info/gmsh/gmsh/-/blob/master/Geo/GModelIO_MSH2.cpp#L370
    //pub mesh_partition: Option<Tag>,
    //pub ghost_elements: Option<Vec<Tag>>,
    //pub domain: Option<(Tag, Tag)>,
    //pub parent_elt: Option<Tag>,
}

fn parse_elt_info(input: &str) -> IResult<&str, EltInfo> {
    let (input, num_info) = parse_u64_sp(input)?;
    if num_info > 2 {
        eprintln!("warning: only reading physical group and geometry information and skipping partitions, ghost elements...");
    }
    let (input, elt_info) = count(parse_u64_sp, num_info as usize)(input)?;
    Ok((input, EltInfo { physical_group: elt_info[0], geometry: elt_info[1] }))
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
    fn node_elt() {
        assert_debug_snapshot!(parse_element_msh2("1 15 2 0 0 5\n").unwrap().1);
    }

    #[test]
    fn line_elt() {
        assert_debug_snapshot!(parse_element_msh2("500 1 2 1 2 30 31\n").unwrap().1);
    }

    #[test]
    fn tri_elt() {
        assert_debug_snapshot!(parse_element_msh2("10 2 2 5 1 1 2 3\n").unwrap().1);
    }

    #[test]
    fn tetra_elt() {
        assert_debug_snapshot!(parse_element_msh2("41 4 2 0 1 1 2 3 4\n").unwrap().1);
    }

    #[test]
    fn elt_extra_fields() {
        assert_debug_snapshot!(parse_element_msh2("41 4 5 0 1 1 2 3 41 42 43 44\n").unwrap().1);
    }

    #[test]
    fn node_msh2() {
        let i = "1201 0 0. 1.";
        assert_debug_snapshot!(parse_node_msh2(i).unwrap().1);
    }

    #[test]
    fn pgroup_msh2() {
        let i = r#"3 1 "Water cube""#;
        assert_debug_snapshot!(parse_physical_group_msh2(i).unwrap().1);
    }

    #[test]
    fn pgroups_section() {
        assert_debug_snapshot!(parse_physical_groups_msh2(
            &r#"$PhysicalNames
4
0 1 "a point"
0 2 "hi"
3 3 "Water-cube"
2 4 "fuselage"
$EndPhysicalNames"#).unwrap().1);
    }

    #[test]
    fn bad_physical_group_dimension() {
        let res = parse_physical_group_msh2(&r#"4 1 "Water cube"#);
        assert!(res.is_err());
        if let Err(trace) = res {
            assert_display_snapshot!(trace);
        }
    }

    #[test]
    fn some_nodes() {
        let i = "$Nodes\n3\n1 0. 0. 1.\n2 1. 1 1\n100 1 1 1\n$EndNodes\n";
        assert_debug_snapshot!(parse_node_section_msh2(i).unwrap().1);
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
        let header_str = first_four_lines(path).unwrap();
        let res = parse_header(&header_str);
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
