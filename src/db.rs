use rusqlite::{params, Connection, Result};
use std::path::PathBuf;

pub struct Db {
    pub conn: Connection,
}

impl Db {
    pub fn new(work_dir: PathBuf) -> anyhow::Result<Self> {
        let db_path = work_dir.join("think.db");
        let conn = Connection::open(db_path)?;
        conn.execute("CREATE TABLE IF NOT EXISTS tasks (id TEXT PRIMARY KEY, title TEXT, status TEXT DEFAULT 'open', assignee TEXT, created_at INTEGER)", [])?;
        conn.execute("CREATE TABLE IF NOT EXISTS audit_logs (id INTEGER PRIMARY KEY AUTOINCREMENT, actor TEXT, action TEXT, target TEXT, status TEXT, timestamp INTEGER)", [])?;
        conn.execute("CREATE TABLE IF NOT EXISTS messages (id INTEGER PRIMARY KEY AUTOINCREMENT, sender TEXT, receiver TEXT, subject TEXT, body TEXT, status TEXT DEFAULT 'unread', timestamp INTEGER)", [])?;
        conn.execute("CREATE TABLE IF NOT EXISTS rigs (name TEXT PRIMARY KEY, path TEXT, repo TEXT, status TEXT DEFAULT 'active', last_sync INTEGER)", [])?;
        conn.execute("CREATE TABLE IF NOT EXISTS costs (id INTEGER PRIMARY KEY AUTOINCREMENT, task_id TEXT, agent_name TEXT, model TEXT, input_tokens INTEGER, output_tokens INTEGER, cost_usd REAL, timestamp INTEGER)", [])?;
        Ok(Self { conn })
    }
    pub fn add_task(&self, id: &str, title: &str) -> Result<()> {
        self.conn.execute("INSERT INTO tasks (id, title, created_at) VALUES (?1, ?2, strftime('%s','now'))", params![id, title])?;
        Ok(())
    }
    pub fn log_audit(&self, actor: &str, action: &str, target: &str, status: &str) -> Result<()> {
        self.conn.execute("INSERT INTO audit_logs (actor, action, target, status, timestamp) VALUES (?1, ?2, ?3, ?4, strftime('%s','now'))", params![actor, action, target, status])?;
        Ok(())
    }

    pub fn log_cost(&self, task_id: &str, agent_name: &str, model: &str, input: i32, output: i32, cost: f64) -> Result<()> {
        self.conn.execute(
            "INSERT INTO costs (task_id, agent_name, model, input_tokens, output_tokens, cost_usd, timestamp) VALUES (?1, ?2, ?3, ?4, ?5, ?6, strftime('%s','now'))",
            params![task_id, agent_name, model, input, output, cost]
        )?;
        Ok(())
    }

    // Mail helpers
    pub fn send_mail(&self, sender: &str, receiver: &str, subject: &str, body: &str) -> Result<()> {
        self.conn.execute(
            "INSERT INTO messages (sender, receiver, subject, body, timestamp) VALUES (?1, ?2, ?3, ?4, strftime('%s','now'))",
            params![sender, receiver, subject, body]
        )?;
        Ok(())
    }

    // Rig helpers
    pub fn add_rig(&self, name: &str, path: &str, repo: &str) -> Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO rigs (name, path, repo, last_sync) VALUES (?1, ?2, ?3, strftime('%s','now'))",
            params![name, path, repo]
        )?;
        Ok(())
    }
}
