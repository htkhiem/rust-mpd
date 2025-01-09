//! The module defines playlist data structures

use crate::convert::FromMap;
use crate::error::{Error, ProtoError};
use crate::proto::*;

use std::borrow::Cow;
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

/// Edit actions, for packing multiple edit actions into one command list.
/// Note that commands will still be executed in the passed order, so later
/// commands must refer to song positions as modified by preceding ones.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub enum EditAction<'a> {
    /// Add a new song.
    ///
    /// Fields: playlist name, song URI, optional position in playlist.
    Add(Cow<'a, str>, Cow<'a, str>, Option<u32>),
    /// Clear all songs in this playlist.
    Clear(Cow<'a, str>),
    /// Move song from one position to another, such that it will be at the given new
    /// position. For example, moving song at pos 0 to 1 means the first song will swap
    /// places with the next one.
    ///
    /// Fields: playlist name, old position, new position.
    Move(Cow<'a, str>, u32, u32),
    /// Remove the song at the given position from the playlist of the given name.
    ///
    /// Fields: playlist name, position to remove.
    Delete(Cow<'a, str>, u32)
}

impl<'a> ToArguments for EditAction<'a> {
    fn to_arguments<F, E>(&self, f: &mut F) -> Result<(), E>
    where F: FnMut(&str) -> Result<(), E> {
        // This will only write the arguments, not the command, of this action
        match self {
            Self::Add(name, uri, opt_pos) => {
                if let Some(pos) = opt_pos {
                    (name, uri, *pos).to_arguments(f)
                }
                else {
                    (name, uri).to_arguments(f)
                }
            },
            Self::Clear(name) => name.to_arguments(f),
            Self::Move(name, old, new) => (name, *old, *new).to_arguments(f),
            Self::Delete(name, pos) => (name, *pos).to_arguments(f)
        }
    }
}

impl<'a> EditAction<'a> {
    /// Get the corresponding command word
    pub fn command(&self) -> &'static str {
        match self {
            Self::Add(_, _, _) => "playlistadd",
            Self::Clear(_) => "playlistclear",
            Self::Move(_, _, _) => "playlistmove",
            Self::Delete(_, _) => "playlistdelete"
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
