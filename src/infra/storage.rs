use anyhow::{Context, Result};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use time::OffsetDateTime;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SessionState {
    Ongoing,
    Completed,
    Interrupted,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionEntry {
    #[serde(with = "time::serde::rfc3339")]
    pub start: OffsetDateTime,

    #[serde(with = "time::serde::rfc3339::option")]
    pub end: Option<OffsetDateTime>,

    pub cfg: crate::domain::config::Config,
    pub state: SessionState,
    pub segments: Vec<String>,

    #[serde(with = "time::serde::rfc3339")]
    pub last_updated: OffsetDateTime,
}

impl SessionEntry {
    pub fn new(cfg: &crate::domain::config::Config) -> anyhow::Result<Self> {
        Ok(Self {
            start: OffsetDateTime::now_utc(),
            end: None,
            cfg: cfg.clone(),
            state: SessionState::Ongoing,
            segments: vec![],
            last_updated: OffsetDateTime::now_utc(),
        })
    }

    pub fn append_to_path(&self, path: &PathBuf) -> Result<()> {
        let mut f = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .with_context(|| format!("open journal {}", path.display()))?;
        let s = serde_json::to_string(self)?;
        writeln!(f, "{}", s)?;
        Ok(())
    }

    pub fn save_to_path(&self, path: &PathBuf) -> Result<()> {
        let s = serde_json::to_string_pretty(self)?;
        fs::write(path, s)?;
        Ok(())
    }
}

pub struct Journal {
    pub dir: PathBuf,
    pub path: PathBuf,
}

impl Journal {
    pub fn open_default() -> Result<Self> {
        let pd = ProjectDirs::from("com", "you", "pomodoro")
            .context("finding project dirs")?;
        let data_dir = pd.data_dir();
        fs::create_dir_all(data_dir).context("creating data dir")?;
        let today = OffsetDateTime::now_utc().date();
        let file_name = format!("journal-{}.jsonl", today);
        let path = data_dir.join(file_name);
        Ok(Journal {
            dir: data_dir.to_path_buf(),
            path,
        })
    }

    pub fn append(&self, entry: &SessionEntry) -> Result<()> {
        entry.append_to_path(&self.path)?;
        Ok(())
    }

    pub fn export_markdown_today(&self) -> Result<()> {
        let content = fs::read_to_string(&self.path).with_context(|| "reading journal file")?;
        let mut md = String::new();
        md.push_str("# Pomodoro journal (today)\n\n");
        for line in content.lines() {
            let e: SessionEntry = serde_json::from_str(line)?;
            md.push_str(&format!(
                "- **start**: {}\n  - task: {:?}\n  - state: {:?}\n  - segments: {:?}\n\n",
                e.start, e.cfg.task, e.state, e.segments
            ));
        }
        let out = self.dir.join("journal-today.md");
        fs::write(&out, md)?;
        Ok(())
    }

    pub fn export_csv_today(&self) -> Result<()> {
        let content = fs::read_to_string(&self.path).with_context(|| "reading journal file")?;
        let mut csv = String::from("start,end,state,task,segments\n");
        for line in content.lines() {
            let e: SessionEntry = serde_json::from_str(line)?;
            let end = e.end.map(|d| d.to_string()).unwrap_or_default();
            let task = e.cfg.task.clone().unwrap_or_default().replace(',', " ");
            let segments = e.segments.join(" | ");
            csv.push_str(&format!("{},{},{},{},{}\n", e.start, end, format!("{:?}", e.state), task, segments));
        }
        let out = self.dir.join("journal-today.csv");
        fs::write(&out, csv)?;
        Ok(())
    }
}
