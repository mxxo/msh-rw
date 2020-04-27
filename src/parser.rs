use crate::*;
use std::path::PathBuf;
use thiserror::Error;

use nom::*;
use nom::bytes::complete::tag;

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

fn parse_header(input: &str) -> IResult<&str, ()> {
    let (input, _) = tag("$MeshFormat\r\n2.2 0 8\r\n$EndMeshFormat")(input)?;
    Ok((input, ()))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn empty_mesh() {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("props");
        path.push("empty.msh");
        Msh2::from_file(&path).unwrap();
    }
}
