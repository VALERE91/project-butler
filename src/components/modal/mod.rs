pub mod add_project;
pub use add_project::AddProjectModal;
pub mod add_app;
pub use add_app::AddAppModal;

use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct ModalProps {
    pub title:    String,
    pub on_close: EventHandler<()>,
    pub children: Element,
    #[props(default = "480px".to_string())]
    pub width:    String,
}

#[component]
pub fn Modal(props: ModalProps) -> Element {
    // Close on ESC key
    use_effect(move || {
        document::eval(r#"
            window.__modal_handler = function(e) {
                if (e.key === 'Escape') {
                    window.__modal_close && window.__modal_close();
                }
            };
            window.addEventListener('keydown', window.__modal_handler);
        "#);
        // cleanup on unmount
        use_drop(move || {
            document::eval(r#"
                window.removeEventListener('keydown', window.__modal_handler);
            "#);
        });
    });

    let on_close = props.on_close.clone();
    use_effect(move || {
        let _ = on_close; // suppress unused warning
    });

    let on_close_esc  = props.on_close.clone();
    let on_close_back = props.on_close.clone();
    let width         = props.width.clone();

    rsx! {
        // Backdrop
        div {
            class:   "modal-backdrop",
            onclick: move |_| on_close_back.call(()),

            // Dialog — stop click propagation so clicking inside doesn't close
            div {
                class:         "modal-dialog",
                style:         "max-width: {width};",
                onclick:       move |e| e.stop_propagation(),
                onkeydown:     move |e| { if e.key() == Key::Escape { on_close_esc.call(()); } },

                // Header
                div { class: "modal-header",
                    h2 { class: "modal-title", "{props.title}" }
                    button {
                        class:   "modal-close-btn",
                        onclick: move |_| props.on_close.call(()),
                        "×"
                    }
                }

                // Content slot
                div { class: "modal-body",
                    {props.children}
                }
            }
        }
    }
}

/// A labelled form field wrapper
#[derive(Props, Clone, PartialEq)]
pub struct FieldProps {
    pub label:    String,
    #[props(default)]
    pub hint:     String,
    pub children: Element,
}

#[component]
pub fn Field(props: FieldProps) -> Element {
    rsx! {
        div { class: "form-field",
            label { class: "form-label", "{props.label}" }
            if !props.hint.is_empty() {
                span { class: "form-hint", "{props.hint}" }
            }
            {props.children}
        }
    }
}

/// Standard text input
#[derive(Props, Clone, PartialEq)]
pub struct TextInputProps {
    pub value:       String,
    pub placeholder: String,
    pub on_change:   EventHandler<String>,
    #[props(default)]
    pub monospace:   bool,
}

#[component]
pub fn TextInput(props: TextInputProps) -> Element {
    let class = if props.monospace { "form-input monospace" } else { "form-input" };
    rsx! {
        input {
            class:       "{class}",
            r#type:      "text",
            value:       "{props.value}",
            placeholder: "{props.placeholder}",
            oninput:     move |e| props.on_change.call(e.value()),
        }
    }
}

/// Textarea
#[derive(Props, Clone, PartialEq)]
pub struct TextAreaProps {
    pub value:       String,
    pub placeholder: String,
    pub on_change:   EventHandler<String>,
    #[props(default = 3)]
    pub rows:        u32,
}

#[component]
pub fn TextArea(props: TextAreaProps) -> Element {
    rsx! {
        textarea {
            class:       "form-input form-textarea",
            rows:        "{props.rows}",
            placeholder: "{props.placeholder}",
            oninput:     move |e| props.on_change.call(e.value()),
            "{props.value}"
        }
    }
}

/// Path input with Browse button
#[derive(Props, Clone, PartialEq)]
pub struct PathInputProps {
    pub value:       String,
    pub placeholder: String,
    pub on_change:   EventHandler<String>,
    pub on_browse:   EventHandler<()>,
    #[props(default = "Browse…".to_string())]
    pub btn_label:   String,
}

#[component]
pub fn PathInput(props: PathInputProps) -> Element {
    rsx! {
        div { class: "path-input-row",
            input {
                class:       "form-input",
                r#type:      "text",
                value:       "{props.value}",
                placeholder: "{props.placeholder}",
                oninput:     move |e| props.on_change.call(e.value()),
            }
            button {
                class:   "btn-browse",
                onclick: move |_| props.on_browse.call(()),
                "{props.btn_label}"
            }
        }
    }
}

/// Modal action buttons row
#[derive(Props, Clone, PartialEq)]
pub struct ModalActionsProps {
    pub on_cancel:  EventHandler<()>,
    pub on_confirm: EventHandler<()>,
    #[props(default = "Cancel".to_string())]
    pub cancel_label: String,
    #[props(default = "Confirm".to_string())]
    pub confirm_label: String,
    #[props(default)]
    pub confirm_disabled: bool,
}

#[component]
pub fn ModalActions(props: ModalActionsProps) -> Element {
    rsx! {
        div { class: "modal-actions",
            button {
                class:   "btn-secondary",
                onclick: move |_| props.on_cancel.call(()),
                "{props.cancel_label}"
            }
            button {
                class:    if props.confirm_disabled { "btn-primary disabled" } else { "btn-primary" },
                disabled: props.confirm_disabled,
                onclick:  move |_| { if !props.confirm_disabled { props.on_confirm.call(()); } },
                "{props.confirm_label}"
            }
        }
    }
}