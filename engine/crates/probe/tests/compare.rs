// SPDX-License-Identifier: Apache-2.0
//! `headwater probe compare` renders each difference through one function,
//! and that function takes counts (#1820).
//!
//! The rows below are the differences that the screening evaluation of
//! 2026-10-18 published, computed then by hand from `Interval::minus`. The
//! test calls the same `Interval::wilson`, `Interval::minus` and line renderer
//! that the verb calls, so a hand copy that drifts from either, or a per-probe
//! Wilson interval that drifts from `Results::rate`, turns it red.

use headwater_probe::grade::{compared, Interval};

#[test]
fn the_published_pooled_rows_come_out_of_the_renderer_exactly() {
    assert_eq!(
        compared("pooled", (39, 72), (11, 72)),
        "pooled: 39/72 against 11/72, +38.9 points [+23.7, +51.7]"
    );
    assert_eq!(
        compared("pooled", (40, 72), (39, 72)),
        "pooled: 40/72 against 39/72, +1.4 points [-14.5, +17.2]"
    );
}

#[test]
fn the_per_probe_cells_of_scenario_1_against_scenario_0_come_out_exactly() {
    let cases = [
        ((8, 18), (0, 18), "+44.4 points [+17.9, +66.3]"),
        ((9, 18), (0, 18), "+50.0 points [+22.6, +71.0]"),
        ((18, 18), (10, 18), "+44.4 points [+17.9, +66.3]"),
        ((4, 18), (1, 18), "+16.7 points [-7.5, +40.1]"),
    ];
    for (treated, control, difference) in cases {
        let line = compared("HW-PROBE-x", treated, control);
        let expected = format!(
            "HW-PROBE-x: {}/{} against {}/{}, {difference}",
            treated.0, treated.1, control.0, control.1
        );
        assert_eq!(line, expected);
    }
}

#[test]
fn a_side_that_graded_nothing_has_no_interval_and_says_so() {
    assert_eq!(Interval::wilson(0, 0), None);
    let line = compared("HW-PROBE-x", (0, 0), (3, 18));
    assert_eq!(
        line,
        "HW-PROBE-x: 0/0 against 3/18, no difference: the treated transcript graded no session"
    );
    let line = compared("HW-PROBE-x", (3, 18), (0, 0));
    assert_eq!(
        line,
        "HW-PROBE-x: 3/18 against 0/0, no difference: the control transcript graded no session"
    );
}

#[test]
fn the_wilson_interval_is_the_one_a_rate_reports() {
    // 11 of 72: the pooled rate of scenario 0, as its result page states it.
    let interval = Interval::wilson(11, 72).expect("72 graded sessions have an interval");
    assert!((interval.point - 11.0 / 72.0).abs() < 1e-12);
    assert!(interval.low > 0.0 && interval.high < 1.0);
    assert!(interval.low < interval.point && interval.point < interval.high);
}
