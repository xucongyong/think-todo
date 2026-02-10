use serde::{Serialize, Deserialize};
use anyhow::Result;
use std::fs;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Task {
    pub id: String,
    pub title: String,
    pub assignee: Option<String>,
    pub status: TaskStatus,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum TaskStatus {
    Open,
    InProgress,
    Closed,
}

pub struct Store {
    pub path: PathBuf,
}

impl Store {
    pub fn new(work_dir: PathBuf) -> Self {
        Self {
            path: work_dir.join("tasks.json"),
        }
    }

    pub fn save(&self, tasks: Vec<Task>) -> Result<()> {
        let content = serde_json::to_string_pretty(&tasks)?;
        fs::write(&self.path, content)?;
        Ok(())
    }

    pub fn load(&self) -> Result<Vec<Task>> {
        if !self.path.exists() {
            return Ok(Vec::new());
        }
        let content = fs::read_to_string(&self.path)?;
        let tasks = serde_json::from_str(&content)?;
        Ok(tasks)
    }

    pub fn add_task(&self, id: &str, title: &str) -> Result<()> {
        let mut tasks = self.load()?;
        tasks.push(Task {
            id: id.to_string(),
            title: title.to_string(),
            assignee: None,
            status: TaskStatus::Open,
        });
        self.save(tasks)
    }
}
