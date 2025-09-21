//! The module defines a tag grouped values struct for parsing grouped list calls.

use crate::{error::Result, proto::Pairs};

struct Group {
    pub key: String,
    pub contents: Vec<String>
}

/// Values as returned by the `list` command, optionally grouped.
pub struct GroupedValues {
    /// Vector of groups. Each group is a (key, contents) pair where
    /// key is the value of the grouping condition for that group and
    /// contents is a vector of unique values of a tag/attribute under
    /// that group.
    pub groups: Vec<(String, Vec<String>)>
}

impl<'a> GroupedValues {
    /// Parse a grouped list call response. sep MUST be lowercase.
    pub fn from_pairs_with_sep<I>(pairs: &'a mut Pairs<I>, sep: &'a str) -> Result<Self>
    where I: Iterator<Item = std::io::Result<String>> {
        let mut groups: Vec<(String, Vec<String>)> = Vec::new();

        let mut curr_group: Option<Group> = None;

        loop {
            match pairs.next() {
                Some(Ok((a, b))) => {
                    if &*a.to_lowercase() == sep {
                        // Flush current group into main struct
                        if let Some(group) = curr_group {
                            groups.push((group.key, group.contents));
                        }
                        curr_group = Some(Group {key: b, contents: Vec::new()});
                    } else if let Some(group) = curr_group.as_mut() {
                        group.contents.push(b);
                    }
                }
                Some(Err(e)) => {return Err(e);},
                None => {
                    // Flush current group into main struct
                    if let Some(group) = curr_group.take() {
                        groups.push((group.key, group.contents));
                    }
                    break;
                }
            }
        }

        Ok(Self { groups })
    }
}
