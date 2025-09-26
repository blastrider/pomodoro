use pomodoro_cli::domain::config::Config;

#[test]
fn config_accepts_defaults() {
    let c = Config::default();
    assert!(c.validate().is_ok());
}

#[test]
fn config_rejects_too_short_focus() {
    let mut c = Config::default();
    c.focus_min = 1;
    assert!(c.validate().is_err());
}
