use std::process::Command;
use anyhow::{Result, Context};

pub struct Tmux;

impl Tmux {
    fn run(args: &[&str]) -> Result<String> {
        let output = Command::new("tmux").args(args).output().with_context(|| format!("Tmux failed: {:?}", args))?;
        if !output.status.success() {
            let err = String::from_utf8_lossy(&output.stderr);
            if err.contains("duplicate") { return Ok(String::new()); }
            anyhow::bail!("Tmux error: {}", err.trim());
        }
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }
    pub fn new_session(name: &str, cmd: &str) -> Result<()> { Self::run(&["new-session", "-d", "-s", name, cmd])?; Ok(()) }
    pub fn kill_session(name: &str) -> Result<()> { let _ = Command::new("tmux").args(&["kill-session", "-t", name]).status(); Ok(()) }
    pub fn has_session(name: &str) -> bool { Command::new("tmux").args(&["has-session", "-t", name]).status().map(|s| s.success()).unwrap_or(false) }
    pub fn display_message(session: &str, msg: &str) -> Result<()> { Self::run(&["display-message", "-t", session, msg])?; Ok(()) }
}
