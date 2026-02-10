use crate::tmux::Tmux;
use crate::db::Db;
use anyhow::Result;
use std::fs;
use std::path::PathBuf;

pub struct Admin { pub session_name: String, pub work_dir: PathBuf }

impl Admin {
    pub fn new(work_dir: PathBuf) -> Self { Self { session_name: "hq-admin".to_string(), work_dir } }
    pub fn start(&self) -> Result<()> {
        if Tmux::has_session(&self.session_name) { println!("Admin already running."); return Ok(()); }
        let prompt_path = self.work_dir.join("prompts").join("admin.md");
        let mut instruction = fs::read_to_string(prompt_path).unwrap_or_else(|_| "You are Think Todo Admin.".to_string());
        let db = Db::new(self.work_dir.clone())?;
        let mut stmt = db.conn.prepare("SELECT id, title FROM tasks WHERE status = 'open'")?;
        let tasks = stmt.query_map([], |row| Ok(format!("- [{}] {}", row.get::<_, String>(0)?, row.get::<_, String>(1)?)))?;
        instruction.push_str("\n\nPending Tasks:\n");
        for t in tasks { instruction.push_str(&t?); instruction.push_str("\n"); }
        let admin_dir = self.work_dir.join("admin");
        let _ = fs::create_dir_all(&admin_dir);
        let cmd = format!("cd {} && gemini --approval-mode yolo \"{}\"", admin_dir.display(), instruction.replace("\"", "\\\""));
        Tmux::new_session(&self.session_name, &cmd)?;
        println!("🚀 Think Todo Admin is online!");
        Ok(())
    }
    pub fn attach(&self) -> Result<()> {
        let _ = std::process::Command::new("tmux").args(&["attach-session", "-t", &self.session_name]).status()?; Ok(())
    }
}
