//! What the higher-order built-ins cost (Saga 30), per operand call:
//! the machine's transitions (counted one per slice) and the heap
//! allocations (counted by this test binary's allocator) - deterministic
//! guards where timings would not be. The bounds are the costs measured
//! when the guard was added, lowered as the fixes land; a change that
//! makes a built-in dearer per element fails here.

use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering};

use xetal_arith::Rng;
use xetal_step::{Machine, Status};

/// The system allocator, counting allocations.
struct Counting;

static ALLOCATIONS: AtomicUsize = AtomicUsize::new(0);

unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        ALLOCATIONS.fetch_add(1, Ordering::Relaxed);
        // SAFETY: forwards to the system allocator unchanged.
        unsafe { System.alloc(layout) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        // SAFETY: forwards to the system allocator unchanged.
        unsafe { System.dealloc(ptr, layout) }
    }
}

#[global_allocator]
static COUNTING: Counting = Counting;

/// Allocations a whole run of `src` makes (tests run one at a time
/// here: a single test function measures everything).
fn allocations(src: &str) -> usize {
    let program = xetal_core::lower(src).expect("lowers");
    let mut out = Vec::new();
    let before = ALLOCATIONS.load(Ordering::Relaxed);
    Machine::new(&program, &mut out, Rng::seeded(1))
        .finish()
        .expect("runs");
    ALLOCATIONS.load(Ordering::Relaxed) - before
}

/// Transitions `src` takes.
fn transitions(src: &str) -> usize {
    let program = xetal_core::lower(src).expect("lowers");
    let mut out = Vec::new();
    let mut machine = Machine::new(&program, &mut out, Rng::seeded(1));
    let mut count = 0;
    while machine.run(1).expect("runs") == Status::Running {
        count += 1;
    }
    count
}

/// Per operand call: `cost` of `make(n)` at two sizes, the difference
/// divided by the difference in calls.
fn per_call(
    cost: fn(&str) -> usize,
    make: impl Fn(usize) -> String,
    calls: impl Fn(usize) -> usize,
) -> f64 {
    let (a, b) = (cost(&make(10)), cost(&make(40)));
    (b - a) as f64 / (calls(40) - calls(10)) as f64
}

/// Most transitions and allocations allowed per operand call, by case
/// (the costs measured when the guard was added; Saga 30 lowers them:
/// a first-order built-in operand is called at once, step 047; lean
/// kernels allocate nothing per element, step 051).
const BOUNDS: [(&str, f64, f64); 5] = [
    ("table prim", 0.05, 0.1),
    ("table lambda", 10.2, 7.2),
    ("each lambda", 10.1, 7.1),
    ("reduce prim", 0.05, 1.1),
    ("inner prim", 0.05, 0.3),
];

/// A case: its name, its program at size n, and its operand calls.
type Case = (
    &'static str,
    Box<dyn Fn(usize) -> String>,
    Box<dyn Fn(usize) -> usize>,
);

/// The programs measured, and how many operand calls each makes.
fn cases() -> Vec<Case> {
    vec![
        (
            "table prim",
            Box::new(|n| format!("x := (r_ange {n}) '* t_able r_ange 20")),
            Box::new(|n| n * 20),
        ),
        (
            "table lambda",
            Box::new(|n| format!("x := (r_ange {n}) '{{ _l * _r }} t_able r_ange 20")),
            Box::new(|n| n * 20),
        ),
        (
            "each lambda",
            Box::new(|n| format!("x := '{{ _r * 2 }} e_ach r_ange {}", n * 20)),
            Box::new(|n| n * 20),
        ),
        (
            "reduce prim",
            Box::new(|n| format!("x := '+ r_/ r_ange {}", n * 20)),
            Box::new(|n| n * 20),
        ),
        (
            "inner prim",
            Box::new(|n| format!("x := ({n} 8 r_eshape 1) '+ '* i_nner 8 20 r_eshape 1")),
            Box::new(|n| n * 20 * 8),
        ),
    ]
}

#[test]
fn costs_per_operand_call() {
    for (name, make, calls) in cases() {
        let t = per_call(transitions, &make, &calls);
        let a = per_call(allocations, &make, &calls);
        println!("{name}: {t:.2} transitions, {a:.2} allocations per call");
        let (_, max_t, max_a) = BOUNDS
            .iter()
            .find(|b| b.0 == name)
            .copied()
            .unwrap_or((name, 0.0, 0.0));
        assert!(
            t <= max_t,
            "{name}: {t:.2} transitions per call, more than {max_t}"
        );
        assert!(
            a <= max_a,
            "{name}: {a:.2} allocations per call, more than {max_a}"
        );
    }
}
