use crate::*;
use std::path::PathBuf;
use nom::*;
use thiserror::Error;

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
        todo!();
    }
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
