use crate::{CliArgs, Config, Journal};
use dioxus::prelude::*;
use std::time::Duration;
use tokio::time::sleep;

#[derive(Clone, Debug, PartialEq)]
enum AppState {
    Configuring,
    Running,
    Finished,
}

pub fn run_gui(cli_args: CliArgs) {
    // Initial configuration based on CLI arguments
    let initial_config = Config::from_cli_and_preset(&cli_args).unwrap_or_default();

    // Launch Dioxus desktop app
    dioxus::desktop::launch::launch_virtual_dom(
        dioxus::prelude::VirtualDom::new_with_props(App, AppProps { initial_config }),
        dioxus::desktop::Config::new()
            .with_window(dioxus::desktop::WindowBuilder::new().with_title("Pomodoro")),
    );
}

#[derive(Props, Clone, PartialEq)]
pub struct AppProps {
    initial_config: Config,
}

use futures_util::stream::StreamExt;

#[component]
fn App(props: AppProps) -> Element {
    let mut state = use_signal(|| AppState::Configuring);
    let mut config = use_signal(|| props.initial_config.clone());

    let current_segment_label = use_signal(String::new);
    let current_remaining_seconds = use_signal(|| 0u64);

    let coroutine = use_coroutine(|mut rx: UnboundedReceiver<Config>| {
        let mut state = state;
        let mut current_segment_label = current_segment_label;
        let mut current_remaining_seconds = current_remaining_seconds;

        async move {
            while let Some(cfg) = rx.next().await {
                state.set(AppState::Running);

                let schedule = cfg.clone().into_schedule();

                // Open Journal and create SessionEntry
                let journal_res = Journal::open_default();
                if let Ok(journal) = journal_res {
                    if let Ok(mut entry) = crate::infra::storage::SessionEntry::new(&cfg) {
                        for seg in schedule.segments {
                            let kind_label = match seg.kind {
                                crate::domain::schedule::SegmentKind::Focus => "FOCUS",
                                crate::domain::schedule::SegmentKind::ShortBreak => "BREAK",
                                crate::domain::schedule::SegmentKind::LongBreak => "LONG BREAK",
                            };

                            current_segment_label.set(kind_label.to_string());
                            let mut remaining = seg.seconds;

                            while remaining > 0 {
                                current_remaining_seconds.set(remaining);
                                sleep(Duration::from_secs(1)).await;
                                remaining = remaining.saturating_sub(1);
                            }

                            // update journal partial after each segment
                            entry
                                .segments
                                .push(format!("{}:{}s", kind_label, seg.seconds));
                            entry.last_updated = time::OffsetDateTime::now_utc();
                            let _ = entry.append_to_path(&journal.path);
                        }

                        entry.end = Some(time::OffsetDateTime::now_utc());
                        entry.state = crate::infra::storage::SessionState::Completed;
                        let _ = journal.append(&entry);
                    }
                }

                state.set(AppState::Finished);
            }
        }
    });

    let state_val = state.read().clone();
    match state_val {
        AppState::Configuring => {
            rsx! {
                div {
                    style: "padding: 20px; font-family: sans-serif;",
                    h1 { "Pomodoro Configuration" }

                    div { margin_bottom: "10px",
                        label { "Focus (min): " }
                        input {
                            "type": "number",
                            value: "{config.read().focus_min}",
                            oninput: move |evt| {
                                if let Ok(val) = evt.value().parse::<u64>() {
                                    config.write().focus_min = val;
                                }
                            }
                        }
                    }
                    div { margin_bottom: "10px",
                        label { "Short Break (min): " }
                        input {
                            "type": "number",
                            value: "{config.read().short_min}",
                            oninput: move |evt| {
                                if let Ok(val) = evt.value().parse::<u64>() {
                                    config.write().short_min = val;
                                }
                            }
                        }
                    }
                    div { margin_bottom: "10px",
                        label { "Long Break (min): " }
                        input {
                            "type": "number",
                            value: "{config.read().long_min}",
                            oninput: move |evt| {
                                if let Ok(val) = evt.value().parse::<u64>() {
                                    config.write().long_min = val;
                                }
                            }
                        }
                    }
                    div { margin_bottom: "10px",
                        label { "Cycles: " }
                        input {
                            "type": "number",
                            value: "{config.read().cycles}",
                            oninput: move |evt| {
                                if let Ok(val) = evt.value().parse::<u8>() {
                                    config.write().cycles = val;
                                }
                            }
                        }
                    }
                    div { margin_bottom: "10px",
                        label { "Task: " }
                        input {
                            "type": "text",
                            value: "{config.read().task.clone().unwrap_or_default()}",
                            oninput: move |evt| {
                                config.write().task = if evt.value().is_empty() { None } else { Some(evt.value().clone()) };
                            }
                        }
                    }

                    button {
                        style: "padding: 10px 20px; font-size: 16px;",
                        onclick: move |_| {
                            coroutine.send(config.read().clone());
                        },
                        "Start Session"
                    }
                }
            }
        }
        AppState::Running => {
            let label = current_segment_label.read().clone();
            let remaining = *current_remaining_seconds.read();
            let mins = remaining / 60;
            let secs = remaining % 60;
            let task_name = config.read().task.clone().unwrap_or_default();

            rsx! {
                div {
                    style: "padding: 50px; text-align: center; font-family: sans-serif;",
                    h2 { "{label}" }
                    if !task_name.is_empty() {
                        h3 { "{task_name}" }
                    }
                    div {
                        style: "font-size: 80px; font-weight: bold; margin: 20px 0;",
                        "{mins:02}:{secs:02}"
                    }
                }
            }
        }
        AppState::Finished => {
            rsx! {
                div {
                    style: "padding: 50px; text-align: center; font-family: sans-serif;",
                    h1 { "Session Finished!" }
                    button {
                        style: "padding: 10px 20px; font-size: 16px;",
                        onclick: move |_| {
                            state.set(AppState::Configuring);
                        },
                        "New Session"
                    }
                }
            }
        }
    }
}
