use crate::tmux::Tmux;
use anyhow::Result;
use std::fs;
use std::path::PathBuf;

pub struct Mayor {
    pub session_name: String,
    pub work_dir: PathBuf,
}

impl Mayor {
    pub fn new(work_dir: PathBuf) -> Self {
        Self {
            session_name: "hq-mayor".to_string(),
            work_dir,
        }
    }

    /// 启动市长 (Gemini 引擎)
    pub fn start(&self) -> Result<()> {
        if Tmux::has_session(&self.session_name) {
            println!("Mayor session already running.");
            return Ok(());
        }

        // 1. 准备工作区
        let mayor_dir = self.work_dir.join("mayor");
        if !mayor_dir.exists() {
            fs::create_dir_all(&mayor_dir)?;
        }

        // 2. 生成启动提示词 (Beacon)
        let prompt = format!(
            "[GAS TOWN] mayor <- human • cold-start


            推进原则 (Propulsion Principle):

            1. 检查 Hook (`gt hook`)

            2. 检查 Mail (`gt mail inbox`)

            3. 有活直接干，没活就待命。"
        );

        // 3. 构建 Gemini 启动命令
        // 注意：这里我们使用 gemini --approval-mode yolo，不加 --no-color
        let cmd = format!(
            "cd {} && gemini --approval-mode yolo "{}"", 
            mayor_dir.display(),
            prompt
        );

        // 4. 在 Tmux 中启动
        println!("Starting Mayor (Gemini Engine)...");
        Tmux::new_session(&self.session_name, &cmd)?;
        
        println!("✅ Mayor started! Run 'gt mayor attach' to enter.");
        Ok(())
    }

    /// 进入市长办公室
    pub fn attach(&self) -> Result<()> {
        // Rust 这里我们要用 exec 替换当前进程，模拟 Go 的 syscall.Exec
        // 但简单起见，我们先用 Command 调用 tmux attach
        let status = std::process::Command::new("tmux")
            .args(&["attach-session", "-t", &self.session_name])
            .status()?;
            
        if !status.success() {
            anyhow::bail!("Failed to attach to mayor session");
        }
        Ok(())
    }
}
