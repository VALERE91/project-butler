use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::mpsc;
use uuid::Uuid;
use crate::config::App;
use std::process::Stdio;
use dioxus::prelude::{info, UnboundedSender};

#[derive(Debug, Clone)]
pub enum ProcessEvent {
    Line   { instance_id: Uuid, text: String },
    Exited { instance_id: Uuid, code: i32 },
    Failed { app_id: Uuid, reason: String },
}

#[derive(Debug, Clone, PartialEq)]
pub enum ProcessStatus {
    Running,
    Exited(i32),
    Killed,
}

#[derive(Debug, Clone)]
pub struct RunningProcess {
    pub instance_id: Uuid,
    pub app_id: Uuid,
    pub name:   String,
    pub status: ProcessStatus,
    pub logs:   Vec<String>,
    killer:     mpsc::Sender<()>,
}

impl PartialEq for RunningProcess {
    fn eq(&self, other: &Self) -> bool {
        self.instance_id == other.instance_id
    }
}

impl RunningProcess {
    pub fn kill(&self) {
        let _ = self.killer.try_send(());
    }

    pub fn is_running(&self) -> bool {
        self.status == ProcessStatus::Running
    }
}

pub async fn spawn(
    app:      &App,
    mut event_tx: UnboundedSender<ProcessEvent>,
) -> Option<RunningProcess> {
    let instance_id = Uuid::new_v4();
    let mut cmd = Command::new(&app.command);
    cmd.args(&app.args);
    for (k, v) in &app.env {
        cmd.env(k, v);
    }
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());
    cmd.kill_on_drop(true);

    if let Some(dir) = &app.cwd {
        cmd.current_dir(dir);
    }

    let mut child: Child = match cmd.spawn() {
        Ok(c)  => c,
        Err(e) => {
            let _ = event_tx.unbounded_send(ProcessEvent::Failed {
                app_id: app.id,
                reason: e.to_string(),
            });
            return None;
        }
    };

    let app_id   = app.id;
    let app_name = app.name.clone();

    // Kill channel
    let (kill_tx, mut kill_rx) = mpsc::channel::<()>(1);

    // Grab stdout / stderr handles before moving child into the task
    let stdout = child.stdout.take().map(BufReader::new);
    let stderr = child.stderr.take().map(BufReader::new);

    tokio::spawn(async move {
        let mut stdout_lines = stdout.map(|r| r.lines());
        let mut stderr_lines = stderr.map(|r| r.lines());

        loop {
            tokio::select! {
                // Kill requested from UI
                _ = kill_rx.recv() => {
                    let _ = child.kill().await;
                    let _ = event_tx.unbounded_send(ProcessEvent::Exited {
                        instance_id,
                        code: -1,
                    });
                    break;
                }

                // stdout line
                line = async {
                    match stdout_lines.as_mut() {
                        Some(l) => l.next_line().await,
                        None    => std::future::pending().await,
                    }
                } => {
                    match line {
                        Ok(Some(text)) => {
                            let _ = event_tx.unbounded_send(ProcessEvent::Line { instance_id, text });
                        }
                        _ => {} // EOF or error — wait for exit event
                    }
                }

                // stderr line
                line = async {
                    match stderr_lines.as_mut() {
                        Some(l) => l.next_line().await,
                        None    => std::future::pending().await,
                    }
                } => {
                    match line {
                        Ok(Some(text)) => {
                            let _ = event_tx.unbounded_send(ProcessEvent::Line {
                                instance_id,
                                text: format!("[err] {text}"),
                            });
                        }
                        _ => {}
                    }
                }

                // Natural exit
                status = child.wait() => {
                    let code = status.ok()
                        .and_then(|s| s.code())
                        .unwrap_or(-1);
                    let _ = event_tx.unbounded_send(ProcessEvent::Exited { instance_id, code });
                    break;
                }
            }
        }

        info!("[launcher] {} exited", app_name);
    });

    Some(RunningProcess {
        instance_id,
        app_id,
        name:   app.name.clone(),
        status: ProcessStatus::Running,
        logs:   Vec::new(),
        killer: kill_tx,
    })
}