use tauri::AppHandle;
use tauri_plugin_store::StoreExt;

use crate::fcm::{
    auth::{authenticate, AuthResult, OAuthToken},
    client::FcmClient,
    config::FcmConfig,
    history::{HistoryEntry, HistoryList},
    message::{SendRequest, SendResult},
    template::{Template, TemplateList},
};

const CONFIG_STORE: &str = "config.json";
const TOKEN_STORE: &str = "token.json";
const TEMPLATES_STORE: &str = "templates.json";
const HISTORY_STORE: &str = "history.json";

// ============================================================================
// 설정 관련 커맨드
// ============================================================================

#[tauri::command]
#[specta::specta]
pub async fn get_config(app: AppHandle) -> Result<FcmConfig, String> {
    let store = app
        .store(CONFIG_STORE)
        .map_err(|e| format!("스토어 열기 실패: {}", e))?;

    let config: FcmConfig = store
        .get("config")
        .and_then(|v| serde_json::from_value(v).ok())
        .unwrap_or_default();

    Ok(config)
}

#[tauri::command]
#[specta::specta]
pub async fn save_config(app: AppHandle, config: FcmConfig) -> Result<(), String> {
    let store = app
        .store(CONFIG_STORE)
        .map_err(|e| format!("스토어 열기 실패: {}", e))?;

    store.set(
        "config",
        serde_json::to_value(&config).map_err(|e| format!("직렬화 실패: {}", e))?,
    );

    store.save().map_err(|e| format!("저장 실패: {}", e))?;

    Ok(())
}

// ============================================================================
// 인증 관련 커맨드
// ============================================================================

#[tauri::command]
#[specta::specta]
pub async fn is_authenticated(app: AppHandle) -> Result<bool, String> {
    let store = app
        .store(TOKEN_STORE)
        .map_err(|e| format!("스토어 열기 실패: {}", e))?;

    let token: Option<OAuthToken> = store
        .get("token")
        .and_then(|v| serde_json::from_value(v).ok());

    Ok(token.map(|t| t.is_valid()).unwrap_or(false))
}

#[tauri::command]
#[specta::specta]
pub async fn get_token(app: AppHandle) -> Result<Option<OAuthToken>, String> {
    let store = app
        .store(TOKEN_STORE)
        .map_err(|e| format!("스토어 열기 실패: {}", e))?;

    let token: Option<OAuthToken> = store
        .get("token")
        .and_then(|v| serde_json::from_value(v).ok());

    Ok(token.filter(|t| t.is_valid()))
}

#[tauri::command]
#[specta::specta]
pub async fn start_oauth(app: AppHandle) -> Result<AuthResult, String> {
    // 1. 설정 가져오기
    let config = get_config(app.clone()).await?;

    if !config.is_valid() {
        return Ok(AuthResult::failure(
            "설정이 완료되지 않았습니다. 설정 탭에서 필수 항목을 입력해주세요.".to_string(),
        ));
    }

    // 2. OAuth 인증 실행
    let result = authenticate(
        &config.oauth_client_id,
        &config.oauth_redirect_url,
        &config.exchange_code_url,
    )
    .await;

    // 3. 성공 시 토큰 저장
    if result.success {
        if let Some(ref token) = result.token {
            let store = app
                .store(TOKEN_STORE)
                .map_err(|e| format!("스토어 열기 실패: {}", e))?;

            store.set(
                "token",
                serde_json::to_value(token).map_err(|e| format!("직렬화 실패: {}", e))?,
            );

            store.save().map_err(|e| format!("저장 실패: {}", e))?;
        }
    }

    Ok(result)
}

#[tauri::command]
#[specta::specta]
pub async fn logout(app: AppHandle) -> Result<(), String> {
    let store = app
        .store(TOKEN_STORE)
        .map_err(|e| format!("스토어 열기 실패: {}", e))?;

    store.delete("token");
    store.save().map_err(|e| format!("저장 실패: {}", e))?;

    Ok(())
}

// ============================================================================
// FCM 발송 커맨드
// ============================================================================

#[tauri::command]
#[specta::specta]
pub async fn send_fcm_message(app: AppHandle, request: SendRequest) -> Result<SendResult, String> {
    // 1. 토큰 확인
    let token = get_token(app.clone())
        .await?
        .ok_or_else(|| "인증되지 않았습니다. 먼저 로그인해주세요.".to_string())?;

    // 2. 설정 가져오기
    let config = get_config(app.clone()).await?;

    if config.firebase_project_id.is_empty() {
        return Err("Firebase 프로젝트 ID가 설정되지 않았습니다.".to_string());
    }

    // 3. FCM 클라이언트 생성 및 발송
    let client = FcmClient::new(&config.firebase_project_id, &token)?;
    let result = client.send(request.clone()).await;

    // 4. 히스토리 저장
    let message_type = match request.message_type {
        crate::fcm::message::MessageType::Single => "single",
        crate::fcm::message::MessageType::Topic => "topic",
    };

    let entry = HistoryEntry::new(
        message_type,
        &request.message.title,
        &request.message.body,
        result.success,
        &result.details,
    );

    // 히스토리 저장 에러는 로깅하되 발송 결과에는 영향 없음
    if let Err(e) = add_history_entry(app, entry).await {
        eprintln!("히스토리 저장 실패: {}", e);
    }

    Ok(result)
}

// ============================================================================
// 템플릿 관련 커맨드
// ============================================================================

#[tauri::command]
#[specta::specta]
pub async fn get_templates(app: AppHandle) -> Result<TemplateList, String> {
    let store = app
        .store(TEMPLATES_STORE)
        .map_err(|e| format!("스토어 열기 실패: {}", e))?;

    let templates: TemplateList = store
        .get("templates")
        .and_then(|v| serde_json::from_value(v).ok())
        .unwrap_or_default();

    Ok(templates)
}

#[tauri::command]
#[specta::specta]
pub async fn save_template(app: AppHandle, template: Template) -> Result<(), String> {
    let store = app
        .store(TEMPLATES_STORE)
        .map_err(|e| format!("스토어 열기 실패: {}", e))?;

    let mut templates: TemplateList = store
        .get("templates")
        .and_then(|v| serde_json::from_value(v).ok())
        .unwrap_or_default();

    templates.save(template);

    store.set(
        "templates",
        serde_json::to_value(&templates).map_err(|e| format!("직렬화 실패: {}", e))?,
    );

    store.save().map_err(|e| format!("저장 실패: {}", e))?;

    Ok(())
}

#[tauri::command]
#[specta::specta]
pub async fn delete_template(app: AppHandle, id: String) -> Result<bool, String> {
    let store = app
        .store(TEMPLATES_STORE)
        .map_err(|e| format!("스토어 열기 실패: {}", e))?;

    let mut templates: TemplateList = store
        .get("templates")
        .and_then(|v| serde_json::from_value(v).ok())
        .unwrap_or_default();

    let deleted = templates.delete(&id);

    store.set(
        "templates",
        serde_json::to_value(&templates).map_err(|e| format!("직렬화 실패: {}", e))?,
    );

    store.save().map_err(|e| format!("저장 실패: {}", e))?;

    Ok(deleted)
}

// ============================================================================
// 히스토리 관련 커맨드
// ============================================================================

#[tauri::command]
#[specta::specta]
pub async fn get_history(app: AppHandle) -> Result<HistoryList, String> {
    let store = app
        .store(HISTORY_STORE)
        .map_err(|e| format!("스토어 열기 실패: {}", e))?;

    let history: HistoryList = store
        .get("history")
        .and_then(|v| serde_json::from_value(v).ok())
        .unwrap_or_default();

    Ok(history)
}

async fn add_history_entry(app: AppHandle, entry: HistoryEntry) -> Result<(), String> {
    let store = app
        .store(HISTORY_STORE)
        .map_err(|e| format!("스토어 열기 실패: {}", e))?;

    let mut history: HistoryList = store
        .get("history")
        .and_then(|v| serde_json::from_value(v).ok())
        .unwrap_or_default();

    history.add(entry);

    store.set(
        "history",
        serde_json::to_value(&history).map_err(|e| format!("직렬화 실패: {}", e))?,
    );

    store.save().map_err(|e| format!("저장 실패: {}", e))?;

    Ok(())
}

#[tauri::command]
#[specta::specta]
pub async fn clear_history(app: AppHandle) -> Result<(), String> {
    let store = app
        .store(HISTORY_STORE)
        .map_err(|e| format!("스토어 열기 실패: {}", e))?;

    let mut history: HistoryList = store
        .get("history")
        .and_then(|v| serde_json::from_value(v).ok())
        .unwrap_or_default();

    history.clear();

    store.set(
        "history",
        serde_json::to_value(&history).map_err(|e| format!("직렬화 실패: {}", e))?,
    );

    store.save().map_err(|e| format!("저장 실패: {}", e))?;

    Ok(())
}
