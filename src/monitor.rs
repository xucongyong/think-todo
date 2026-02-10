use crate::db::Db;
use anyhow::Result;
use std::fs;
use std::path::PathBuf;
use std::thread;
use std::time::Duration;

pub struct Monitor { pub work_dir: PathBuf }

impl Monitor {
    pub fn new(work_dir: PathBuf) -> Self { Self { work_dir } }
    pub fn watch(&self) -> Result<()> {
        let db = Db::new(self.work_dir.clone())?;
        let logs_dir = self.work_dir.join(".logs").join("tasks");
        println!("👀 Monitor started...");
        loop {
            if logs_dir.exists() {
                if let Ok(entries) = fs::read_dir(&logs_dir) {
                    for entry in entries.flatten() {
                        let path = entry.path(); 
                        if !path.is_dir() { continue; }
                        let task_id = path.file_name().unwrap().to_string_lossy().to_string();
                        // Fix: Iterate over &path so we don't move it
                        if let Ok(log_files) = fs::read_dir(&path) {
                            for log_file in log_files.flatten() {
                                let content = fs::read_to_string(log_file.path()).unwrap_or_default();
                                if content.contains("[TASK_DONE]") {
                                    let _ = db.conn.execute("UPDATE tasks SET status = 'closed' WHERE id = ?1", [task_id.clone()]);
                                }
                            }
                        }
                    }
                }
            }
            thread::sleep(Duration::from_secs(3));
        }
    }
}
