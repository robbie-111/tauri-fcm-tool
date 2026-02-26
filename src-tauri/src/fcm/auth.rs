use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use specta::Type;
use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;
use std::sync::mpsc;
use std::thread;

use super::exchange::{exchange_code_via_external_api, ExchangeCodeResponse};
use super::pkce::{generate_code_challenge, generate_code_verifier, generate_state};

const FCM_SCOPE: &str = "https://www.googleapis.com/auth/firebase.messaging";
const GOOGLE_AUTH_URL: &str = "https://accounts.google.com/o/oauth2/v2/auth";

/// OAuth 토큰 정보
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct OAuthToken {
    pub access_token: String,
    #[serde(default)]
    pub refresh_token: String,
    #[serde(default)]
    pub id_token: String,
    pub token_type: String,
    pub expiry: DateTime<Utc>,
}

impl OAuthToken {
    /// 토큰이 유효한지 확인
    pub fn is_valid(&self) -> bool {
        !self.access_token.is_empty() && Utc::now() < self.expiry
    }

    /// ExchangeCodeResponse로부터 OAuthToken 생성
    pub fn from_response(resp: ExchangeCodeResponse) -> Self {
        let expiry = Utc::now() + Duration::seconds(resp.expires_in);
        Self {
            access_token: resp.access_token,
            refresh_token: resp.refresh_token,
            id_token: resp.id_token,
            token_type: resp.token_type,
            expiry,
        }
    }
}

/// OAuth 인증 결과
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct AuthResult {
    pub success: bool,
    pub message: String,
    #[serde(default)]
    pub token: Option<OAuthToken>,
}

impl AuthResult {
    pub fn success(token: OAuthToken) -> Self {
        Self {
            success: true,
            message: "인증 성공".to_string(),
            token: Some(token),
        }
    }

    pub fn failure(message: String) -> Self {
        Self {
            success: false,
            message,
            token: None,
        }
    }
}

/// OAuth 인증 URL 생성
pub fn build_auth_url(
    client_id: &str,
    redirect_url: &str,
    state: &str,
    code_challenge: &str,
) -> String {
    format!(
        "{}?client_id={}&redirect_uri={}&response_type=code&scope={}&state={}&code_challenge={}&code_challenge_method=S256&access_type=offline&prompt=consent",
        GOOGLE_AUTH_URL,
        urlencoding::encode(client_id),
        urlencoding::encode(redirect_url),
        urlencoding::encode(FCM_SCOPE),
        urlencoding::encode(state),
        urlencoding::encode(code_challenge),
    )
}

/// OAuth 콜백 서버 시작 및 인증 코드 수신
pub fn start_oauth_callback_server(expected_state: &str) -> Result<String, String> {
    let listener =
        TcpListener::bind("127.0.0.1:8080").map_err(|e| format!("콜백 서버 바인딩 실패: {}", e))?;

    listener
        .set_nonblocking(false)
        .map_err(|e| format!("서버 설정 실패: {}", e))?;

    let (tx, rx) = mpsc::channel();
    let state = expected_state.to_string();

    thread::spawn(move || {
        if let Ok((mut stream, _)) = listener.accept() {
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            let mut request_line = String::new();

            if reader.read_line(&mut request_line).is_ok() {
                // GET /callback?code=xxx&state=yyy HTTP/1.1
                if let Some(params) = request_line.split('?').nth(1) {
                    let params: Vec<&str> = params
                        .split_whitespace()
                        .next()
                        .unwrap_or("")
                        .split('&')
                        .collect();

                    let mut code = None;
                    let mut received_state = None;
                    let mut error = None;

                    for param in params {
                        let kv: Vec<&str> = param.split('=').collect();
                        if kv.len() == 2 {
                            match kv[0] {
                                "code" => code = Some(kv[1].to_string()),
                                "state" => received_state = Some(kv[1].to_string()),
                                "error" => error = Some(kv[1].to_string()),
                                _ => {}
                            }
                        }
                    }

                    let (response_body, result) = if let Some(err) = error {
                        (
                            format!(
                                r#"<html><head><meta charset="utf-8"></head><body style="font-family: Arial; text-align: center; padding: 50px;"><h1 style="color: #f44336;">인증 실패</h1><p>오류: {}</p></body></html>"#,
                                err
                            ),
                            Err(format!("OAuth 오류: {}", err)),
                        )
                    } else if received_state.as_ref() != Some(&state) {
                        (
                            r#"<html><head><meta charset="utf-8"></head><body style="font-family: Arial; text-align: center; padding: 50px;"><h1 style="color: #f44336;">인증 실패</h1><p>State 검증 실패 (보안 오류)</p></body></html>"#.to_string(),
                            Err("State 불일치".to_string()),
                        )
                    } else if let Some(auth_code) = code {
                        (
                            r#"<html><head><meta charset="utf-8"></head><body style="font-family: Arial; text-align: center; padding: 50px;"><h1 style="color: #4CAF50;">인증 성공!</h1><p>이 창을 닫고 애플리케이션으로 돌아가세요.</p></body></html>"#.to_string(),
                            Ok(auth_code),
                        )
                    } else {
                        (
                            r#"<html><head><meta charset="utf-8"></head><body style="font-family: Arial; text-align: center; padding: 50px;"><h1 style="color: #f44336;">인증 실패</h1><p>인증 코드가 없습니다.</p></body></html>"#.to_string(),
                            Err("인증 코드 없음".to_string()),
                        )
                    };

                    let response = format!(
                        "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                        response_body.len(),
                        response_body
                    );

                    let _ = stream.write_all(response.as_bytes());
                    let _ = tx.send(result);
                }
            }
        }
    });

    // 5분 타임아웃
    rx.recv_timeout(std::time::Duration::from_secs(300))
        .map_err(|_| "인증 타임아웃".to_string())?
}

/// OAuth 인증 전체 플로우 실행
pub async fn authenticate(
    client_id: &str,
    redirect_url: &str,
    exchange_code_url: &str,
) -> AuthResult {
    // 1. PKCE 파라미터 생성
    let code_verifier = generate_code_verifier();
    let code_challenge = generate_code_challenge(&code_verifier);
    let state = generate_state();

    // 2. 인증 URL 생성
    let auth_url = build_auth_url(client_id, redirect_url, &state, &code_challenge);

    // 3. 브라우저 열기
    if let Err(e) = open::that(&auth_url) {
        return AuthResult::failure(format!("브라우저 열기 실패: {}", e));
    }

    // 4. 콜백 서버 시작 및 인증 코드 수신
    let code = match start_oauth_callback_server(&state) {
        Ok(code) => code,
        Err(e) => return AuthResult::failure(e),
    };

    // 5. 토큰 교환
    match exchange_code_via_external_api(exchange_code_url, &code, redirect_url, &code_verifier)
        .await
    {
        Ok(response) => {
            let token = OAuthToken::from_response(response);
            AuthResult::success(token)
        }
        Err(e) => AuthResult::failure(format!("토큰 교환 실패: {}", e)),
    }
}
