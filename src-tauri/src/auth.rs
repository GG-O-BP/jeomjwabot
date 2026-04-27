use serde::Deserialize;
use shared::{ChzzkAuth, CimeAuth, IpcError};

const CHZZK_BASE: &str = "https://openapi.chzzk.naver.com";
const CIME_BASE: &str = "https://api.cime.kr";

fn net_err(e: impl std::fmt::Display) -> IpcError {
    IpcError::Network(e.to_string())
}

fn proto_err(e: impl std::fmt::Display) -> IpcError {
    IpcError::Protocol(e.to_string())
}

/// Chzzk Open API의 응답 envelope. content 안에 url 또는 다른 필드가 들어온다.
#[derive(Debug, Deserialize)]
struct ChzzkEnvelope<T> {
    code: i32,
    message: Option<String>,
    content: Option<T>,
}

#[derive(Debug, Deserialize)]
struct UrlField {
    url: String,
}

pub async fn fetch_chzzk_session_url(auth: &ChzzkAuth) -> Result<String, IpcError> {
    let client = reqwest::Client::new();
    let req = if let Some(token) = auth.access_token.as_deref() {
        client
            .get(format!("{CHZZK_BASE}/open/v1/sessions/auth"))
            .bearer_auth(token)
    } else {
        client
            .get(format!("{CHZZK_BASE}/open/v1/sessions/auth/client"))
            .header("Client-Id", &auth.client_id)
            .header("Client-Secret", &auth.client_secret)
    };

    let resp = req.send().await.map_err(net_err)?;
    let status = resp.status();
    let text = resp.text().await.map_err(net_err)?;

    if !status.is_success() {
        return Err(IpcError::Auth(format!("치지직 세션 응답 {status}: {text}")));
    }

    if let Ok(env) = serde_json::from_str::<ChzzkEnvelope<UrlField>>(&text) {
        if env.code != 200 {
            return Err(IpcError::Auth(
                env.message.unwrap_or_else(|| "치지직 세션 거부".into()),
            ));
        }
        if let Some(c) = env.content {
            return Ok(c.url);
        }
    }
    if let Ok(direct) = serde_json::from_str::<UrlField>(&text) {
        return Ok(direct.url);
    }
    Err(proto_err(format!("치지직 응답 파싱 실패: {text}")))
}

pub async fn fetch_cime_session_url(auth: &CimeAuth) -> Result<String, IpcError> {
    let client = reqwest::Client::new();
    let resp = client
        .get(format!("{CIME_BASE}/api/openapi/open/v1/sessions/auth"))
        .bearer_auth(&auth.access_token)
        .send()
        .await
        .map_err(net_err)?;
    let status = resp.status();
    let text = resp.text().await.map_err(net_err)?;
    if !status.is_success() {
        return Err(IpcError::Auth(format!("씨미 세션 응답 {status}: {text}")));
    }
    let parsed: UrlField =
        serde_json::from_str(&text).map_err(|e| proto_err(format!("{e}: {text}")))?;
    Ok(parsed.url)
}

pub async fn subscribe_chzzk_event(
    auth: &ChzzkAuth,
    session_key: &str,
    event: &str,
) -> Result<(), IpcError> {
    let client = reqwest::Client::new();
    let path = format!("{CHZZK_BASE}/open/v1/sessions/events/subscribe/{event}");
    let req = if let Some(token) = auth.access_token.as_deref() {
        client.post(&path).bearer_auth(token)
    } else {
        client
            .post(&path)
            .header("Client-Id", &auth.client_id)
            .header("Client-Secret", &auth.client_secret)
    };
    let resp = req
        .query(&[("sessionKey", session_key)])
        .send()
        .await
        .map_err(net_err)?;
    if !resp.status().is_success() {
        let s = resp.status();
        let t = resp.text().await.unwrap_or_default();
        return Err(IpcError::Auth(format!("치지직 구독 {event} 응답 {s}: {t}")));
    }
    Ok(())
}

pub async fn subscribe_cime_event(
    auth: &CimeAuth,
    session_key: &str,
    event: &str,
) -> Result<(), IpcError> {
    let client = reqwest::Client::new();
    let resp = client
        .post(format!(
            "{CIME_BASE}/api/openapi/open/v1/sessions/events/subscribe/{event}"
        ))
        .bearer_auth(&auth.access_token)
        .query(&[("sessionKey", session_key)])
        .send()
        .await
        .map_err(net_err)?;
    if !resp.status().is_success() {
        let s = resp.status();
        let t = resp.text().await.unwrap_or_default();
        return Err(IpcError::Auth(format!("씨미 구독 {event} 응답 {s}: {t}")));
    }
    Ok(())
}
