use serde::Deserialize;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::AsyncBufReadExt;
use tokio::process::Command;
use tokio::sync::{mpsc, Mutex};

use crate::db::{AiConfig, ChatMessage};

/// Claude CLI JSON 输出结构
#[derive(Debug, Deserialize)]
struct ClaudeOutput {
    result: String,
    #[allow(dead_code)]
    cost_usd: Option<f64>,
    #[allow(dead_code)]
    duration_ms: Option<u64>,
    #[allow(dead_code)]
    num_turns: Option<u32>,
    session_id: Option<String>,
    #[allow(dead_code)]
    model: Option<String>,
}

/// Claude CLI stream-json 输出结构
#[derive(Debug, Deserialize)]
struct ClaudeStreamOutput {
    #[allow(dead_code)]
    r#type: Option<String>,
    result: Option<String>,
    done: Option<bool>,
    session_id: Option<String>,
}

/// AI 客户端（通过 Claude Code CLI 子进程通信）
pub struct AiClient {
    config: Arc<Mutex<AiConfig>>,
    session_id: Arc<Mutex<Option<String>>>,
}

impl AiClient {
    /// 创建新的 AI 客户端
    pub fn new(config: AiConfig) -> Self {
        Self {
            config: Arc::new(Mutex::new(config)),
            session_id: Arc::new(Mutex::new(None)),
        }
    }

    /// 更新配置
    pub async fn update_config(&self, new_config: AiConfig) {
        let mut config = self.config.lock().await;
        *config = new_config;
        // 配置变更时重置会话
        *self.session_id.lock().await = None;
    }

    /// 获取配置
    pub async fn get_config(&self) -> AiConfig {
        self.config.lock().await.clone()
    }

    /// 构建 Claude CLI 的完整 prompt
    fn build_prompt(messages: &[ChatMessage]) -> String {
        let mut parts = Vec::new();
        for msg in messages {
            let role_label = match msg.role.as_str() {
                "system" => "[系统设定]",
                "user" => "[用户]",
                "assistant" => "[番茄猫]",
                _ => "[未知]",
            };
            parts.push(format!("{}\n{}", role_label, msg.content));
        }
        parts.join("\n\n")
    }

    /// 生成系统提示词（基于当前情绪和状态）
    pub fn generate_system_prompt(mood: &str, today_count: i32) -> String {
        let mood_desc = match mood {
            "happy" => "开心、活力充沛",
            "focused" => "专注、安静",
            "tired" => "疲惫、需要休息",
            "sleeping" => "困倦、想睡觉",
            "listening" => "好奇、等待主人说话",
            "thinking" => "认真思索中",
            _ => "平静",
        };

        format!(
            r#"你是「番茄猫」，用户桌面上的 AI 助手。
你的性格：友善、乐于助人、有点可爱。
你喜欢用简短温暖的语言回应用户，可以适当使用 emoji。

当前状态：
- 心情: {}
- 今日专注: {} 个番茄

核心能力：
1. 回答用户的各种问题
2. 帮助管理待办事项（用户可以说"帮我记个事"等）
3. 提供专注效率建议和提醒
4. 聊日常话题

行为指南：
1. 根据当前心情调整语气
2. 回复要简短（1-3句话）
3. 如果用户提到待办事项，主动询问是否需要创建任务
4. 适当使用 emoji 和颜文字表达情绪
5. 像个真正的助理一样关心用户的工作效率

当前心情：{} — 据此调整你的回应风格"#,
            mood_desc,
            today_count,
            mood_desc
        )
    }

    /// 发送聊天消息（非流式，通过 Claude CLI 子进程）
    pub async fn chat(&self, messages: Vec<ChatMessage>) -> Result<String, String> {
        let config = self.config.lock().await;
        let prompt = Self::build_prompt(&messages);

        let mut cmd = Command::new("claude");
        cmd.arg("-p");
        cmd.arg(&prompt);
        cmd.arg("--output-format");
        cmd.arg("json");
        cmd.arg("--bare");

        // Session 续传
        let sid = self.session_id.lock().await.clone();
        if let Some(ref sid) = sid {
            cmd.arg("--resume");
            cmd.arg(sid);
        }

        // Token 预算控制
        cmd.arg("--max-turns");
        cmd.arg(config.max_turns.to_string());

        if let Some(budget) = config.max_budget_usd {
            cmd.arg("--max-budget-usd");
            cmd.arg(budget.to_string());
        }

        drop(config); // 释放锁

        let output = tokio::time::timeout(Duration::from_secs(120), cmd.output())
            .await
            .map_err(|_| "Claude CLI 响应超时（120 秒），请检查网络或减少 max_turns".to_string())?
            .map_err(|e| {
                format!(
                    "Claude CLI 调用失败: {}（请确认已安装 Claude Code: npm install -g @anthropic-ai/claude-code）",
                    e
                )
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("Claude CLI 执行错误: {}", stderr));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let claude_output: ClaudeOutput = serde_json::from_str(&stdout)
            .map_err(|e| format!("解析 Claude 响应失败: {} (输出: {})", e, &stdout[..stdout.len().min(200)]))?;

        // 保存 session_id 用于后续对话续传
        if let Some(sid) = claude_output.session_id {
            *self.session_id.lock().await = Some(sid);
        }

        Ok(claude_output.result)
    }

    /// 发送聊天消息（流式，通过 Claude CLI 子进程）
    pub async fn chat_stream(
        &self,
        messages: Vec<ChatMessage>,
    ) -> Result<mpsc::Receiver<String>, String> {
        let config = self.config.lock().await;
        let prompt = Self::build_prompt(&messages);

        let mut cmd = Command::new("claude");
        cmd.arg("-p");
        cmd.arg(&prompt);
        cmd.arg("--output-format");
        cmd.arg("stream-json");
        cmd.arg("--bare");

        let sid = self.session_id.lock().await.clone();
        if let Some(ref sid) = sid {
            cmd.arg("--resume");
            cmd.arg(sid);
        }

        cmd.arg("--max-turns");
        cmd.arg(config.max_turns.to_string());

        if let Some(budget) = config.max_budget_usd {
            cmd.arg("--max-budget-usd");
            cmd.arg(budget.to_string());
        }

        drop(config);

        // 捕获 stdout 用于流式解析
        let mut child = cmd
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::inherit())
            .spawn()
            .map_err(|e| {
                format!(
                    "Claude CLI 启动失败: {}（请确认已安装 Claude Code）",
                    e
                )
            })?;

        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| "无法获取 Claude CLI stdout".to_string())?;

        let (tx, rx) = mpsc::channel(100);
        let session_id_clone = self.session_id.clone();

        tokio::spawn(async move {
            let reader = tokio::io::BufReader::new(stdout);
            let mut lines = reader.lines();
            let mut new_session_id: Option<String> = None;

            while let Ok(Some(line)) = lines.next_line().await {
                if line.is_empty() {
                    continue;
                }

                if let Ok(stream_output) = serde_json::from_str::<ClaudeStreamOutput>(&line) {
                    if let Some(content) = stream_output.result {
                        let _ = tx.send(content).await;
                    }
                    if stream_output.done == Some(true) {
                        new_session_id = stream_output.session_id;
                    }
                }
            }

            // 更新 session_id
            if let Some(sid) = new_session_id {
                *session_id_clone.lock().await = Some(sid);
            }

            // 等待子进程结束
            let _ = child.wait().await;
        });

        Ok(rx)
    }

    /// 清空对话会话
    pub async fn clear_session(&self) {
        *self.session_id.lock().await = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::ChatMessage;

    #[test]
    fn test_build_prompt_single_message() {
        let messages = vec![ChatMessage {
            id: 1,
            role: "user".to_string(),
            content: "你好".to_string(),
            created_at: "2026-06-16T08:00:00Z".to_string(),
        }];

        let prompt = AiClient::build_prompt(&messages);
        assert_eq!(prompt, "[用户]\n你好");
    }

    #[test]
    fn test_build_prompt_multiple_messages() {
        let messages = vec![
            ChatMessage {
                id: 1,
                role: "system".to_string(),
                content: "你是番茄猫".to_string(),
                created_at: "2026-06-16T08:00:00Z".to_string(),
            },
            ChatMessage {
                id: 2,
                role: "user".to_string(),
                content: "今天天气怎么样？".to_string(),
                created_at: "2026-06-16T08:01:00Z".to_string(),
            },
            ChatMessage {
                id: 3,
                role: "assistant".to_string(),
                content: "今天天气很好呢！".to_string(),
                created_at: "2026-06-16T08:02:00Z".to_string(),
            },
        ];

        let prompt = AiClient::build_prompt(&messages);
        assert!(prompt.contains("[系统设定]\n你是番茄猫"));
        assert!(prompt.contains("[用户]\n今天天气怎么样？"));
        assert!(prompt.contains("[番茄猫]\n今天天气很好呢！"));
    }

    #[test]
    fn test_build_prompt_unknown_role() {
        let messages = vec![ChatMessage {
            id: 1,
            role: "unknown".to_string(),
            content: "测试内容".to_string(),
            created_at: "2026-06-16T08:00:00Z".to_string(),
        }];

        let prompt = AiClient::build_prompt(&messages);
        assert_eq!(prompt, "[未知]\n测试内容");
    }

    #[test]
    fn test_generate_system_prompt_happy() {
        let prompt = AiClient::generate_system_prompt("happy", 5);
        assert!(prompt.contains("心情: 开心、活力充沛"));
        assert!(prompt.contains("今日专注: 5 个番茄"));
        assert!(prompt.contains("开心、活力充沛 — 据此调整你的回应风格"));
    }

    #[test]
    fn test_generate_system_prompt_focused() {
        let prompt = AiClient::generate_system_prompt("focused", 0);
        assert!(prompt.contains("心情: 专注、安静"));
        assert!(prompt.contains("今日专注: 0 个番茄"));
    }

    #[test]
    fn test_generate_system_prompt_tired() {
        let prompt = AiClient::generate_system_prompt("tired", 10);
        assert!(prompt.contains("心情: 疲惫、需要休息"));
    }

    #[test]
    fn test_generate_system_prompt_sleeping() {
        let prompt = AiClient::generate_system_prompt("sleeping", 3);
        assert!(prompt.contains("心情: 困倦、想睡觉"));
    }

    #[test]
    fn test_generate_system_prompt_listening() {
        let prompt = AiClient::generate_system_prompt("listening", 2);
        assert!(prompt.contains("心情: 好奇、等待主人说话"));
    }

    #[test]
    fn test_generate_system_prompt_thinking() {
        let prompt = AiClient::generate_system_prompt("thinking", 1);
        assert!(prompt.contains("心情: 认真思索中"));
    }

    #[test]
    fn test_generate_system_prompt_unknown_mood() {
        let prompt = AiClient::generate_system_prompt("custom_mood", 0);
        assert!(prompt.contains("心情: 平静"));
    }

    #[tokio::test]
    async fn test_new_client() {
        let config = AiConfig {
            id: 1,
            max_turns: 3,
            max_budget_usd: Some(0.1),
            updated_at: "2026-06-16".to_string(),
        };

        let client = AiClient::new(config.clone());
        let stored_config = client.get_config().await;

        assert_eq!(stored_config.max_turns, 3);
        assert_eq!(stored_config.max_budget_usd, Some(0.1));
    }

    #[tokio::test]
    async fn test_update_config() {
        let config = AiConfig {
            id: 1,
            max_turns: 3,
            max_budget_usd: Some(0.1),
            updated_at: "2026-06-16".to_string(),
        };

        let client = AiClient::new(config);

        // 设置一个 session_id
        *client.session_id.lock().await = Some("test_session".to_string());

        // 更新配置
        let new_config = AiConfig {
            id: 1,
            max_turns: 5,
            max_budget_usd: Some(0.2),
            updated_at: "2026-06-16".to_string(),
        };

        client.update_config(new_config).await;

        let stored_config = client.get_config().await;
        assert_eq!(stored_config.max_turns, 5);
        assert_eq!(stored_config.max_budget_usd, Some(0.2));

        // 验证 session_id 被重置
        let session_id = client.session_id.lock().await.clone();
        assert!(session_id.is_none());
    }

    #[tokio::test]
    async fn test_clear_session() {
        let config = AiConfig {
            id: 1,
            max_turns: 3,
            max_budget_usd: Some(0.1),
            updated_at: "2026-06-16".to_string(),
        };

        let client = AiClient::new(config);

        // 设置一个 session_id
        *client.session_id.lock().await = Some("test_session".to_string());
        assert!(client.session_id.lock().await.is_some());

        // 清空会话
        client.clear_session().await;
        assert!(client.session_id.lock().await.is_none());
    }
}
