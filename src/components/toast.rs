use dioxus::prelude::*;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq)]
pub enum ToastKind {
    Success,
    Error,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Toast {
    pub id:      Uuid,
    pub message: String,
    pub kind:    ToastKind,
}

impl Toast {
    pub fn success(message: impl Into<String>) -> Self {
        Self { id: Uuid::new_v4(), message: message.into(), kind: ToastKind::Success }
    }

    pub fn error(message: impl Into<String>) -> Self {
        Self { id: Uuid::new_v4(), message: message.into(), kind: ToastKind::Error }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct ToastListProps {
    pub toasts: Signal<Vec<Toast>>,
}

#[component]
pub fn ToastList(props: ToastListProps) -> Element {
    rsx! {
        div { class: "toast-list",
            for toast in props.toasts.read().iter() {
                ToastItem {
                    key:    "{toast.id}",
                    toast:  toast.clone(),
                    toasts: props.toasts,
                }
            }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
struct ToastItemProps {
    toast:  Toast,
    toasts: Signal<Vec<Toast>>,
}

#[component]
fn ToastItem(props: ToastItemProps) -> Element {
    let mut toasts = props.toasts;
    let id         = props.toast.id;

    // Auto-dismiss after 3 seconds
    use_effect(move || {
        spawn(async move {
            tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
            toasts.write().retain(|t| t.id != id);
        });
    });

    let (icon, class) = match props.toast.kind {
        ToastKind::Success => ("✓", "toast toast-success"),
        ToastKind::Error   => ("✗", "toast toast-error"),
    };

    rsx! {
        div { class: "{class}",
            span { class: "toast-icon", "{icon}" }
            span { class: "toast-message", "{props.toast.message}" }
            button {
                class:   "toast-close",
                onclick: move |_| toasts.write().retain(|t| t.id != id),
                "×"
            }
        }
    }
}

/// Call this anywhere you have access to the toasts signal
pub fn push_toast(mut toasts: Signal<Vec<Toast>>, toast: Toast) {
    toasts.write().push(toast);
}