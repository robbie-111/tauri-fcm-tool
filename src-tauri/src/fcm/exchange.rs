use serde::{Deserialize, Serialize};
use std::time::Duration;

/// 외부 API로 토큰 교환 요청
#[derive(Debug, Serialize)]
pub struct ExchangeCodeRequest {
    pub code: String,
    pub redirect_uri: String,
    pub code_verifier: String,
}

/// 외부 API 토큰 교환 응답
#[derive(Debug, Deserialize)]
pub struct ExchangeCodeResponse {
    pub access_token: String,
    #[serde(default)]
    pub refresh_token: String,
    #[serde(default)]
    pub id_token: String,
    pub expires_in: i64,
    pub token_type: String,
}

/// 외부 API를 통해 authorization code를 token으로 교환
pub async fn exchange_code_via_external_api(
    api_url: &str,
    code: &str,
    redirect_uri: &str,
    code_verifier: &str,
) -> Result<ExchangeCodeResponse, String> {
    let payload = ExchangeCodeRequest {
        code: code.to_string(),
        redirect_uri: redirect_uri.to_string(),
        code_verifier: code_verifier.to_string(),
    };

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .map_err(|e| format!("HTTP 클라이언트 생성 실패: {}", e))?;

    let resp = client
        .post(api_url)
        .json(&payload)
        .send()
        .await
        .map_err(|e| format!("토큰 교환 요청 실패: {}", e))?;

    let status = resp.status();
    let body = resp
        .text()
        .await
        .map_err(|e| format!("응답 읽기 실패: {}", e))?;

    if !status.is_success() {
        return Err(format!(
            "토큰 교환 실패: HTTP {} | {}",
            status.as_u16(),
            body
        ));
    }

    let token_response: ExchangeCodeResponse = serde_json::from_str(&body)
        .map_err(|e| format!("응답 파싱 실패: {} | body: {}", e, body))?;

    if token_response.access_token.is_empty() {
        return Err("응답에 access_token이 없습니다".to_string());
    }

    Ok(token_response)
}
