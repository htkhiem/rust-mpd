//! The module defines structures and protocols for asynchronous MPD communication
//!
//! The MPD supports very simple protocol for asynchronous client notifications about
//! different player events. First user issues [`idle`](Client::idle) command
//! with optional argument to filter events by source subsystem (like
//! "database", "player", "mixer" etc.)
//!
//! Once in "idle" mode, client connection timeout is disabled, and MPD will notify
//! client about next event when one occurs (if originated from one of designated
//! subsystems, if specified).
//!
//! (Actually MPD notifies only about general subsystem source of event, e.g.
//! if user changed volume, client will get [`mixer`](Subsystem::Mixer) event
//! in idle mode, so it should issue [`status`](Client::status) command then and
//! check for any mixer-related field changes.)
//!
//! Once some such event occurs, and client is notified about it, idle mode is interrupted,
//! and client must issue another `idle` command to continue listening for interesting
//! events.
//!
//! While in "idle" mode, client can't issue any commands, except for special `noidle`
//! command, which interrupts "idle" mode, and provides a list queued events
//! since last `idle` command, if they occurred.
//!
//! The module describes subsystems enum only, but the main workflow is determined by
//! [`IdleGuard`] struct, which catches mutable reference
//! to original [`Client`] struct, thus enforcing MPD contract in regards of (im)possibility
//! to send commands while in "idle" mode.

use crate::client::Client;
use crate::error::{Error, ParseError};
use crate::proto::Proto;

use std::fmt;
use std::io::{Read, Write};
use std::mem::forget;
use std::str::FromStr;

/// Subsystems for `idle` command
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Subsystem {
    /// database: the song database has been modified after update.
    Database,
    /// update: a database update has started or finished.
    /// If the database was modified during the update, the database event is also emitted.
    Update,
    /// stored_playlist: a stored playlist has been modified, renamed, created or deleted
    Playlist,
    /// playlist: the current playlist has been modified
    Queue,
    /// player: the player has been started, stopped or seeked
    Player,
    /// mixer: the volume has been changed
    Mixer,
    /// output: an audio output has been enabled or disabled
    Output,
    /// options: options like repeat, random, crossfade, replay gain
    Options,
    /// partition: a partition was added, removed or changed
    Partition,
    /// sticker: the sticker database has been modified.
    Sticker,
    /// subscription: a client has subscribed or unsubscribed to a channel
    Subscription,
    /// message: a message was received on a channel this client is subscribed to; this event is only emitted when the queue is empty
    Message,
    /// neighbor: a neighbor was found or lost
    Neighbor,
    /// mount: the mount list has changed
    Mount,
}

impl FromStr for Subsystem {
    type Err = ParseError;
    fn from_str(s: &str) -> Result<Subsystem, ParseError> {
        use self::Subsystem::*;
        match s {
            "database" => Ok(Database),
            "update" => Ok(Update),
            "stored_playlist" => Ok(Playlist),
            "playlist" => Ok(Queue),
            "player" => Ok(Player),
            "mixer" => Ok(Mixer),
            "output" => Ok(Output),
            "options" => Ok(Options),
            "partition" => Ok(Partition),
            "sticker" => Ok(Sticker),
            "subscription" => Ok(Subscription),
            "message" => Ok(Message),
            "neighbor" => Ok(Neighbor),
            "mount" => Ok(Mount),
            _ => Err(ParseError::BadValue(s.to_owned())),
        }
    }
}

impl Subsystem {
    fn to_str(self) -> &'static str {
        use self::Subsystem as S;
        match self {
            S::Database => "database",
            S::Update => "update",
            S::Playlist => "stored_playlist",
            S::Queue => "playlist",
            S::Player => "player",
            S::Mixer => "mixer",
            S::Output => "output",
            S::Options => "options",
            S::Partition => "partition",
            S::Sticker => "sticker",
            S::Subscription => "subscription",
            S::Message => "message",
            S::Neighbor => "neighbor",
            S::Mount => "mount",
        }
    }
}

impl fmt::Display for Subsystem {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self.to_str())
    }
}

use std::result::Result as StdResult;
impl crate::proto::ToArguments for Subsystem {
    fn to_arguments<F, E>(&self, f: &mut F) -> StdResult<(), E>
    where F: FnMut(&str) -> StdResult<(), E> {
        f(self.to_str())
    }
}

/// "Idle" mode guard enforcing MPD asynchronous events protocol
pub struct IdleGuard<'a, S: 'a + Read + Write>(&'a mut Client<S>);

impl<'a, S: 'a + Read + Write> IdleGuard<'a, S> {
    /// Get list of subsystems with new events, interrupting idle mode in process
    pub fn get(self) -> Result<Vec<Subsystem>, Error> {
        let result = self.0.read_list("changed").and_then(|v| v.into_iter().map(|b| b.parse().map_err(From::from)).collect());
        forget(self);
        result
    }
}

impl<'a, S: 'a + Read + Write> Drop for IdleGuard<'a, S> {
    fn drop(&mut self) {
        let _ = self.0.run_command("noidle", ()).map(|_| self.0.drain());
    }
}

/// This trait implements `idle` command of MPD protocol
///
/// See module's documentation for details.
pub trait Idle {
    /// Stream type of a client
    type Stream: Read + Write;

    /// Start listening for events from a set of subsystems
    ///
    /// If empty subsystems slice is given, wait for all event from any subsystem.
    ///
    /// This method returns `IdleGuard`, which takes mutable reference of an initial client,
    /// thus disallowing any operations on this mpd connection.
    ///
    /// You can call `.get()` method of this struct to stop waiting and get all queued events
    /// matching given subsystems filter. This call consumes a guard, stops waiting
    /// and releases client object.
    ///
    /// If the guard goes out of scope, wait lock is released as well, but all queued events
    /// will be silently ignored.
    fn idle<'a>(&'a mut self, subsystems: &[Subsystem]) -> Result<IdleGuard<'a, Self::Stream>, Error>;

    /// Wait for events from a set of subsystems and return list of affected subsystems
    ///
    /// This is a blocking operation. If empty subsystems slice is given,
    /// wait for all event from any subsystem.
    fn wait(&mut self, subsystems: &[Subsystem]) -> Result<Vec<Subsystem>, Error> {
        self.idle(subsystems).and_then(IdleGuard::get)
    }
}

impl<S: Read + Write> Idle for Client<S> {
    type Stream = S;
    fn idle<'a>(&'a mut self, subsystems: &[Subsystem]) -> Result<IdleGuard<'a, S>, Error> {
        self.run_command("idle", subsystems)?;
        Ok(IdleGuard(self))
    }
}
