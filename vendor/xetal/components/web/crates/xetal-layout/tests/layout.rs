//! Where the panes split, as a dragged divider moves it.

use xetal_layout::{Axis, Split};

#[test]
fn the_default_split_is_even_columns_and_a_third_for_the_output() {
    let split = Split::default();
    assert_eq!((split.source, split.output), (50.0, 32.0));
    assert_eq!(split.style(), "--source: 50%; --output: 32%;");
}

#[test]
fn a_column_divider_gives_the_source_the_width_left_of_the_pointer() {
    // The panes span x = 100..900; the pointer at 300 is a quarter in.
    let split = Split::default().dragged(Axis::Columns, 300.0, 100.0, 800.0);
    assert_eq!(split.source, 25.0);
    assert_eq!(split.output, 32.0);
}

#[test]
fn a_row_divider_gives_the_output_the_height_below_the_pointer() {
    // The panes span y = 50..650; the pointer at 500 leaves 150 below.
    let split = Split::default().dragged(Axis::Rows, 500.0, 50.0, 600.0);
    assert_eq!(split.output, 25.0);
    assert_eq!(split.source, 50.0);
}

#[test]
fn a_split_never_hides_a_pane() {
    let wide = Split::default().dragged(Axis::Columns, 5000.0, 0.0, 1000.0);
    let narrow = Split::default().dragged(Axis::Columns, -50.0, 0.0, 1000.0);
    assert_eq!((wide.source, narrow.source), (90.0, 10.0));
    let tall = Split::default().dragged(Axis::Rows, 0.0, 0.0, 1000.0);
    assert_eq!(tall.output, 90.0);
}

#[test]
fn a_split_with_no_size_to_measure_is_kept() {
    let split = Split::default().dragged(Axis::Columns, 10.0, 0.0, 0.0);
    assert_eq!(split, Split::default());
}

#[test]
fn a_split_is_saved_and_read_back_as_text() {
    let split = Split {
        source: 62.5,
        output: 20.0,
    };
    assert_eq!(split.to_text(), "62.5 20");
    assert_eq!(Split::from_text("62.5 20"), split);
    assert_eq!(Split::from_text("rubbish"), Split::default());
    assert_eq!(
        Split::from_text("99 1"),
        Split {
            source: 90.0,
            output: 10.0
        }
    );
}
