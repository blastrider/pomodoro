use pomodoro::domain::schedule::Schedule;

#[test]
fn schedule_length_for_2_cycles() {
    let s = Schedule::from_minutes_for_test(1, 1, 1, 2);
    // 2 focuses + 1 short + 1 long = 4 segments
    assert_eq!(s.segments.len(), 4);
}
