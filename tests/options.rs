extern crate mpd;

mod helpers;
use helpers::connect;
use mpd::{Idle, Song, State, Subsystem};
use std::time::Duration;

#[test]
fn status() {
    let mut mpd = connect();
    let status = mpd.status().unwrap();

    assert_eq!(status.song, None);
    assert_eq!(status.state, State::Stop);
}

#[test]
fn status_during_playback() {
    let mut mpd = connect();
    mpd.push(Song { file: "silence.flac".to_string(), ..Song::default() }).expect("adding song to queue should not fail");
    mpd.play().expect("starting playback should not fail");
    mpd.idle(&[Subsystem::Player]).expect("waiting for playback should not fail");

    let status = mpd.status().expect("getting status should not fail");

    assert!(status.song.is_some());
    assert_eq!(status.state, State::Play);
    assert_eq!(status.duration, Some(Duration::from_millis(500)));
}

#[test]
fn stats() {
    let mut mpd = connect();
    let stats = mpd.stats().unwrap();
    println!("{:?}", stats);
}

macro_rules! test_options_impl {
    ($name:ident, $val1:expr, $tval1:expr, $val2:expr, $tval2:expr) => {
        #[test]
        fn $name() {
            let mut mpd = connect();
            mpd.$name($val1).unwrap();
            assert_eq!(mpd.status().unwrap().$name, $tval1);
            mpd.$name($val2).unwrap();
            assert_eq!(mpd.status().unwrap().$name, $tval2);
        }
    };
}

macro_rules! test_option {
    ($name:ident, $val1:expr, $val2:expr) => {
        test_options_impl!($name, $val1, $val1, $val2, $val2);
    };
    ($name:ident, $val1:expr => $tval1:expr, $val2:expr => $tval2:expr) => {
        test_options_impl!($name, $val1, $tval1, $val2, $tval2);
    };
}

test_option!(consume, true, false);
test_option!(single, true, false);
test_option!(random, true, false);
test_option!(repeat, true, false);
// test_option!(mixrampdb, 1.0f32, 0.0f32);
// test_option!(mixrampdelay, 1 => Some(Duration::from_secs(1)), 0 => None);

#[test]
fn volume() {
    let mut mpd = connect();
    if mpd.status().unwrap().volume >= 0 {
        mpd.volume(100).unwrap();
        assert_eq!(mpd.status().unwrap().volume, 100);
        mpd.volume(0).unwrap();
        assert_eq!(mpd.status().unwrap().volume, 0);
    }
}

#[test]
fn crossfade() {
    let mut mpd = connect();
    mpd.crossfade(1000).unwrap();
    assert_eq!(mpd.status().unwrap().crossfade, Some(Duration::from_secs(1000)));
    mpd.crossfade(0).unwrap();
    assert_eq!(mpd.status().unwrap().crossfade, if mpd.version >= mpd::Version(0, 19, 0) { None } else { Some(Duration::from_secs(0)) });
}
