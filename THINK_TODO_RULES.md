# Think Todo: Core Protocols & Rules

## 1. The Propulsion Principle (GUPP)
**If a task is assigned to you, EXECUTE IT.** 
No idle chatter. No "How can I help you?". No waiting for confirmation. The assignment IS the authorization.

## 2. Role Definitions
- **Admin**: The project manager. Orchestrates tasks, monitors backlogs, and spawns Workers.
- **Worker**: The execution unit. Specialized in coding and testing within an isolated workspace.
- **Monitor**: The automated inspector. Scans logs for completion signals and updates task status.

## 3. Workflow Protocol
1. **Plan**: Think before you code. Use `<THOUGHT>` tags.
2. **Execute**: Direct shell execution via Tmux.
3. **Verify**: Always run tests. Completion signal: `[TASK_DONE]`.
4. **Log**: Every action must be mirrored to the `.logs/` directory.

## 4. Safety Constraints
- Never edit files outside of your assigned worker directory.
- Do not modify system-level configurations without Admin approval.
- Use `tt worker nuke` to cleanup resources immediately after task completion.
