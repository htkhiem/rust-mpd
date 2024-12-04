//! The module defines MPD status data structures

use crate::convert::FromIter;
use crate::error::{Error, ParseError};
use crate::song::{Id, QueuePlace};

use std::fmt;
use std::str::FromStr;
use std::time::Duration;

/// MPD status
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, PartialEq, Clone, Default)]
pub struct Status {
    /// volume (0-100, or -1 if volume is unavailable (e.g. for HTTPD output type)
    pub volume: i8,
    /// repeat mode
    pub repeat: bool,
    /// random mode
    pub random: bool,
    /// single mode
    pub single: bool,
    /// consume mode
    pub consume: bool,
    /// queue version number
    pub queue_version: u32,
    /// queue length
    pub queue_len: u32,
    /// playback state
    pub state: State,
    /// currently playing song place in the queue
    pub song: Option<QueuePlace>,
    /// next song to play place in the queue
    pub nextsong: Option<QueuePlace>,
    /// time current song played, and total song duration (in seconds resolution)
    pub time: Option<(Duration, Duration)>,
    /// elapsed play time current song played (in milliseconds resolution)
    pub elapsed: Option<Duration>,
    /// current song duration
    pub duration: Option<Duration>,
    /// current song bitrate, kbps
    pub bitrate: Option<u32>,
    /// crossfade timeout, seconds
    pub crossfade: Option<Duration>,
    /// mixramp threshold, dB
    pub mixrampdb: f32,
    /// mixramp duration, seconds
    pub mixrampdelay: Option<Duration>,
    /// current audio playback format
    pub audio: Option<AudioFormat>,
    /// current DB updating job number (if DB updating is in progress)
    pub updating_db: Option<u32>,
    /// last player error (if happened, can be reset with
    /// [`clearerror()`](crate::Client::clearerror) method)
    pub error: Option<String>,
    /// replay gain mode
    pub replaygain: Option<ReplayGain>,
}

impl FromIter for Status {
    fn from_iter<I: Iterator<Item = Result<(String, String), Error>>>(iter: I) -> Result<Status, Error> {
        let mut result = Status::default();

        for res in iter {
            let line = res?;
            match &*line.0 {
                "volume" => result.volume = line.1.parse()?,

                "repeat" => result.repeat = &*line.1 == "1",
                "random" => result.random = &*line.1 == "1",
                "single" => result.single = &*line.1 == "1",
                "consume" => result.consume = &*line.1 == "1",

                "playlist" => result.queue_version = line.1.parse()?,
                "playlistlength" => result.queue_len = line.1.parse()?,
                "state" => result.state = line.1.parse()?,
                "songid" => match result.song {
                    None => result.song = Some(QueuePlace { id: Id(line.1.parse()?), pos: 0, prio: 0 }),
                    Some(ref mut place) => place.id = Id(line.1.parse()?),
                },
                "song" => match result.song {
                    None => result.song = Some(QueuePlace { pos: line.1.parse()?, id: Id(0), prio: 0 }),
                    Some(ref mut place) => place.pos = line.1.parse()?,
                },
                "nextsongid" => match result.nextsong {
                    None => result.nextsong = Some(QueuePlace { id: Id(line.1.parse()?), pos: 0, prio: 0 }),
                    Some(ref mut place) => place.id = Id(line.1.parse()?),
                },
                "nextsong" => match result.nextsong {
                    None => result.nextsong = Some(QueuePlace { pos: line.1.parse()?, id: Id(0), prio: 0 }),
                    Some(ref mut place) => place.pos = line.1.parse()?,
                },
                "time" => {
                    let mut splits = line.1.splitn(2, ':').map(|v| v.parse().map_err(ParseError::BadInteger).map(Duration::from_secs));
                    result.time = match (splits.next(), splits.next()) {
                        (Some(Ok(a)), Some(Ok(b))) => Ok(Some((a, b))),
                        (Some(Err(e)), _) | (_, Some(Err(e))) => Err(e),
                        _ => Ok(None),
                    }?;
                }
                "elapsed" => result.elapsed = Some(Duration::try_from_secs_f64(line.1.parse()?)?),
                "duration" => result.duration = Some(Duration::try_from_secs_f64(line.1.parse()?)?),
                "bitrate" => result.bitrate = Some(line.1.parse()?),
                "xfade" => result.crossfade = Some(Duration::from_secs(line.1.parse()?)),
                "mixrampdb" => result.mixrampdb = line.1.parse::<f32>()?,
                "mixrampdelay" => result.mixrampdelay = Some(Duration::from_secs_f64(line.1.parse()?)),
                "audio" => result.audio = Some(line.1.parse()?),
                "updating_db" => result.updating_db = Some(line.1.parse()?),
                "error" => result.error = Some(line.1.to_owned()),
                "replay_gain_mode" => result.replaygain = Some(line.1.parse()?),
                _ => (),
            }
        }

        Ok(result)
    }
}

/// Audio playback format
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct AudioFormat {
    /// Sample rate, kbps.
    /// For DSD, to align with MPD's internal handling, the returned rate will be in kilobytes per second instead.
    /// See https://mpd.readthedocs.io/en/latest/user.html#audio-output-format.
    pub rate: u32,
    /// Sample resolution in bits, can be 0 for floating point resolution or 1 for DSD.
    pub bits: u8,
    /// Number of channels.
    pub chans: u8,
}

impl FromStr for AudioFormat {
    type Err = ParseError;
    fn from_str(s: &str) -> Result<AudioFormat, ParseError> {
        if s.contains("dsd") {
            // DSD format string only contains two terms: "dsd..." and number of channels.
            // To shoehorn into our current AudioFormat struct, use the following conversion:
            // - Sample rate: 44100 * the DSD multiplier / 8. For example, DSD64 is sampled at 2.8224MHz.
            // - Bits: 1 (DSD is a sequence of single-bit values, or PDM).
            // - Channels: as usual.
            let mut it = s.split(':');
            let dsd_mul: u32 = it.next().ok_or(ParseError::NoRate).and_then(|v| v[3..].parse().map_err(ParseError::BadRate))?;
            return Ok(AudioFormat {
                rate: dsd_mul * 44100 / 8,
                bits: 1,
                chans: it.next().ok_or(ParseError::NoChans).and_then(|v| v.parse().map_err(ParseError::BadChans))?,
            });
        }
        let mut it = s.split(':');
        Ok(AudioFormat {
            rate: it.next().ok_or(ParseError::NoRate).and_then(|v| v.parse().map_err(ParseError::BadRate))?,
            bits: it.next().ok_or(ParseError::NoBits).and_then(
                |v| {
                    if v == "f" {
                        Ok(0)
                    } else {
                        v.parse().map_err(ParseError::BadBits)
                    }
                },
            )?,
            chans: it.next().ok_or(ParseError::NoChans).and_then(|v| v.parse().map_err(ParseError::BadChans))?,
        })
    }
}

/// Playback state
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize), serde(rename_all = "lowercase"))]
#[derive(Default, Debug, Copy, Clone, PartialEq)]
pub enum State {
    /// player stopped
    #[default]
    Stop,
    /// player is playing
    Play,
    /// player paused
    Pause,
}

impl FromStr for State {
    type Err = ParseError;
    fn from_str(s: &str) -> Result<State, ParseError> {
        match s {
            "stop" => Ok(State::Stop),
            "play" => Ok(State::Play),
            "pause" => Ok(State::Pause),
            _ => Err(ParseError::BadState(s.to_owned())),
        }
    }
}

/// Replay gain mode
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize), serde(rename_all = "lowercase"))]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ReplayGain {
    /// off
    Off,
    /// track
    Track,
    /// album
    Album,
    /// auto
    Auto,
}

impl FromStr for ReplayGain {
    type Err = ParseError;
    fn from_str(s: &str) -> Result<ReplayGain, ParseError> {
        use self::ReplayGain::*;
        match s {
            "off" => Ok(Off),
            "track" => Ok(Track),
            "album" => Ok(Album),
            "auto" => Ok(Auto),
            _ => Err(ParseError::BadValue(s.to_owned())),
        }
    }
}

impl fmt::Display for ReplayGain {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::ReplayGain as R;
        f.write_str(match *self {
            R::Off => "off",
            R::Track => "track",
            R::Album => "album",
            R::Auto => "auto",
        })
    }
}
