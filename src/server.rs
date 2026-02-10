use axum::{
    extract::Path,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use tower_http::services::ServeDir;
use crate::db::Db;
use crate::worker::Worker;
use crate::tmux::Tmux;
use std::env;
use std::fs;

#[derive(Serialize)]
struct DashboardData {
    tasks: Vec<TaskData>,
    agents: Vec<String>,
    recent_logs: Vec<LogData>,
    stats: StatsData,
}

#[derive(Serialize, Deserialize, Clone)]
struct TaskData {
    id: String,
    title: String,
    status: String,
    assignee: Option<String>,
    engine: Option<String>,
}

#[derive(Serialize)]
struct LogData {
    timestamp: i64,
    actor: String,
    action: String,
    target: String,
}

#[derive(Serialize)]
struct StatsData {
    total_cost: f64,
    tasks_done: i64,
    tasks_total: i64,
}

#[derive(Serialize)]
struct AgentLogResponse {
    content: String,
    path: String,
}

#[derive(Deserialize)]
struct AddTaskRequest {
    id: String,
    title: String,
}

#[derive(Deserialize)]
struct SlingRequest {
    task_id: String,
    agent_name: String,
    engine: String,
}

#[derive(Deserialize)]
struct NudgeRequest {
    agent_name: String,
    message: String,
}

pub async fn start_server(port: u16) {
    let app = Router::new()
        .route("/api/dashboard", get(get_dashboard))
        .route("/api/logs/{task_id}/{agent_name}", get(get_agent_logs))
        .route("/api/prompts/{role}", get(get_prompt))
        .route("/api/agents/{agent_name}/files", get(list_agent_files))
        .route("/api/tasks/{task_id}/history", get(get_task_history))
        // Actions
        .route("/api/tasks", post(add_task))
        .route("/api/tasks/{task_id}", axum::routing::delete(delete_task))
        .route("/api/start", post(start_task))
        .route("/api/done/{task_id}", post(done_task))
        .route("/api/nudge", post(nudge_agent))
        .fallback_service(ServeDir::new("ui"));

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    println!("🌐 Think-Todo WebUI is running at: http://localhost:{}", port);
    
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn add_task(Json(req): Json<AddTaskRequest>) -> Json<serde_json::Value> {
    let work_dir = env::current_dir().unwrap();
    let db = Db::new(work_dir).unwrap();
    match db.add_task(&req.id, &req.title) {
        Ok(_) => Json(serde_json::json!({"status": "success"})),
        Err(e) => Json(serde_json::json!({"status": "error", "message": e.to_string()})),
    }
}

async fn delete_task(Path(task_id): Path<String>) -> Json<serde_json::Value> {
    let work_dir = env::current_dir().unwrap();
    let db = Db::new(work_dir).unwrap();
    let _ = db.conn.execute("DELETE FROM tasks WHERE id = ?1", rusqlite::params![task_id]);
    Json(serde_json::json!({"status": "success"}))
}

async fn start_task(Json(req): Json<SlingRequest>) -> Json<serde_json::Value> {
    let work_dir = env::current_dir().unwrap();
    let db = Db::new(work_dir.clone()).unwrap();
    
    let w = Worker::new(req.task_id.clone(), req.agent_name.clone(), work_dir, req.engine.clone());
    if let Ok(_) = w.spawn() {
        let _ = db.log_audit(&req.agent_name, "task_started", &req.task_id, "success");
        let _ = db.conn.execute("UPDATE tasks SET assignee = ?1, status = 'in_progress', engine = ?2 WHERE id = ?3", rusqlite::params![req.agent_name, req.engine, req.task_id]);
        Json(serde_json::json!({"status": "success"}))
    } else {
        Json(serde_json::json!({"status": "error"}))
    }
}

async fn done_task(Path(task_id): Path<String>) -> Json<serde_json::Value> {
    let work_dir = env::current_dir().unwrap();
    let db = Db::new(work_dir.clone()).unwrap();
    
    let mut stmt = db.conn.prepare("SELECT assignee FROM tasks WHERE id = ?1").unwrap();
    let assignee: Option<String> = stmt.query_row(rusqlite::params![task_id], |row| row.get(0)).unwrap_or(None);
    
    if let Some(name) = assignee {
        let _ = Worker::nuke(&name, &work_dir);
    }
    let _ = db.conn.execute("UPDATE tasks SET status = 'closed' WHERE id = ?1", rusqlite::params![task_id]);
    let _ = db.log_audit("web", "task_closed", &task_id, "success");
    
    Json(serde_json::json!({"status": "success"}))
}

async fn nudge_agent(Json(req): Json<NudgeRequest>) -> Json<serde_json::Value> {
    let work_dir = env::current_dir().unwrap();
    let db = Db::new(work_dir).unwrap();
    
    if Tmux::has_session(&req.agent_name) {
        let _ = Tmux::display_message(&req.agent_name, &format!("!!! NUDGE: {} !!!", req.message));
        let _ = db.log_audit("web", "nudge_sent", &req.agent_name, "success");
    } else {
        let _ = db.send_mail("web", &req.agent_name, "NUDGE: Web Action", &req.message);
    }
    Json(serde_json::json!({"status": "success"}))
}

async fn get_prompt(Path(role): Path<String>) -> Json<serde_json::Value> {
    let work_dir = env::current_dir().unwrap();
    let path = work_dir.join("prompts").join(format!("{}.md", role));
    let content = fs::read_to_string(path).unwrap_or_else(|_| "Prompt not found.".to_string());
    Json(serde_json::json!({"content": content}))
}

async fn list_agent_files(Path(agent_name): Path<String>) -> Json<serde_json::Value> {
    let work_dir = env::current_dir().unwrap();
    let agent_path = work_dir.join("workers").join(&agent_name);
    let mut files = Vec::new();
    
    if agent_path.exists() {
        if let Ok(entries) = fs::read_dir(agent_path) {
            for entry in entries.flatten() {
                if let Ok(name) = entry.file_name().into_string() {
                    if name != ".git" && name != ".DS_Store" {
                        files.push(name);
                    }
                }
            }
        }
    }
    Json(serde_json::json!({"files": files}))
}

async fn get_task_history(Path(task_id): Path<String>) -> Json<serde_json::Value> {
    let work_dir = env::current_dir().unwrap();
    let db = Db::new(work_dir).unwrap();
    
    // Search for logs where target is task_id OR actor is the task's assignee
    let mut stmt = db.conn.prepare("SELECT timestamp, actor, action, target, status FROM audit_logs WHERE target = ?1 OR actor IN (SELECT assignee FROM tasks WHERE id = ?1) ORDER BY timestamp DESC").unwrap();
    
    let history = stmt.query_map([&task_id], |row| {
        Ok(serde_json::json!({
            "timestamp": row.get::<_, i64>(0)?,
            "actor": row.get::<_, String>(1)?,
            "action": row.get::<_, String>(2)?,
            "target": row.get::<_, String>(3)?,
            "status": row.get::<_, String>(4)?,
        }))
    }).unwrap().map(|r| r.unwrap()).collect::<Vec<_>>();

    Json(serde_json::json!({"history": history}))
}

async fn get_agent_logs(Path((task_id, agent_name)): Path<(String, String)>) -> Json<AgentLogResponse> {
    let work_dir = env::current_dir().unwrap();
    // Path: .logs/tasks/<task_id>/<agent_name>.log
    let log_path = work_dir.join(".logs").join("tasks").join(&task_id).join(format!("{}.log", agent_name));
    
    let content = if log_path.exists() {
        fs::read_to_string(&log_path).unwrap_or_else(|_| "Error reading log file.".to_string())
    } else {
        format!("Log file not found at: {:?}", log_path)
    };

    Json(AgentLogResponse {
        content,
        path: log_path.to_string_lossy().to_string(),
    })
}

async fn get_dashboard() -> Json<DashboardData> {
    let work_dir = env::current_dir().unwrap();
    let db = Db::new(work_dir).unwrap();

    // 1. Get Tasks (Make engine field optional to handle legacy data)
    let mut stmt = db.conn.prepare("SELECT id, title, status, assignee, engine FROM tasks").unwrap();
    let tasks = stmt.query_map([], |row| {
        Ok(TaskData {
            id: row.get(0)?,
            title: row.get(1)?,
            status: row.get(2)?,
            assignee: row.get(3)?,
            engine: row.get(4).ok(),
        })
    }).unwrap().map(|r| r.unwrap()).collect::<Vec<_>>();

    // 2. Get Recent Logs
    let mut stmt = db.conn.prepare("SELECT timestamp, actor, action, target FROM audit_logs ORDER BY timestamp DESC LIMIT 20").unwrap();
    let logs = stmt.query_map([], |row| {
        Ok(LogData {
            timestamp: row.get(0)?,
            actor: row.get(1)?,
            action: row.get(2)?,
            target: row.get(3)?,
        })
    }).unwrap().map(|r| r.unwrap()).collect::<Vec<_>>();

    // 3. Get Active Agents (from tasks in progress)
    let agents = tasks.iter()
        .filter(|t| t.status == "in_progress")
        .filter_map(|t| t.assignee.clone())
        .collect::<Vec<_>>();

    // 4. Get Stats
    let mut stmt = db.conn.prepare("SELECT SUM(cost_usd) FROM costs").unwrap();
    let total_cost: f64 = stmt.query_row([], |row| row.get(0)).unwrap_or(0.0);
    
    let tasks_total = tasks.len() as i64;
    let tasks_done = tasks.iter().filter(|t| t.status == "closed").count() as i64;

    Json(DashboardData {
        tasks,
        agents,
        recent_logs: logs,
        stats: StatsData { total_cost, tasks_done, tasks_total },
    })
}
