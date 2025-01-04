//! The module defines playlist data structures

use crate::convert::FromMap;
use crate::error::{Error, ProtoError};

use std::collections::BTreeMap;

/// Save mode when calling save().
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub enum SaveMode {
    /// Return error if there is an existing playlist with the same name
    Create,
    /// Replace any existing playlist with the given name. If none exists, return an error.
    Replace,
    /// Append to any existing playlist with the given name. If none exists, return an error.
    Append
}

impl SaveMode {
    /// Return a string representation of this save mode to be used in constructing the save command.
    ///
    /// Requires MPD 0.23.15+.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Create => "create",
            Self::Replace => "replace",
            Self::Append => "append"
        }
    }
}

/// Playlist
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct Playlist {
    /// name
    pub name: String,
    /// last modified
    pub last_mod: String,
}

impl FromMap for Playlist {
    fn from_map(map: BTreeMap<String, String>) -> Result<Playlist, Error> {
        Ok(Playlist {
            name: map.get("playlist").map(|v| v.to_owned()).ok_or(Error::Proto(ProtoError::NoField("playlist")))?,
            last_mod: map.get("Last-Modified").map(|v| v.to_owned()).ok_or(Error::Proto(ProtoError::NoField("Last-Modified")))?,
        })
    }
}
