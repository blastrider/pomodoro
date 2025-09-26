use crate::domain::schedule::SegmentKind;
use crate::infra::storage::{Journal, SessionEntry, SessionState};
use crate::ui::terminal::Terminal;
use anyhow::{Context, Result};
use std::sync::{Arc, Mutex};
use time::OffsetDateTime;
use tracing::{error, info};

pub struct SessionRunner {
    cfg: crate::domain::config::Config,
    journal: Journal,
    beep: bool,
    notify: bool,
    state: Arc<Mutex<Option<SessionEntry>>>,
}

impl SessionRunner {
    pub fn new(
        cfg: crate::domain::config::Config,
        journal: Journal,
        beep: bool,
        notify: bool,
    ) -> Self {
        Self {
            cfg,
            journal,
            beep,
            notify,
            state: Arc::new(Mutex::new(None)),
        }
    }

    pub fn install_ctrlc_handler(&mut self) -> Result<()> {
        let st = self.state.clone();
        let j = self.journal.path.clone();
        ctrlc::set_handler(move || {
            if let Ok(mut guard) = st.lock() {
                if let Some(entry) = guard.as_mut() {
                    entry.state = SessionState::Interrupted;
                    entry.end = Some(OffsetDateTime::now_utc());
                    let _ = entry.save_to_path(&j);
                    info!("Saved interrupted session to journal");
                }
            }
            std::process::exit(130);
        })
        .context("installing ctrlc handler")?;
        Ok(())
    }

    pub async fn run(&mut self) -> Result<SessionEntry> {
        let schedule = self.cfg.clone().into_schedule();
        let mut terminal = Terminal::new(self.cfg.task.clone())?;
        let entry = SessionEntry::new(&self.cfg)?;

        // keep state for ctrlc
        {
            let mut guard = self.state.lock().unwrap();
            *guard = Some(entry.clone());
        }

        for seg in schedule.segments {
            let kind_label = match seg.kind {
                SegmentKind::Focus => "FOCUS",
                SegmentKind::ShortBreak => "BREAK",
                SegmentKind::LongBreak => "LONG BREAK",
            };
            info!("Starting segment: {} ({}s)", kind_label, seg.seconds);
            terminal
                .show_segment(kind_label.to_string(), seg.seconds)
                .await?;

            if self.beep {
                crate::infra::notify::beep();
            }
            if self.notify {
                let _ = crate::infra::notify::notify(kind_label, &self.cfg.task);
            }

            // update journal partial after each segment
            let mut guard = self.state.lock().unwrap();
            if let Some(e) = guard.as_mut() {
                e.segments.push(format!("{}:{}s", kind_label, seg.seconds));
                e.last_updated = OffsetDateTime::now_utc();
                if let Err(err) = e.append_to_path(&self.journal.path) {
                    error!("Failed to append session partial to journal: {:?}", err);
                }
            }
        }

        // finish entry
        let mut guard = self.state.lock().unwrap();
        if let Some(mut e) = guard.take() {
            e.end = Some(OffsetDateTime::now_utc());
            e.state = SessionState::Completed;
            self.journal.append(&e)?;
            info!("Session saved to journal");
            Ok(e)
        } else {
            Err(anyhow::anyhow!("Session state gone"))
        }
    }

    pub fn export_markdown(&self) -> Result<()> {
        self.journal.export_markdown_today()
    }

    pub fn export_csv(&self) -> Result<()> {
        self.journal.export_csv_today()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::schedule::Schedule;

    #[tokio::test]
    async fn test_short_schedule_run() {
        let cfg = crate::domain::config::Config {
            focus_min: 0,
            short_min: 0,
            long_min: 0,
            cycles: 2,
            task: Some("test".into()),
        };
        let schedule = Schedule::from_minutes_for_test(1, 1, 1, 2);
        assert_eq!(schedule.segments.len(), 4);
    }
}
