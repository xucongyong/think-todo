use crate::tmux::Tmux;
use anyhow::Result;
use std::fs;
use std::path::PathBuf;

pub struct Worker { 
    pub id: String, 
    pub name: String, 
    pub work_dir: PathBuf,
    pub engine: String,
    pub role: String, // mayor, worker, witness
}

impl Worker {
    pub fn new(id: String, name: String, work_dir: PathBuf, engine: String, role: String) -> Self { 
        Self { id, name, work_dir, engine, role } 
    }
    pub fn spawn(&self) -> Result<()> {
        let session_name = format!("worker-{}", self.name);
        let worker_path = self.work_dir.join("workers").join(&self.name);
        let _ = fs::create_dir_all(&worker_path);
        
        let base_prompt = fs::read_to_string(self.work_dir.join("prompts").join("base.md")).unwrap_or_default();
        let role_prompt = fs::read_to_string(self.work_dir.join("prompts").join("roles").join(format!("{}.md", self.role)))
            .unwrap_or_else(|_| "You are a specialized agent.".to_string());
        
        let final_instruction = format!("{}\n\n{}\n\nMISSION ID: {}\nMISSIONS: {}\n\nEXECUTE NOW.", 
            base_prompt, role_prompt, self.id, self.id);
        
        let log_dir = self.work_dir.join(".logs").join("tasks").join(&self.id);
        let _ = fs::create_dir_all(&log_dir);
        let log_file = log_dir.join(format!("{}.log", self.name));

        // Choose CLI tool based on engine
        let engine_cmd = match self.engine.as_str() {
            "opencode" => format!("opencode \"{}\"", final_instruction.replace("\"", "\\\"")),
            "claude" => format!("claude \"{}\"", final_instruction.replace("\"", "\\\"")),
            _ => format!("gemini --approval-mode yolo \"{}\"", final_instruction.replace("\"", "\\\"")),
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