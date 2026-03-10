use dioxus::prelude::*;
use uuid::Uuid;
use crate::components::log_panel::LogPanel;
use crate::components::main_content::MainContent;
use crate::components::modal::{AddAppModal, AddProjectModal};
use crate::components::sidenav::SideNav;
use crate::components::toast::{push_toast, Toast, ToastList};
use crate::process::{ProcessEvent, ProcessStatus, RunningProcess};

mod components;
mod config;
mod process;

const MAIN_CSS: Asset = asset!("/assets/styling/main.css");

fn main() {
    tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(async {
            dioxus::launch(App);
        });
}

#[component]
fn App() -> Element {
    let collapsed  = use_signal(|| false);
    let selected   = use_signal(|| 0usize);
    let config = config::Config::load("config.json")?;
    let config     = use_signal(|| config);
    let processes: Signal<Vec<RunningProcess>> = use_signal(Vec::new);
    let active_tab: Signal<Option<Uuid>>       = use_signal(|| None);
    let toasts: Signal<Vec<Toast>>             = use_signal(Vec::new);
    let mut show_add_project: Signal<bool>         = use_signal(|| false);
    let mut show_add_app: Signal<bool>             = use_signal(|| false);

    let process_tx: Coroutine<ProcessEvent> = use_coroutine(move |mut rx: UnboundedReceiver<ProcessEvent>| {
        let mut processes = processes;
        let mut toasts    = toasts;

        async move {
            while let Ok(event) = rx.recv().await {
                match event {
                    ProcessEvent::Line { instance_id, text } => {
                        let mut procs = processes.write();
                        if let Some(p) = procs.iter_mut().find(|p| p.instance_id == instance_id) {
                            p.logs.push(text);
                            if p.logs.len() > 1000 {
                                p.logs.drain(0..100);
                            }
                        }
                    }
                    ProcessEvent::Exited { instance_id, code } => {
                        let mut procs = processes.write();
                        if let Some(p) = procs.iter_mut().find(|p| p.instance_id == instance_id) {
                            p.status = if code == -1 {
                                ProcessStatus::Killed
                            } else {
                                ProcessStatus::Exited(code)
                            };
                            let msg = if code == 0 {
                                format!("'{}' exited cleanly", p.name)
                            } else if code == -1 {
                                format!("'{}' was killed", p.name)
                            } else {
                                format!("'{}' exited with code {code}", p.name)
                            };
                            push_toast(toasts, if code == 0 {
                                Toast::success(msg)
                            } else {
                                Toast::error(msg)
                            });

                            drop(procs);
                            processes.write();
                        }
                    }
                    ProcessEvent::Failed { app_id: _, reason } => {
                        push_toast(toasts, Toast::error(format!("Launch failed: {reason}")));
                    }
                }
            }
        }
    });

    let on_launch = move |app: config::App| {
        let tx        = process_tx.tx();  // get the UnboundedSender from the coroutine
        let mut procs = processes;
        let mut tabs  = active_tab;

        spawn(async move {
            match process::spawn(&app, tx).await {
                Some(proc) => {
                    let app_id = proc.app_id;
                    procs.write().push(proc);
                    tabs.set(Some(app_id));
                }
                None => {}
            }
        });
    };

    // ── Derive current project ────────────────────────────────────────────────

    let project = config.read().projects
        .get(*selected.read())
        .cloned();

    let project_id = project.as_ref().map(|p| p.id);

    // ── Render ────────────────────────────────────────────────────────────────

    rsx! {
        document::Stylesheet { href: MAIN_CSS }

        div { class: "app-layout",

            // ── Modals ────────────────────────────────────────────────────────
            if *show_add_project.read() {
                AddProjectModal {
                    config,
                    on_close: move |_| *show_add_project.write() = false,
                    on_saved: move |name| push_toast(toasts, Toast::success(
                        format!("Project '{name}' added")
                    )),
                }
            }

            if *show_add_app.read() {
                if let Some(pid) = project_id {
                    AddAppModal {
                        config,
                        project_id:  pid,
                        on_close:    move |_| *show_add_app.write() = false,
                        on_saved:    move |name| push_toast(toasts, Toast::success(
                            format!("App '{name}' added")
                        )),
                    }
                }
            }

            // ── Layout ────────────────────────────────────────────────────────
            div { class: "app-body",
                SideNav {
                    projects:         config.read().projects.clone(),
                    selected,
                    collapsed,
                    on_add: show_add_project,
                }

                if let Some(project) = project {
                    MainContent {
                        project,
                        add_app: show_add_app,
                        on_launch,
                    }
                } else {
                    div { class: "content-empty",
                        "No projects yet — add one to get started."
                    }
                }
            }

            if !processes.read().is_empty() && active_tab.read().is_some() {
                LogPanel { processes, active_tab }
            }

            ToastList { toasts }
        }
    }
}