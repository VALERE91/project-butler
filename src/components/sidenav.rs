use dioxus::prelude::*;
use crate::config::{Config, Project};

#[derive(Props, Clone, PartialEq)]
pub struct SideNavProps {
    pub projects:  Vec<Project>,
    pub selected:  Signal<usize>,
    pub collapsed: Signal<bool>,
    pub on_add:    Signal<bool>,
}

#[component]
pub fn SideNav(props: SideNavProps) -> Element {
    let collapsed = props.collapsed;
    let nav_class = if *collapsed.read() { "sidenav collapsed" } else { "sidenav" };
    let mut show_add_project = props.on_add.clone();

    rsx! {
        nav { class: "{nav_class}",
            NavHeader { collapsed }
            div { class: "nav-items",
                for (i, project) in props.projects.iter().enumerate() {
                    NavItem {
                        key:       "{i}",
                        project:   project.clone(),
                        index:     i,
                        selected:  props.selected,
                        collapsed,
                    }
                }
                button {
                    class:   "btn-add-project",
                    onclick: move |_| *show_add_project.write() = true,
                    "+ Add Project"
                }
            }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
struct NavHeaderProps {
    collapsed: Signal<bool>,
}

#[component]
fn NavHeader(props: NavHeaderProps) -> Element {
    let mut collapsed = props.collapsed;
    let icon = if *collapsed.read() { "→" } else { "←" };

    rsx! {
        div { class: "nav-header",
            if !*collapsed.read() {
                span { class: "nav-title", "Projects" }
            }
            button {
                class:   "collapse-btn",
                onclick: move |_| {
                    let current = *collapsed.read();
                    collapsed.set(!current);
                },
                "{icon}"
            }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
struct NavItemProps {
    project:   Project,
    index:     usize,
    selected:  Signal<usize>,
    collapsed: Signal<bool>,
}

#[component]
fn NavItem(props: NavItemProps) -> Element {
    let mut selected = props.selected;
    let accent = props.project.color.clone().unwrap_or("#3b82f6".into());
    let icon_data = props.project.icon_data;
    let icon = props.project.icon;

    rsx! {
        div {
            class: if *selected.read() == props.index { "nav-item active" } else { "nav-item" },
            style: if *selected.read() == props.index {
                "--accent: {accent}; border-left-color: {accent};"
            } else {
                "--accent: {accent}; border-left-color: transparent;"
            },
            onclick: move |_| {
                let idx = props.index;
                selected.set(idx);
            },
            if let Some(data_url) = &icon_data {
                img { class: "nav-icon-img", src: "{data_url}" }
            } else if let Some(emoji) = &icon {
                span { class: "nav-icon", "{emoji}" }
            }
            if !*props.collapsed.read() {
                span { class: "nav-label", "{props.project.name}" }
            }
        }
    }
}