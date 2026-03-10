use dioxus::prelude::*;
use uuid::Uuid;
use crate::process::{RunningProcess, ProcessStatus};

#[derive(Props, Clone, PartialEq)]
pub struct LogPanelProps {
    pub processes:  Signal<Vec<RunningProcess>>,
    pub active_tab: Signal<Option<Uuid>>,
}

#[component]
pub fn LogPanel(props: LogPanelProps) -> Element {
    let processes  = props.processes;
    let mut active = props.active_tab;

    // Only show panel if there are tracked processes
    let procs = processes.read();
    if procs.is_empty() {
        return rsx! { div {} };
    }

    // Fallback: if active tab was removed, pick first
    let active_id = {
        let current = *active.read();
        match current {
            Some(id) if procs.iter().any(|p| p.instance_id == id) => id,
            _ => {
                let first = procs[0].instance_id;
                drop(procs);
                active.set(Some(first));
                first
            }
        }
    };

    let procs = processes.read();
    let current = procs.iter().find(|p| p.instance_id == active_id);

    rsx! {
        div { class: "log-panel",

            // ── Tab bar ───────────────────────────────────────────────────────
            div { class: "log-tabs",
                for proc in procs.iter() {
                    LogTab {
                        key:        "{proc.instance_id}",
                        process:    proc.clone(),
                        is_active:  proc.instance_id == active_id,
                        active_tab: active,
                        processes,
                    }
                }
                // Collapse / hide panel button
                div { class: "log-tab-spacer" }
                button {
                    class:   "log-collapse-btn",
                    onclick: move |_| active.set(None),
                    "▼  Hide"
                }
            }

            // ── Log output ────────────────────────────────────────────────────
            if let Some(proc) = current {
                LogOutput { process: proc.clone() }
            }
        }
    }
}

// ── LogTab ────────────────────────────────────────────────────────────────────

#[derive(Props, Clone, PartialEq)]
struct LogTabProps {
    process:    RunningProcess,
    is_active:  bool,
    active_tab: Signal<Option<Uuid>>,
    processes:  Signal<Vec<RunningProcess>>,
}

#[component]
fn LogTab(props: LogTabProps) -> Element {
    let mut active    = props.active_tab;
    let mut processes = props.processes;
    let instance_id        = props.process.instance_id;
    let is_running    = props.process.is_running();

    let status_class = if is_running { "log-tab-dot running" } else { "log-tab-dot stopped" };
    let tab_class    = if props.is_active { "log-tab active" } else { "log-tab" };

    rsx! {
        div { class: "{tab_class}",
            onclick: move |_| active.set(Some(instance_id)),

            // Status dot
            span { class: "{status_class}" }

            // App name
            span { class: "log-tab-name", "{props.process.name}" }

            // Kill / close button
            if is_running {
                button {
                    class: "log-tab-kill",
                    title: "Kill process",
                    onclick: move |e| {
                        e.stop_propagation();
                        props.process.kill();
                    },
                    "■"
                }
            } else {
                button {
                    class: "log-tab-kill",
                    title: "Remove",
                    onclick: move |e| {
                        e.stop_propagation();
                        processes.write().retain(|p| p.instance_id != instance_id);
                        // If we removed the active tab, clear selection
                        if *active.read() == Some(instance_id) {
                            active.set(None);
                        }
                    },
                    "×"
                }
            }
        }
    }
}

// ── LogOutput ─────────────────────────────────────────────────────────────────

#[derive(Props, Clone, PartialEq)]
struct LogOutputProps {
    process: RunningProcess,
}

#[component]
fn LogOutput(props: LogOutputProps) -> Element {
    // Auto-scroll to bottom when logs update
    use_effect(move || {
        let _ = document::eval(r#"
            const el = document.getElementById('log-output');
            if (el) el.scrollTop = el.scrollHeight;
        "#);
    });

    let exit_line = match &props.process.status {
        ProcessStatus::Exited(code) => Some(format!("[exited with code {code}]")),
        ProcessStatus::Killed       => Some("[killed]".into()),
        ProcessStatus::Running      => None,
    };

    rsx! {
        div { class: "log-output", id: "log-output",
            for (i, line) in props.process.logs.iter().enumerate() {
                div {
                    key:   "{props.process.app_id}-{i}",
                    class: if line.starts_with("[err]") { "log-line err" } else { "log-line" },
                    "{line}"
                }
            }
            if let Some(line) = exit_line {
                div { class: "log-line exit", "{line}" }
            }
            if props.process.logs.is_empty() {
                div { class: "log-line muted", "Waiting for output..." }
            }
        }
    }
}