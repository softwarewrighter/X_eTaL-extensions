//! The messages between the live demo's page and the worker that runs
//! programs: plain text, so they round-trip whatever they hold.

use xetal_runner::{Event, Mode, Request};

#[test]
fn a_request_round_trips() {
    let req = Request {
        src: "x := 1\n\"a:b\" c_at \"5:x\"".into(),
        seed: 42,
        mode: Mode::Notebook(None),
        boxed: true,
        files: vec![
            (
                "Hello.xtl".into(),
                "l:h_ello := { @ -> \"hi X\u{332}\" }\n".into(),
            ),
            ("work/m.txt".into(), String::new()),
        ],
    };
    assert_eq!(Request::decode(&req.encode()), Some(req.clone()));
    for mode in [Mode::Run, Mode::Notebook(Some(3))] {
        let req = Request {
            mode,
            boxed: false,
            ..req.clone()
        };
        assert_eq!(Request::decode(&req.encode()), Some(req));
    }
}

#[test]
fn every_event_round_trips() {
    for e in [
        Event::Out("2 730".into()),
        Event::Out(String::new()),
        Event::Err("error[domain]: 3:x".into()),
        Event::Picture("<svg>\u{2190}</svg>".into()),
        Event::Wrote("work/tttml.model".into(), "1 2\n3".into()),
        Event::Done,
        Event::Ready,
        Event::Source("# a comment\nx := 1".into()),
    ] {
        assert_eq!(Event::decode(&e.encode()), Some(e.clone()), "{e:?}");
    }
}

#[test]
fn what_is_not_a_message_is_none() {
    for bad in ["", "x", "3:ab", "9:short", "1:z", "1:w3:abc"] {
        assert_eq!(Event::decode(bad), None, "{bad:?}");
    }
    assert_eq!(Request::decode("1:x"), None);
}
