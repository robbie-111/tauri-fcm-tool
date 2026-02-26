use serde::{Deserialize, Serialize};
use specta::Type;

/// FCM 앱 설정
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct FcmConfig {
    /// Google OAuth 2.0 클라이언트 ID
    pub oauth_client_id: String,
    /// OAuth 리다이렉트 URL
    pub oauth_redirect_url: String,
    /// 외부 토큰 교환 API URL
    pub exchange_code_url: String,
    /// Firebase 프로젝트 ID
    pub firebase_project_id: String,
}

impl Default for FcmConfig {
    fn default() -> Self {
        Self {
            oauth_client_id: String::new(),
            oauth_redirect_url: "http://localhost:8080/callback".to_string(),
            exchange_code_url: "https://percent-config.111percent.net/ExchangeAuthorizationCode"
                .to_string(),
            firebase_project_id: String::new(),
        }
    }
}

impl FcmConfig {
    /// 설정이 유효한지 확인
    pub fn is_valid(&self) -> bool {
        !self.oauth_client_id.is_empty()
            && !self.exchange_code_url.is_empty()
            && !self.firebase_project_id.is_empty()
    }
}
