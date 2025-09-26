use anyhow::Result;
use tracing::debug;

pub fn beep() {
    // simple ASCII BEL
    print!("\x07");
    let _ = std::io::Write::flush(&mut std::io::stdout());
}

pub fn notify(title: &str, body: &Option<String>) -> Result<()> {
    #[cfg(feature = "notify")]
    {
        use notify_rust::Notification;
        let b = body.clone().unwrap_or_default();
        Notification::new()
            .summary(title)
            .body(&b)
            .show()?;
        Ok(())
    }
    #[cfg(not(feature = "notify"))]
    {
        // fallback: just write a message to terminal
        debug!("notif: {} - {:?}", title, body);
        Ok(())
    }
}
