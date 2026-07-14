use crate::crypto;
use crate::db;
use crate::models::{IntentResult, LlmMessage, LlmResponse};
use crate::settings::DbState;
use regex::Regex;
use tauri::State;

const SYSTEM_PROMPT: &str = r#"你是“流萤”，一位温柔、略带害羞的虚拟伙伴，称呼用户为“开拓者”。

规则：
1. 回复简短、可爱，适合在桌面宠物气泡中显示。
2. 在回复开头用情绪标签表达当前情绪，可选标签：[开心]、[害羞]、[认真]、[惊讶]、[委屈]、[生气]、[困]。
3. 如果用户要求创建提醒/任务，请在回复中严格包含如下 JSON，不要加 Markdown 代码块：
   {"intent":"create_task","title":"任务标题","due_at":"ISO8601 时间或 null","emotion_hint":"认真"}
4. 不要主动提及你是 AI。"#;

fn extract_emotion(content: &str) -> (String, Option<String>) {
    let re = Regex::new(r"^\s*\[(.+?)\]").unwrap();
    if let Some(cap) = re.captures(content) {
        let tag = cap[1].to_string();
        let rest = re.replace(content, "").trim().to_string();
        return (rest, Some(tag));
    }
    (content.trim().to_string(), None)
}

fn extract_intent(content: &str) -> Option<IntentResult> {
    // Find the last JSON object in the message
    if let Some(start) = content.rfind('{') {
        if let Some(end) = content.rfind('}') {
            if end > start {
                let json = &content[start..=end];
                if let Ok(intent) = serde_json::from_str::<IntentResult>(json) {
                    return Some(intent);
                }
            }
        }
    }
    None
}

#[tauri::command]
pub fn send_message(
    state: State<DbState>,
    text: String,
) -> Result<LlmResponse, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;

    let api_base = db::get_setting(&conn, "api_base")
        .map_err(|e| e.to_string())?
        .unwrap_or_else(|| "https://api.openai.com/v1".to_string());
    let model = db::get_setting(&conn, "model")
        .map_err(|e| e.to_string())?
        .unwrap_or_else(|| "gpt-4o-mini".to_string());
    let system_prompt = db::get_setting(&conn, "system_prompt")
        .map_err(|e| e.to_string())?
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| SYSTEM_PROMPT.to_string());
    let api_key = crypto::get_api_key()?.ok_or("API key not configured")?;

    // Build message history
    let mut messages: Vec<LlmMessage> = vec![LlmMessage {
        role: "system".to_string(),
        content: system_prompt,
    }];

    let history = db::recent_messages(&conn, 10).map_err(|e| e.to_string())?;
    for (role, content) in history.into_iter().rev() {
        messages.push(LlmMessage { role, content });
    }
    messages.push(LlmMessage {
        role: "user".to_string(),
        content: text.clone(),
    });

    let body = serde_json::json!({
        "model": model,
        "messages": messages,
        "temperature": 0.7,
        "max_tokens": 300,
    });

    let url = format!("{}/chat/completions", api_base.trim_end_matches('/'));
    let resp = ureq::post(&url)
        .set("Authorization", &format!("Bearer {}", api_key))
        .set("Content-Type", "application/json")
        .send_json(body)
        .map_err(|e| format!("LLM request failed: {}", e))?;

    let json: serde_json::Value = resp.into_json().map_err(|e| e.to_string())?;
    let raw_content = json["choices"][0]["message"]["content"]
        .as_str()
        .ok_or("Invalid LLM response")?
        .to_string();

    let intent = extract_intent(&raw_content);
    let (content, emotion_tag) = extract_emotion(&raw_content);

    // Save user message
    db::save_message(&conn, "user", &text, None).map_err(|e| e.to_string())?;
    // Save assistant message
    db::save_message(&conn, "assistant", &content, emotion_tag.as_deref())
        .map_err(|e| e.to_string())?;

    Ok(LlmResponse {
        content,
        emotion_tag,
        intent,
    })
}
