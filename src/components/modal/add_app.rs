use dioxus::prelude::*;
use crate::config::{App, Config};
use crate::components::modal::{
    Field, Modal, ModalActions, PathInput, TextArea, TextInput,
};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Clone, PartialEq, Default)]
struct AppForm {
    name:        String,
    description: String,
    icon:        String,
    command:     String,
    working_dir: String,
    args:        String, // space-separated, split on confirm
    env_vars:    Vec<EnvRow>,
    confirm:     bool,
}

#[derive(Clone, PartialEq, Default)]
struct EnvRow {
    id:    Uuid,
    key:   String,
    value: String,
}

impl EnvRow {
    fn new() -> Self {
        Self { id: Uuid::new_v4(), key: String::new(), value: String::new() }
    }
}

impl AppForm {
    fn is_valid(&self) -> bool {
        !self.name.trim().is_empty() && !self.command.trim().is_empty()
    }

    fn into_app(self) -> App {
        let env: HashMap<String, String> = self.env_vars
            .into_iter()
            .filter(|r| !r.key.trim().is_empty())
            .map(|r| (r.key.trim().to_string(), r.value.trim().to_string()))
            .collect();

        let args: Vec<String> = self.args
            .split_whitespace()
            .map(|s| s.to_string())
            .collect();

        App {
            id:          Uuid::new_v4(),
            name:        self.name.trim().to_string(),
            description: self.description.trim().to_string(),
            icon:        if self.icon.is_empty() { None } else { Some(self.icon) },
            icon_data:   None,
            command:     self.command.trim().to_string(),
            cwd: if self.working_dir.is_empty() { None } else { Some(self.working_dir) },
            args,
            env,
            confirm:     self.confirm,
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct AddAppModalProps {
    pub config:      Signal<Config>,
    pub project_id:  Uuid,
    pub on_close:    EventHandler<()>,
    pub on_saved:    EventHandler<String>,
}

#[component]
pub fn AddAppModal(props: AddAppModalProps) -> Element {
    let mut form   = use_signal(AppForm::default);
    let mut config = props.config;
    let on_close   = props.on_close.clone();

    let on_browse_command = move |_| {
        let mut form = form.clone();
        spawn(async move {
            let file = rfd::AsyncFileDialog::new()
                .set_title("Pick executable")
                .pick_file()
                .await;
            if let Some(f) = file {
                form.write().command = f.path().to_string_lossy().into_owned();
            }
        });
    };

    let on_browse_workdir = move |_| {
        let mut form = form.clone();
        spawn(async move {
            let folder = rfd::AsyncFileDialog::new()
                .set_title("Pick working directory")
                .pick_folder()
                .await;
            if let Some(f) = folder {
                form.write().working_dir = f.path().to_string_lossy().into_owned();
            }
        });
    };

    let on_browse_icon = move |_| {
        let mut form = form.clone();
        spawn(async move {
            let file = rfd::AsyncFileDialog::new()
                .set_title("Pick app icon")
                .add_filter("Image", &["png", "svg", "jpg", "webp"])
                .pick_file()
                .await;
            if let Some(f) = file {
                form.write().icon = f.path().to_string_lossy().into_owned();
            }
        });
    };

    let on_confirm = move |_| {
        let f = form.read().clone();
        if !f.is_valid() { return; }

        let app_name   = f.name.clone();
        let app        = f.into_app();
        let project_id = props.project_id;

        let mut cfg = config.write();
        if let Some(project) = cfg.projects.iter_mut().find(|p| p.id == project_id) {
            project.apps.push(app);
        }
        drop(cfg);

        if let Err(e) = config.read().save("config.json") {
            eprintln!("[launcher] Failed to save config: {e}");
        }

        props.on_saved.call(app_name);
        props.on_close.call(());
    };

    // ── Render ────────────────────────────────────────────────────────────────

    let form_read = form.read();

    rsx! {
        Modal {
            title:    "Add App",
            on_close: props.on_close,
            width:    "560px".to_string(),

            // ── Name ──────────────────────────────────────────────────────────
            Field { label: "Name", hint: "Required",
                TextInput {
                    value:       form_read.name.clone(),
                    placeholder: "e.g. Dev Server",
                    on_change:   move |v| form.write().name = v,
                }
            }

            // ── Description ───────────────────────────────────────────────────
            Field { label: "Description",
                TextArea {
                    value:       form_read.description.clone(),
                    placeholder: "What does this app do?",
                    on_change:   move |v| form.write().description = v,
                    rows:        2,
                }
            }

            // ── Icon ──────────────────────────────────────────────────────────
            Field { label: "Icon", hint: "Emoji (e.g. 🖥️) or image file",
                div { class: "icon-row",
                    TextInput {
                        value:       form_read.icon.clone(),
                        placeholder: "🖥️  or  /path/to/icon.png",
                        on_change:   move |v| form.write().icon = v,
                    }
                    button {
                        class:   "btn-browse",
                        onclick: on_browse_icon,
                        "Browse…"
                    }
                }
            }

            // ── Command ───────────────────────────────────────────────────────
            Field { label: "Command", hint: "Required — executable to run",
                PathInput {
                    value:       form_read.command.clone(),
                    placeholder: "e.g. cargo  or  /usr/bin/python3",
                    on_change:   move |v| form.write().command = v,
                    on_browse:   on_browse_command,
                    btn_label:   "Browse…".to_string(),
                }
            }

            // ── Working directory ─────────────────────────────────────────────
            Field { label: "Working Directory", hint: "Optional — defaults to current dir",
                PathInput {
                    value:       form_read.working_dir.clone(),
                    placeholder: "e.g. /home/user/project",
                    on_change:   move |v| form.write().working_dir = v,
                    on_browse:   on_browse_workdir,
                    btn_label:   "Browse…".to_string(),
                }
            }

            // ── Arguments ─────────────────────────────────────────────────────
            Field { label: "Arguments", hint: "Space-separated",
                TextInput {
                    value:       form_read.args.clone(),
                    placeholder: "e.g. run --release --bin server",
                    on_change:   move |v| form.write().args = v,
                    monospace:   true,
                }
            }

            // ── Environment variables ─────────────────────────────────────────
            Field { label: "Environment Variables",
                div { class: "env-list",
                    for (i, row) in form_read.env_vars.iter().enumerate() {
                        EnvRowInput {
                            key:      "{row.id}",
                            row:      row.clone(),
                            on_key:   move |v| form.write().env_vars[i].key   = v,
                            on_value: move |v| form.write().env_vars[i].value = v,
                            on_remove: move |_| { form.write().env_vars.remove(i); },
                        }
                    }
                    button {
                        class:   "btn-add-env",
                        onclick: move |_| form.write().env_vars.push(EnvRow::new()),
                        "+ Add variable"
                    }
                }
            }

            // ── Confirm toggle ────────────────────────────────────────────────
            div { class: "checkbox-row",
                input {
                    r#type:   "checkbox",
                    id:       "confirm-toggle",
                    class:    "checkbox",
                    checked:  form_read.confirm,
                    onchange: move |e| form.write().confirm = e.checked(),
                }
                label {
                    r#for: "confirm-toggle",
                    class: "checkbox-label",
                    "Ask for confirmation before launching"
                }
            }

            // ── Actions ───────────────────────────────────────────────────────
            ModalActions {
                on_cancel:        on_close,
                on_confirm,
                confirm_label:    "Add App",
                confirm_disabled: !form_read.is_valid(),
            }
        }
    }
}

// ── EnvRowInput ───────────────────────────────────────────────────────────────

#[derive(Props, Clone, PartialEq)]
struct EnvRowInputProps {
    row:      EnvRow,
    on_key:   EventHandler<String>,
    on_value: EventHandler<String>,
    on_remove: EventHandler<()>,
}

#[component]
fn EnvRowInput(props: EnvRowInputProps) -> Element {
    rsx! {
        div { class: "env-row",
            input {
                class:       "form-input env-key",
                r#type:      "text",
                placeholder: "KEY",
                value:       "{props.row.key}",
                oninput:     move |e| props.on_key.call(e.value()),
            }
            span { class: "env-eq", "=" }
            input {
                class:       "form-input env-val",
                r#type:      "text",
                placeholder: "value",
                value:       "{props.row.value}",
                oninput:     move |e| props.on_value.call(e.value()),
            }
            button {
                class:   "env-remove",
                onclick: move |_| props.on_remove.call(()),
                "×"
            }
        }
    }
}