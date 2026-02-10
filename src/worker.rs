use crate::tmux::Tmux;
use anyhow::Result;
use std::fs;
use std::path::PathBuf;

pub struct Worker { 
    pub id: String, 
    pub name: String, 
    pub work_dir: PathBuf,
    pub engine: String, // gemini, opencode, claude
}

impl Worker {
    pub fn new(id: String, name: String, work_dir: PathBuf, engine: String) -> Self { 
        Self { id, name, work_dir, engine } 
    }
    pub fn spawn(&self) -> Result<()> {
        let session_name = format!("worker-{}", self.name);
        let worker_path = self.work_dir.join("workers").join(&self.name);
        let _ = fs::create_dir_all(&worker_path);
        
        let prompt_path = self.work_dir.join("prompts").join("worker.md");
        let base = fs::read_to_string(prompt_path).unwrap_or_else(|_| "You are Think Todo Worker.".to_string());
        
        let log_dir = self.work_dir.join(".logs").join("tasks").join(&self.id);
        let _ = fs::create_dir_all(&log_dir);
        let log_file = log_dir.join(format!("{}.log", self.name));

        let final_prompt = format!("{} MISSION: {} . Start coding now.", base.replace("\"", "'"), self.id.replace("\"", "'"));
        
        // Choose CLI tool based on engine
        let engine_cmd = match self.engine.as_str() {
            "opencode" => format!("opencode \"{}\"", final_prompt.replace("\"", "\\\"")),
            "claude" => format!("claude \"{}\"", final_prompt.replace("\"", "\\\"")),
            _ => format!("gemini --approval-mode yolo \"{}\"", final_prompt.replace("\"", "\\\"")),
        };

        let cmd = format!("export PATH=$PATH:/Users/xucongyong/.bun/bin && cd {} && ({} 2>&1 | tee {})", 
            worker_path.display(), 
            engine_cmd,
            log_file.display()
        );
        
        Tmux::new_session(&session_name, &cmd)?;
        println!("✅ Worker {} dispatched with engine {}!", self.name, self.engine);
        Ok(())
    }
    pub fn nuke(name: &str, work_dir: &PathBuf) -> Result<()> {
        let _ = Tmux::kill_session(&format!("worker-{}", name));
        let worker_path = work_dir.join("workers").join(name);
        let _ = fs::remove_dir_all(worker_path);
        Ok(())
    }
}