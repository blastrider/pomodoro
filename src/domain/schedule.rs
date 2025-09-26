use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SegmentKind {
    Focus,
    ShortBreak,
    LongBreak,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Segment {
    pub kind: SegmentKind,
    /// duration in seconds
    pub seconds: u64,
    pub cycle_index: u8,
}

#[derive(Debug, Clone)]
pub struct Schedule {
    pub segments: Vec<Segment>,
}

impl Schedule {
    /// Build a schedule from a Config (durations in minutes).
    pub fn from_config(cfg: &crate::domain::config::Config) -> Self {
        let mut segs = Vec::new();
        for i in 1..=cfg.cycles {
            segs.push(Segment {
                kind: SegmentKind::Focus,
                seconds: cfg.focus_min * 60,
                cycle_index: i,
            });
            if i == cfg.cycles {
                // add long break after last focus
                segs.push(Segment {
                    kind: SegmentKind::LongBreak,
                    seconds: cfg.long_min * 60,
                    cycle_index: i,
                });
            } else {
                segs.push(Segment {
                    kind: SegmentKind::ShortBreak,
                    seconds: cfg.short_min * 60,
                    cycle_index: i,
                });
            }
        }
        Schedule { segments: segs }
    }

    /// Public helper: build a schedule using durations given in **seconds**.
    /// Useful for tests/integration where you want short, fast-running segments.
    pub fn from_seconds_for_test(focus_s: u64, short_s: u64, long_s: u64, cycles: u8) -> Self {
        let mut segs = Vec::new();
        for i in 1..=cycles {
            segs.push(Segment {
                kind: SegmentKind::Focus,
                seconds: focus_s,
                cycle_index: i,
            });
            if i == cycles {
                segs.push(Segment {
                    kind: SegmentKind::LongBreak,
                    seconds: long_s,
                    cycle_index: i,
                });
            } else {
                segs.push(Segment {
                    kind: SegmentKind::ShortBreak,
                    seconds: short_s,
                    cycle_index: i,
                });
            }
        }
        Schedule { segments: segs }
    }
}
