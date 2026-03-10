use dioxus::prelude::*;
use crate::config::{Project, App};

#[derive(Props, Clone, PartialEq)]
pub struct MainContentProps {
    pub project: Project,
    pub on_launch: EventHandler<App>,
    pub add_app: Signal<bool>,
}

#[component]
pub fn MainContent(props: MainContentProps) -> Element {
    let project = &props.project;
    let mut add_app = props.add_app.clone();
    let accent  = project.color.clone().unwrap_or("#3b82f6".into());

    rsx! {
        div { class: "main-content",

            // ── Header ───────────────────────────────────────────────────────
            div { class: "content-header",
                style: "border-bottom: 2px solid {accent};",

                if let Some(data_url) = &project.icon_data {
                    img { class: "nav-icon-img", src: "{data_url}" }
                } else if let Some(emoji) = &project.icon {
                    span { class: "nav-icon", "{emoji}" }
                }
                div { class: "content-header-text",
                    h1 { class: "content-header-title", "{project.name}" }
                    span { class: "content-header-count",
                        "{project.apps.len()} app"
                        if project.apps.len() != 1 { "s" } else { "" }
                    }
                }

                button {
                    class:   "btn-add-app",
                    onclick: move |_| *add_app.write() = true,
                    "+ Add App"
                }
            }

            // ── Grid ─────────────────────────────────────────────────────────
            if project.apps.is_empty() {
                div { class: "content-empty",
                    span { "No apps configured for this project." }
                }
            } else {
                div { class: "app-grid",
                    for app in project.apps.iter() {
                        AppCard {
                            key:       "{app.id}",
                            app:       app.clone(),
                            accent:    accent.clone(),
                            on_launch: props.on_launch,
                        }
                    }
                }
            }
        }
    }
}

// ── AppCard ───────────────────────────────────────────────────────────────────

#[derive(Props, Clone, PartialEq)]
struct AppCardProps {
    app:       App,
    accent:    String,
    on_launch: EventHandler<App>,
}

#[component]
fn AppCard(props: AppCardProps) -> Element {
    let app    = props.app.clone();
    let accent = props.accent.clone();

    rsx! {
        div { class: "app-card",

            // Icon area
            div { class: "app-card-icon", style: "background: {accent}22;",
                span {
                    if let Some(icon) = &props.app.icon {
                        "{icon}"
                    } else {
                        // fallback: first letter of app name
                        "{props.app.name.chars().next().unwrap_or('?')}"
                    }
                }
            }

            // Text
            div { class: "app-card-body",
                h3 { class: "app-card-name", "{props.app.name}" }
                if !props.app.description.is_empty() {
                    p { class: "app-card-desc", "{props.app.description}" }
                }
            }

            // Launch button
            button {
                class:   "app-card-btn",
                style:   "background: {accent};",
                onclick: move |_| props.on_launch.call(app.clone()),
                "▶  Launch"
            }
        }
    }
}