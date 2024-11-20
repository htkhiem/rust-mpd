//! The module defines LSInfo entry structs and methods.

use crate::convert::FromIter;
use crate::directory::Directory;
use crate::error::{Error, ParseError, ProtoError};
use crate::song::Song;

/// Enum over lsinfo entry types
#[derive(Debug, Clone, PartialEq)]
pub enum LsInfoEntry {
    /// A file that is an MPD-recognised song
    Song(Song),
    /// A directory
    Directory(Directory)
    // TODO: playlist
}

impl FromIter for LsInfoEntry {
    /// build song from map
    fn from_iter<I: Iterator<Item = Result<(String, String), Error>>>(mut iter: I) -> Result<LsInfoEntry, Error> {
        // Peek at the first element to see if we're dealing with a directory
        // or a song file.
        // TODO: add playlist support

        let maybe_first_elem = iter.next();
        if let Some(first_elem) = maybe_first_elem {
            if let Ok((k, v)) = first_elem {
                // We have to set dir name or song URI by ourselves since we
                // have already advanced the iterator past it.
                match k.as_str() {
                    "directory" => {
                        let mut dir = Directory::from_iter(iter)?;
                        dir.name = v;
                        return Ok(LsInfoEntry::Directory(dir));
                    },
                    "file" => {
                        let mut song = Song::from_iter(iter)?;
                        song.file = v;
                        return Ok(LsInfoEntry::Song(song));
                    },
                    _ => return Err(Error::Parse(ParseError::BadPair))
                }
            }
            return Err(Error::Proto(ProtoError::NotPair));
        }
        return Err(Error::Proto(ProtoError::NotPair));
    }
}
