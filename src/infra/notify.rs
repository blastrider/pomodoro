use anyhow::Result;
use std::io::Write;
use std::process::Command;
use tracing::debug;

/// Try several ways to produce an audible beep:
/// 1) ASCII BEL to the terminal (may be ignored if terminal bell is disabled)
/// 2) canberra-gtk-play (common on desktops)
/// 3) paplay / aplay / play (pulseaudio/alsa/sox)
/// 4) afplay on macOS
/// 5) powershell beep on Windows
///
/// All attempts are best-effort and errors are ignored (only debug logged).
pub fn beep() {
    // 1) Terminal BEL (avoid `let _ =` on unit)
    print!("\x07");
    let _ = std::io::stdout().flush();

    // 2) Desktop sound helpers (try several common players)
    // Linux common players: try them in one combined condition per clippy
    if cfg!(target_os = "linux") {
        if Command::new("canberra-gtk-play")
            .arg("-i")
            .arg("bell")
            .spawn()
            .is_ok()
        {
            debug!("beep: played with canberra-gtk-play");
            return;
        }

        if Command::new("paplay")
            .arg("/usr/share/sounds/freedesktop/stereo/complete.oga")
            .spawn()
            .is_ok()
        {
            debug!("beep: played with paplay");
            return;
        }

        if Command::new("aplay")
            .arg("/usr/share/sounds/alsa/Front_Center.wav")
            .spawn()
            .is_ok()
        {
            debug!("beep: played with aplay");
            return;
        }

        if Command::new("play")
            .arg("/usr/share/sounds/freedesktop/stereo/complete.oga")
            .spawn()
            .is_ok()
        {
            debug!("beep: played with play (sox)");
            return;
        }
    }

    // macOS: collapse condition + command call as suggested by clippy
    if cfg!(target_os = "macos")
        && Command::new("afplay")
            .arg("/System/Library/Sounds/Glass.aiff")
            .spawn()
            .is_ok()
    {
        debug!("beep: played with afplay");
        return;
    }

    // Windows: collapsed similarly
    if cfg!(target_os = "windows")
        && Command::new("powershell")
            .arg("-c")
            .arg("[console]::beep(1000,200)")
            .spawn()
            .is_ok()
    {
        debug!("beep: played with powershell");
        return;
    }

    debug!("beep: no method succeeded (terminal bell may be disabled or no player available)");
}

pub fn notify(title: &str, body: &Option<String>) -> Result<()> {
    #[cfg(feature = "notify")]
    {
        use notify_rust::Notification;
        let b = body.clone().unwrap_or_default();
        Notification::new().summary(title).body(&b).show()?;
        Ok(())
    }
    #[cfg(not(feature = "notify"))]
    {
        // fallback: just write a message to terminal
        debug!("notif: {} - {:?}", title, body);
        Ok(())
    }
}
