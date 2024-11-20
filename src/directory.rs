//! The module defines directory structs and methods.

use crate::convert::FromIter;
use crate::error::Error;

/// MPD-recognised directory.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Directory {
    /// directory name
    pub name: String,
    /// last modification time
    pub last_mod: Option<String>
}

impl FromIter for Directory {
    /// build from map
    fn from_iter<I: Iterator<Item = Result<(String, String), Error>>>(iter: I) -> Result<Directory, Error> {
        let mut result = Directory::default();

        for res in iter {
            let line = res?;
            match &*line.0 {
                "directory" => result.name = line.1.to_owned(),
                "Last-Modified" => result.last_mod = Some(line.1.to_owned()),
                _ => {}
            }
        }

        Ok(result)
    }
}
