use anyhow::Context;
use anyhow::Result;
use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;
use tokio::time::sleep;

pub struct Terminal {
    task: Option<String>,
}

impl Terminal {
    pub fn new(task: Option<String>) -> Result<Self> {
        Ok(Self { task })
    }

    pub async fn show_segment(&mut self, label: String, seconds: u64) -> Result<()> {
        let style = ProgressStyle::with_template("{prefix} {bar:40.cyan/blue} {pos}/{len}s {elapsed}")
            .context("invalid progress style template")?;
        let pb = ProgressBar::new(seconds);
        pb.set_style(style);
        pb.set_prefix(format!("[{}] {}", label, self.task.clone().unwrap_or_default()));

        let mut remaining = seconds;
        while remaining > 0 {
            pb.set_position(seconds - remaining);
            sleep(Duration::from_secs(1)).await;
            remaining = remaining.saturating_sub(1);
        }
        pb.finish_with_message("done");
        Ok(())
    }
}
