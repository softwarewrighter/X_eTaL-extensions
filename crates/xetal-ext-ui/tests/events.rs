//! The program-side event queue, headless.

use std::time::{Duration, Instant};

use xetal_ext_ui::{Events, hosted, on_main};

#[test]
fn events_arrive_in_order_and_ticks_fill_the_gaps() {
    let e = Events::new();
    let main = e.clone();
    let t = std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(30));
        main.push("key Space");
        main.push("close");
    });
    let start = Instant::now();
    assert_eq!(e.next(Duration::from_millis(5)), "frame");
    assert!(start.elapsed() >= Duration::from_millis(5));
    assert_eq!(e.next(Duration::from_secs(5)), "key Space");
    assert_eq!(e.next(Duration::from_secs(5)), "close");
    t.join().unwrap();
}

#[test]
fn the_queue_keeps_the_newest() {
    let e = Events::new();
    for i in 0..300 {
        e.push(format!("e{i}"));
    }
    assert_eq!(e.next(Duration::ZERO), "e44");
}

#[test]
fn without_a_host_a_job_is_an_error() {
    assert!(!hosted());
    let err = on_main(|_| 1).unwrap_err();
    assert!(err.starts_with("no UI host"), "{err}");
}

#[test]
fn scripted_events_are_all_kept() {
    // SAFETY: no other test in this binary reads these variables.
    #[allow(unsafe_code)]
    unsafe {
        std::env::set_var(
            "XETAL_EVENTS",
            (0..1000)
                .map(|i| format!("e{i}"))
                .collect::<Vec<_>>()
                .join(","),
        );
    }
    let e = xetal_ext_ui::scripted();
    assert_eq!(e.next(Duration::ZERO), "e0");
    for _ in 1..1000 {
        e.next(Duration::ZERO);
    }
    assert_eq!(e.next(Duration::ZERO), "close");
}
