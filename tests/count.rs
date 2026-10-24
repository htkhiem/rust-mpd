extern crate mpd;

mod helpers;
use helpers::connect;
use mpd::{Query, Term};

fn match_all<'a>() -> Query<'a> {
    let mut q = Query::new();
    q.and(Term::Base, "");
    q
}

// The fixture library (tests/data/silence.flac) contains exactly one song,
// and `Daemon::start()` waits for MPD's initial library scan to finish
// before returning, so the DB should hold exactly 1 song.
#[test]
fn count() {
    let mut mpd = connect();
    assert_eq!(mpd.count(&match_all(), None).expect("count should succeed"), 1);
}

#[test]
fn count_grouped() {
    let mut mpd = connect();
    // Grouped reply has one `songs:` line per group value; the sum is the
    // total number of matching songs.
    assert_eq!(mpd.count(&match_all(), Some("artist")).expect("count should succeed"), 1);
}
