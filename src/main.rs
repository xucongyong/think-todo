mod tmux;
mod admin;
mod worker;
mod db;
mod monitor;

use clap::{Parser, Subcommand};
use anyhow::Result;
use std::env;
use rusqlite::params;

#[derive(Parser)]
#[command(name = "tt")]
#[command(about = "Think Todo (tt) - AI Agent Orchestrator")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
    #[arg(long, global = true)]
    debug: bool,
}

#[derive(Subcommand)]
enum Commands {
    Admin { #[command(subcommand)] action: AdminCommands },
    Worker { #[command(subcommand)] action: WorkerCommands },
    Task { #[command(subcommand)] action: TaskCommands },
    Monitor { #[command(subcommand)] action: MonitorCommands },
    Mail { #[command(subcommand)] action: MailCommands },
    Rig { #[command(subcommand)] action: RigCommands },
    Beads { #[command(subcommand)] action: BeadsCommands },
    Costs { #[command(subcommand)] action: CostsCommands },
    Sling { task_id: String, agent_name: String },
    Handoff { #[command(subcommand)] action: HandoffCommands },
    Done { task_id: String },
    Peek { agent_name: String },
    Trail,
    Nudge { agent_name: String, message: String },
}

#[derive(Subcommand)]
enum AdminCommands { Start, Attach, Stop }

#[derive(Subcommand)]
enum WorkerCommands {
    Spawn { task_id: String, name: String },
    Nuke { name: String },
}

#[derive(Subcommand)]
enum TaskCommands {
    Add { id: String, title: String },
    List,
}

#[derive(Subcommand)]
enum MonitorCommands { Start }

#[derive(Subcommand)]
enum HandoffCommands { New, Status }

#[derive(Subcommand)]
enum MailCommands {
    Inbox,
    Send { receiver: String, #[arg(short, long)] subject: String, #[arg(short, long)] body: String },
    Read { id: i32 },
}

#[derive(Subcommand)]
enum RigCommands {
    List,
    Add { name: String, path: String, #[arg(short, long)] repo: Option<String> },
    Status { name: String },
}

#[derive(Subcommand)]
enum BeadsCommands {
    List,
}

#[derive(Subcommand)]
enum CostsCommands {
    List,
    Summary,
    Add { task_id: String, agent: String, model: String, input: i32, output: i32, cost: f64 },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    if cli.debug { env::set_var("RUST_LOG", "debug"); } else { env::set_var("RUST_LOG", "info"); }
    env_logger::init();
    let work_dir = env::current_dir()?;
    let database = db::Db::new(work_dir.clone())?;

    match cli.command {
        Commands::Admin { action } => {
            let a = admin::Admin::new(work_dir);
            match action {
                AdminCommands::Start => a.start()?,
                AdminCommands::Attach => a.attach()?,
                AdminCommands::Stop => tmux::Tmux::kill_session(&a.session_name)?,
            }
        }
        Commands::Worker { action } => match action {
            WorkerCommands::Spawn { task_id, name } => {
                // Fix: Clone name so we can use it for logging later
                let w = worker::Worker::new(task_id, name.clone(), work_dir);
                w.spawn()?;
                let _ = database.log_audit("user", "spawn", &name, "success");
            }
            WorkerCommands::Nuke { name } => worker::Worker::nuke(&name, &work_dir)?,
        },
        Commands::Task { action } => match action {
            TaskCommands::Add { id, title } => {
                database.add_task(&id, &title)?;
                println!("✅ Task [{}] registered.", id);
            }
            TaskCommands::List => {
                let mut stmt = database.conn.prepare("SELECT id, title, status FROM tasks")?;
                let rows = stmt.query_map([], |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?, row.get::<_, String>(2)?)))?;
                println!("THINK TODO BACKLOG:");
                for r in rows { let (id, title, status) = r?; println!("- [{}] {} ({})", id, title, status); }
            }
        },
        Commands::Monitor { action } => match action {
            MonitorCommands::Start => {
                let m = monitor::Monitor::new(work_dir);
                m.watch()?;
            }
        },
        Commands::Mail { action } => match action {
            MailCommands::Inbox => {
                let mut stmt = database.conn.prepare("SELECT id, sender, subject, status FROM messages ORDER BY timestamp DESC")?;
                let rows = stmt.query_map([], |row| Ok((row.get::<_, i32>(0)?, row.get::<_, String>(1)?, row.get::<_, String>(2)?, row.get::<_, String>(3)?)))?;
                println!("📬 MAIL INBOX:");
                for r in rows {
                    let (id, sender, subject, status) = r?;
                    let marker = if status == "unread" { "●" } else { " " };
                    println!("{} [{}] From: {} | Subject: {}", marker, id, sender, subject);
                }
            }
            MailCommands::Send { receiver, subject, body } => {
                database.send_mail("user", &receiver, &subject, &body)?;
                println!("🚀 Mail sent to {}.", receiver);
            }
            MailCommands::Read { id } => {
                let mut stmt = database.conn.prepare("SELECT sender, subject, body, timestamp FROM messages WHERE id = ?1")?;
                let mut rows = stmt.query_map(params![id], |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?, row.get::<_, String>(2)?, row.get::<_, i64>(3)?)))?;
                if let Some(r) = rows.next() {
                    let (sender, subject, body, _ts) = r?;
                    println!("--- MAIL MESSAGE ---");
                    println!("From: {}", sender);
                    println!("Subject: {}", subject);
                    println!("\n{}", body);
                    println!("--------------------");
                    database.conn.execute("UPDATE messages SET status = 'read' WHERE id = ?1", params![id])?;
                } else {
                    println!("❌ Message not found.");
                }
            }
        },
        Commands::Rig { action } => match action {
            RigCommands::List => {
                let mut stmt = database.conn.prepare("SELECT name, path, status FROM rigs")?;
                let rows = stmt.query_map([], |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?, row.get::<_, String>(2)?)))?;
                println!("🏗️ REGISTERED RIGS:");
                for r in rows {
                    let (name, path, status) = r?;
                    println!("- {} [{}] ({})", name, path, status);
                }
            }
            RigCommands::Add { name, path, repo } => {
                database.add_rig(&name, &path, &repo.unwrap_or_default())?;
                println!("✅ Rig '{}' added.", name);
            }
            RigCommands::Status { name } => {
                let mut stmt = database.conn.prepare("SELECT path, repo, status, last_sync FROM rigs WHERE name = ?1")?;
                let mut rows = stmt.query_map(params![name], |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?, row.get::<_, String>(2)?, row.get::<_, i64>(3)?)))?;
                if let Some(r) = rows.next() {
                    let (path, repo, status, ts) = r?;
                    println!("RIG STATUS: {}", name);
                    println!("Path: {}", path);
                    println!("Repo: {}", repo);
                    println!("Status: {}", status);
                    println!("Last Sync: {}", ts);
                } else {
                    println!("❌ Rig not found.");
                }
            }
        },
        Commands::Beads { action } => match action {
            BeadsCommands::List => {
                println!("╔══════════════════════════════════════════════════════════════════════════╗");
                println!("║ 💠 THINK-TODO COCKPIT (SYSTEM PULSE)                                     ║");
                println!("╠══════════════════════════════════════════════════════════════════════════╣");

                // 1. Task Progress Summary
                let mut stmt = database.conn.prepare("SELECT status, COUNT(*) FROM tasks GROUP BY status")?;
                let rows = stmt.query_map([], |row| Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?)))?;
                let mut counts = std::collections::HashMap::new();
                for r in rows { let (s, c) = r?; counts.insert(s, c); }
                let open = counts.get("open").unwrap_or(&0);
                let in_p = counts.get("in_progress").unwrap_or(&0);
                let closed = counts.get("closed").unwrap_or(&0);
                let total = open + in_p + closed;
                let progress = if total > 0 { (closed as f64 / total as f64) * 100.0 } else { 0.0 };

                println!("  [TASKS] Progress: [{:<20}] {:.1}%", "=".repeat((progress/5.0) as usize), progress);
                println!("          Total: {} | ⏳ Open: {} | 🚀 Active: {} | ✅ Done: {}", total, open, in_p, closed);
                println!("╟──────────────────────────────────────────────────────────────────────────╢");

                // 2. Active Workers (Frontline)
                let mut stmt = database.conn.prepare("SELECT id, assignee FROM tasks WHERE status = 'in_progress'")?;
                let rows = stmt.query_map([], |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)))?;
                println!("  [FRONTLINE] Active Workers:");
                let mut active_any = false;
                for r in rows {
                    let (tid, agent) = r?;
                    println!("  → Agent '{}' is working on '{}'", agent, tid);
                    active_any = true;
                }
                if !active_any { println!("  (No active workers currently)"); }
                println!("╟──────────────────────────────────────────────────────────────────────────╢");

                // 3. Recent Activity (Trail)
                let mut stmt = database.conn.prepare("SELECT actor, action, target, timestamp FROM audit_logs ORDER BY timestamp DESC LIMIT 3")?;
                let rows = stmt.query_map([], |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?, row.get::<_, String>(2)?, row.get::<_, i64>(3)?)))?;
                println!("  [RECENT TRAIL]");
                for r in rows {
                    let (actor, action, target, _ts) = r?;
                    println!("  • {} {} {}", actor, action, target);
                }
                println!("╟──────────────────────────────────────────────────────────────────────────╢");

                // 4. Financial Status (Costs)
                let mut stmt = database.conn.prepare("SELECT SUM(cost_usd) FROM costs")?;
                let total_cost: f64 = stmt.query_row([], |row| row.get(0)).unwrap_or(0.0);
                println!("  [ECONOMY] Total System Cost: ${:.4}", total_cost);
                println!("╚══════════════════════════════════════════════════════════════════════════╝");
            }
        },
        Commands::Costs { action } => match action {
            CostsCommands::List => {
                let mut stmt = database.conn.prepare("SELECT task_id, agent_name, model, input_tokens, output_tokens, cost_usd, timestamp FROM costs ORDER BY timestamp DESC")?;
                let rows = stmt.query_map([], |row| Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, i32>(3)?,
                    row.get::<_, i32>(4)?,
                    row.get::<_, f64>(5)?,
                    row.get::<_, i64>(6)?
                )))?;
                println!("💸 DETAILED COSTS:");
                println!("{:<10} {:<15} {:<15} {:<10} {:<10} {:<10}", "TASK", "AGENT", "MODEL", "IN", "OUT", "COST($)");
                for r in rows {
                    let (task, agent, model, input, output, cost, _ts) = r?;
                    println!("{:<10} {:<15} {:<15} {:<10} {:<10} ${:<10.4}", task, agent, model, input, output, cost);
                }
            }
            CostsCommands::Summary => {
                let mut stmt = database.conn.prepare("SELECT model, SUM(input_tokens), SUM(output_tokens), SUM(cost_usd) FROM costs GROUP BY model")?;
                let rows = stmt.query_map([], |row| Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, i64>(2)?,
                    row.get::<_, f64>(3)?
                )))?;
                println!("📊 COST SUMMARY BY MODEL:");
                for r in rows {
                    let (model, input, output, cost) = r?;
                    println!("- {}: {} in / {} out | Total: ${:.4}", model, input, output, cost);
                }
            }
            CostsCommands::Add { task_id, agent, model, input, output, cost } => {
                database.log_cost(&task_id, &agent, &model, input, output, cost)?;
                println!("✅ Cost entry added for task {}.", task_id);
            }
        },
        Commands::Sling { task_id, agent_name } => {
            println!("🎯 SLING: Dispatching task '{}' to agent '{}'...", task_id, agent_name);
            let w = worker::Worker::new(task_id.clone(), agent_name.clone(), work_dir);
            w.spawn()?;
            database.log_audit(&agent_name, "sling_assigned", &task_id, "success")?;
            database.conn.execute("UPDATE tasks SET assignee = ?1, status = 'in_progress' WHERE id = ?2", params![agent_name, task_id])?;
            println!("🚀 Agent '{}' is now on the hook for '{}'.", agent_name, task_id);
        },
        Commands::Handoff { action } => match action {
            HandoffCommands::New => {
                println!("🤝 HANDOFF: Initiating session transfer...");
                println!("[HINT] Current session context saved. Run 'tt sling' with a new agent name to resume.");
            }
            HandoffCommands::Status => {
                println!("🔍 HANDOFF STATUS: No pending transfers.");
            }
        },
        Commands::Done { task_id } => {
            println!("🏁 DONE: Closing task '{}'...", task_id);
            // Find the assignee to nuke their dir
            let mut stmt = database.conn.prepare("SELECT assignee FROM tasks WHERE id = ?1")?;
            let mut rows = stmt.query_map(params![task_id], |row| row.get::<_, Option<String>>(0))?;
            if let Some(assignee) = rows.next() {
                if let Some(name) = assignee? {
                    println!("🧹 Cleaning up worker '{}'...", name);
                    let _ = worker::Worker::nuke(&name, &work_dir);
                }
            }
            database.conn.execute("UPDATE tasks SET status = 'closed' WHERE id = ?1", params![task_id])?;
            database.log_audit("user", "task_closed", &task_id, "success")?;
            println!("✅ Task '{}' is now marked as DONE and cleaned up.", task_id);
        },
        Commands::Peek { agent_name } => {
            println!("👀 PEEK: Viewing recent activity for agent '{}'...", agent_name);
            let mut stmt = database.conn.prepare("SELECT id FROM tasks WHERE assignee = ?1 AND status = 'in_progress'")?;
            let mut rows = stmt.query_map(params![agent_name], |row| row.get::<_, String>(0))?;
            if let Some(task_id) = rows.next() {
                let task_id = task_id?;
                let log_path = work_dir.join(".logs").join("tasks").join(&task_id).join(format!("{}.log", agent_name));
                if log_path.exists() {
                    let content = std::fs::read_to_string(&log_path)?;
                    let lines: Vec<&str> = content.lines().collect();
                    let last_lines = if lines.len() > 10 { &lines[lines.len()-10..] } else { &lines[..] };
                    println!("--- LOG TAIL (last 10 lines) ---");
                    for line in last_lines { println!("{}", line); }
                    println!("--------------------------------");
                } else {
                    println!("❌ Log file not found at {:?}", log_path);
                }
            } else {
                println!("❌ No active task found for agent '{}'.", agent_name);
            }
        },
        Commands::Trail => {
            println!("🛤️ TRAIL: Recent System Activity");
            let mut stmt = database.conn.prepare("SELECT actor, action, target, status, timestamp FROM audit_logs ORDER BY timestamp DESC LIMIT 15")?;
            let rows = stmt.query_map([], |row| Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, i64>(4)?
            )))?;
            for r in rows {
                let (actor, action, target, status, ts) = r?;
                println!("[{}] {} -> {} on {} ({})", ts, actor, action, target, status);
            }
        }
        Commands::Nudge { agent_name, message } => {
            println!("🔔 NUDGING agent '{}' with message: {}", agent_name, message);
            if tmux::Tmux::has_session(&agent_name) {
                tmux::Tmux::display_message(&agent_name, &format!("!!! NUDGE: {} !!!", message))?;
                database.log_audit("user", "nudge_sent", &agent_name, "success")?;
                println!("✅ Message displayed in agent's tmux session.");
            } else {
                println!("❌ Agent '{}' has no active tmux session. Logging to mail instead...", agent_name);
                database.send_mail("user", &agent_name, "NUDGE: Action Required", &message)?;
                database.log_audit("user", "nudge_mailed", &agent_name, "success")?;
                println!("✅ Nudge sent to agent's inbox.");
            }
        }
    }
    Ok(())
}
