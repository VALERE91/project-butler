# Project Butler

A desktop application for organizing and launching your development tools, services, and scripts — all from one place.

Built with Rust and [Dioxus](https://dioxuslabs.com/).

---

## Overview

Project Butler lets you group related commands and applications into **projects**, then launch them with a single click. Each running process streams its output in real time to a tabbed log panel at the bottom of the screen.

It's designed for developers who regularly juggle multiple services, scripts, or tools across different codebases.

---

## Features

- **Project organization** — group apps and commands into named projects with custom colors and icons
- **One-click launch** — run any configured command without opening a terminal
- **Real-time log output** — stdout and stderr streamed to a tabbed panel, with stderr clearly labeled
- **Process control** — kill running processes directly from the UI
- **Environment variables** — configure per-app env vars, working directories, and arguments
- **Persistent config** — everything is saved to `config.json` automatically
- **Native file dialogs** — pick executables, working directories, and icon images from your filesystem
- **Icon support** — use emojis or image files (PNG, SVG, JPG, WebP) as project and app icons

---

## Getting Started

### Prerequisites

- [Rust](https://rustup.rs/) (stable)
- [Dioxus CLI](https://dioxuslabs.com/learn/0.7/getting_started)

```bash
cargo install dioxus-cli
```

### Running

```bash
dx serve --platform desktop
```

On first launch, a `config.json` will be created in the project root to store your configuration.

---

## Usage

1. **Add a project** — click `+ Add Project` in the sidebar, give it a name, color, and optional icon
2. **Add apps** — select a project, click `+ Add App`, and configure the command, arguments, working directory, and environment variables
3. **Launch** — click `Launch` on any app card to run it
4. **Monitor** — view live output in the log panel at the bottom; switch between processes using the tabs
5. **Stop** — click `Kill` on any running process tab to terminate it

---

## Configuration

All configuration is stored in `config.json` at the project root:

```json
{
  "projects": [
    {
      "id": "uuid",
      "name": "My Project",
      "description": "Optional description",
      "color": "#6366f1",
      "icon": "🚀",
      "apps": [
        {
          "id": "uuid",
          "name": "Dev Server",
          "description": "Start the development server",
          "icon": "⚡",
          "command": "npm",
          "args": ["run", "dev"],
          "env": { "NODE_ENV": "development" },
          "cwd": "/path/to/project",
          "confirm": false
        }
      ]
    }
  ]
}
```

The file is updated automatically as you add, edit, or remove projects and apps through the UI. You can also edit it manually.

---

## Project Structure

```
project-butler/
├── src/
│   ├── main.rs              # App entry point and root layout
│   ├── config.rs            # Config loading and saving
│   ├── process/
│   │   └── mod.rs           # Process spawning and output capture
│   └── components/
│       ├── sidenav.rs       # Project list sidebar
│       ├── main_content.rs  # App card grid
│       ├── log_panel.rs     # Tabbed process output panel
│       ├── toast.rs         # Notification toasts
│       └── modal/           # Add/edit forms for projects and apps
├── assets/
│   └── tailwind.css         # Compiled Tailwind CSS
├── config.json              # Your saved projects and apps
├── Cargo.toml
└── Dioxus.toml
```

---

## Tech Stack

| **Component** | **Details**                              |
|---------------|------------------------------------------|
| Language      | Rust (Edition 2021)                      |
| UI Framework  | [Dioxus](https://dioxuslabs.com/) 0.7    |
| Async Runtime | [Tokio](https://tokio.rs/)               |
| Styling       | [Tailwind CSS](https://tailwindcss.com/) |
| Serialization | serde / serde_json                       |
| File Dialogs  | [rfd](https://github.com/PolyMeilex/rfd) |