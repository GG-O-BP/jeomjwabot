//! 씨미 OAuth Authorization Code Flow.
//!
//! 인증 URL 빌더 + Authorization Code → Access/Refresh Token 교환 + 토큰 갱신.
//! references/cime-authentication.html에 명시된 필드명·엔드포인트를 그대로 사용.

use chrono::{DateTime, Duration as ChronoDuration, Utc};
use serde::Deserialize;
use shared::{IpcError, CIME_REDIRECT_URI};

const AUTH_URL: &str = "https://ci.me/auth/openapi/account-interlock";
const TOKEN_URL: &str = "https://api.cime.kr/api/openapi/auth/v1/token";

#[derive(Debug, Clone)]
pub struct ExchangeOutcome {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub scope: Option<String>,
}

/// 사용자가 브라우저에서 열어야 할 인증 URL을 만든다.
/// scope는 OAuth 표준대로 공백 구분으로 직렬화하지만, 씨미 포털 설정만으로
/// 정해지는 경우에 대비해 인자가 비면 scope 파라미터를 생략한다.
pub fn build_auth_url(client_id: &str, state: &str, scopes: &[&str]) -> String {
    let mut url = url::Url::parse(AUTH_URL).expect("AUTH_URL 정적 상수 파싱 실패");
    {
        let mut q = url.query_pairs_mut();
        q.append_pair("clientId", client_id);
        q.append_pair("redirectUri", CIME_REDIRECT_URI);
        q.append_pair("state", state);
        if !scopes.is_empty() {
            q.append_pair("scope", &scopes.join(" "));
        }
    }
    url.into()
}

#[derive(Debug, Deserialize)]
struct CimeEnvelope<T> {
    code: i32,
    message: Option<String>,
    content: Option<T>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TokenContent {
    access_token: String,
    refresh_token: Option<String>,
    /// 씨미 문서 예시는 문자열("3600"). 둘 다 받게 string으로 받는다.
    expires_in: Option<String>,
    scope: Option<String>,
}

pub async fn exchange_code(
    client_id: &str,
    client_secret: &str,
    code: &str,
) -> Result<ExchangeOutcome, IpcError> {
    // RFC 6749 §4.1.3: 인증 요청에 redirect_uri를 보냈다면 토큰 교환에도
    // 동일 값을 보내야 한다. 씨미 docs 표에는 없지만 표준 준수 서버는 요구.
    let body = serde_json::json!({
        "grantType": "authorization_code",
        "clientId": client_id,
        "clientSecret": client_secret,
        "code": code,
        "redirectUri": CIME_REDIRECT_URI,
    });
    tracing::info!(
        client_id = %client_id,
        client_secret_len = client_secret.len(),
        code_len = code.len(),
        redirect_uri = CIME_REDIRECT_URI,
        "씨미 authorization_code 교환 요청"
    );
    post_token(body).await
}

pub async fn refresh_token(
    client_id: &str,
    client_secret: &str,
    refresh_token: &str,
) -> Result<ExchangeOutcome, IpcError> {
    let body = serde_json::json!({
        "grantType": "refresh_token",
        "clientId": client_id,
        "clientSecret": client_secret,
        "refreshToken": refresh_token,
    });
    tracing::info!(
        client_id = %client_id,
        client_secret_len = client_secret.len(),
        refresh_token_len = refresh_token.len(),
        "씨미 refresh_token 갱신 요청"
    );
    post_token(body).await
}

async fn post_token(body: serde_json::Value) -> Result<ExchangeOutcome, IpcError> {
    let resp = reqwest::Client::new()
        .post(TOKEN_URL)
        .header("User-Agent", "jeomjwabot/0.1")
        .json(&body)
        .send()
        .await
        .map_err(|e| IpcError::Network(format!("씨미 토큰 요청 실패: {e}")))?;
    let status = resp.status();
    let text = resp
        .text()
        .await
        .map_err(|e| IpcError::Network(format!("씨미 토큰 응답 본문 읽기 실패: {e}")))?;

    tracing::info!(status = %status, body = %text, "씨미 토큰 응답 수신");

    if !status.is_success() {
        return Err(IpcError::Auth(format!("씨미 토큰 응답 {status}: {text}")));
    }

    let env: CimeEnvelope<TokenContent> = serde_json::from_str(&text)
        .map_err(|e| IpcError::Protocol(format!("씨미 토큰 응답 파싱 실패: {e}: {text}")))?;
    if env.code != 200 {
        return Err(IpcError::Auth(
            env.message.unwrap_or_else(|| "씨미 토큰 발급 거부".into()),
        ));
    }
    let content = env
        .content
        .ok_or_else(|| IpcError::Protocol("씨미 토큰 응답이 비어있습니다".into()))?;

    let expires_at = content
        .expires_in
        .as_deref()
        .and_then(|s| s.parse::<i64>().ok())
        .map(|secs| Utc::now() + ChronoDuration::seconds(secs));

    Ok(ExchangeOutcome {
        access_token: content.access_token,
        refresh_token: content.refresh_token,
        expires_at,
        scope: content.scope,
    })
}
