use dioxus::prelude::*;
use crate::config::{Config, Project};
use crate::components::modal::{
    Modal, Field, TextInput, TextArea, PathInput, ModalActions,
};
use uuid::Uuid;

#[derive(Clone, PartialEq, Default)]
struct ProjectForm {
    name:        String,
    description: String,
    icon_path:   String,
    color:       String,
}

impl ProjectForm {
    fn is_valid(&self) -> bool {
        !self.name.trim().is_empty()
    }

    fn into_project(self) -> Project {
        Project {
            id:          Uuid::new_v4(),
            name:        self.name.trim().to_string(),
            description: Some(self.description.trim().to_string()),
            icon:        if self.icon_path.is_empty() {
                None
            } else {
                Some(self.icon_path)
            },
            icon_data:   None,
            color:       if self.color.is_empty() {
                None
            } else {
                Some(self.color)
            },
            apps:        vec![],
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct AddProjectModalProps {
    pub config:   Signal<Config>,
    pub on_close: EventHandler<()>,
    pub on_saved: EventHandler<String>, // emits project name for toast
}

#[component]
pub fn AddProjectModal(props: AddProjectModalProps) -> Element {
    let mut form     = use_signal(ProjectForm::default);
    let mut config   = props.config;
    let on_close     = props.on_close.clone();

    // ── Handlers ──────────────────────────────────────────────────────────────

    let on_browse_icon = move |_| {
        let mut form = form.clone();
        spawn(async move {
            let file = rfd::AsyncFileDialog::new()
                .set_title("Pick project icon")
                .add_filter("Image", &["png", "svg", "jpg", "webp"])
                .pick_file()
                .await;
            if let Some(f) = file {
                form.write().icon_path = f.path().to_string_lossy().into_owned();
            }
        });
    };

    let on_confirm = move |_| {
        let f = form.read().clone();
        if !f.is_valid() { return; }

        let project_name = f.name.clone();
        let project      = f.into_project();

        config.write().projects.push(project);

        if let Err(e) = config.read().save("config.json") {
            eprintln!("[launcher] Failed to save config: {e}");
        }

        props.on_saved.call(project_name);
        props.on_close.call(());
    };

    // ── Render ────────────────────────────────────────────────────────────────

    let form_read    = form.read();
    let color_value  = if form_read.color.is_empty() { "#3b82f6".to_string() } else { form_read.color.clone() };

    rsx! {
        Modal {
            title:    "Add Project",
            on_close: props.on_close,

            // Name
            Field { label: "Name", hint: "Required",
                TextInput {
                    value:       form_read.name.clone(),
                    placeholder: "e.g. Game Client",
                    on_change:   move |v| form.write().name = v,
                }
            }

            // Description
            Field { label: "Description",
                TextArea {
                    value:       form_read.description.clone(),
                    placeholder: "Short description of this project…",
                    on_change:   move |v| form.write().description = v,
                }
            }

            // Icon
            Field { label: "Icon", hint: "PNG, SVG, JPG or WebP",
                PathInput {
                    value:       form_read.icon_path.clone(),
                    placeholder: "/path/to/icon.png",
                    btn_label:   "Browse…",
                    on_change:   move |v| form.write().icon_path = v,
                    on_browse:   on_browse_icon,
                }
            }

            // Color
            Field { label: "Accent Color",
                div { class: "color-row",
                    input {
                        r#type:   "color",
                        class:    "color-picker",
                        value:    "{color_value}",
                        oninput:  move |e| form.write().color = e.value(),
                    }
                    input {
                        r#type:      "text",
                        class:       "form-input color-hex",
                        value:       "{color_value}",
                        placeholder: "#3b82f6",
                        oninput:     move |e| form.write().color = e.value(),
                    }
                }
            }

            // Actions
            ModalActions {
                on_cancel:        on_close,
                on_confirm,
                confirm_label:    "Add Project",
                confirm_disabled: !form_read.is_valid(),
            }
        }
    }
}