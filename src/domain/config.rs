use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub focus_min: u64,
    pub short_min: u64,
    pub long_min: u64,
    pub cycles: u8,
    pub task: Option<String>,
}

impl Config {
    pub fn validate(&self) -> Result<()> {
        if !(5..=120).contains(&self.focus_min) {
            return Err(anyhow!("focus must be between 5 and 120 minutes"));
        }
        if !(1..=30).contains(&self.short_min) {
            return Err(anyhow!("short break must be between 1 and 30 minutes"));
        }
        if !(5..=60).contains(&self.long_min) {
            return Err(anyhow!("long break must be between 5 and 60 minutes"));
        }
        if !(1..=12).contains(&self.cycles) {
            return Err(anyhow!("cycles must be between 1 and 12"));
        }
        if let Some(t) = &self.task {
            if t.chars().count() > 80 {
                return Err(anyhow!("task label must be <= 80 characters"));
            }
        }
        Ok(())
    }

    pub fn default() -> Self {
        Self {
            focus_min: 25,
            short_min: 5,
            long_min: 15,
            cycles: 4,
            task: None,
        }
    }

    pub fn from_preset_file(path: &Path) -> Result<Self> {
        let s = fs::read_to_string(path)
            .with_context(|| format!("reading preset file {}", path.display()))?;
        if path.extension().and_then(|s| s.to_str()) == Some("json") {
            let cfg: Config = serde_json::from_str(&s).context("parsing json preset")?;
            cfg.validate()?;
            Ok(cfg)
        } else {
            // try yaml if feature enabled, otherwise error
            #[cfg(feature = "serde_yaml")]
            {
                let cfg: Config =
                    serde_yaml::from_str(&s).context("parsing yaml preset (serde_yaml)")?;
                cfg.validate()?;
                Ok(cfg)
            }
            #[cfg(not(feature = "serde_yaml"))]
            {
                Err(anyhow!("YAML presets require building with the `serde_yaml` feature"))
            }
        }
    }

    pub fn from_cli_and_preset(cli: &crate::Cli) -> Result<Self> {
        let mut base = if let Some(p) = &cli.preset {
            Self::from_preset_file(p)?
        } else {
            Self::default()
        };

        if let Some(f) = cli.focus {
            base.focus_min = f;
        }
        if let Some(s) = cli.short {
            base.short_min = s;
        }
        if let Some(l) = cli.long {
            base.long_min = l;
        }
        if let Some(c) = cli.cycles {
            base.cycles = c;
        }
        if let Some(t) = &cli.task {
            base.task = Some(t.clone());
        }
        base.validate()?;
        Ok(base)
    }

    pub fn into_schedule(self) -> crate::domain::schedule::Schedule {
        crate::domain::schedule::Schedule::from_config(&self)
    }
}