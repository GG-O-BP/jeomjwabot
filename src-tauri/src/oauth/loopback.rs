//! 한 번만 받는 로컬 HTTP 콜백 서버.
//!
//! 시각장애 사용자 동선:
//!   1. 백엔드가 시스템 브라우저로 OAuth 인증 페이지를 연다.
//!   2. 사용자가 브라우저에서 승인.
//!   3. 인증 서버가 redirect_uri(이 서버)로 `?code=...` 리다이렉트.
//!   4. 본 모듈이 단일 GET 요청을 받아 query 파싱 → 응답으로 한국어 안내 HTML 반환.
//!   5. 백엔드가 점좌봇 윈도우를 자동 포커스 — 화면리더가 즉시 다음 안내를 읽는다.
//!
//! 외부 redirect URI 호스트(127.0.0.1)·포트는 [`shared::CIME_REDIRECT_URI`]와 일치해야 한다.

use std::net::SocketAddr;
use std::time::Duration;

use shared::IpcError;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

#[derive(Debug, Default, Clone)]
pub struct CallbackParams {
    pub code: Option<String>,
    pub state: Option<String>,
    pub error: Option<String>,
    pub error_description: Option<String>,
}

pub struct LoopbackServer {
    listener: TcpListener,
}

impl LoopbackServer {
    /// 콜백 수신 전용 리스너를 미리 바인딩한다. 브라우저 오픈 *전에* 호출해
    /// 포트 충돌을 빨리 감지한다.
    pub async fn bind(addr: SocketAddr) -> Result<Self, IpcError> {
        let listener = TcpListener::bind(addr).await.map_err(|e| {
            IpcError::Internal(format!(
                "로컬 콜백 포트 {addr} 바인딩 실패 (다른 프로세스 사용 중?): {e}"
            ))
        })?;
        Ok(Self { listener })
    }

    /// 다음 GET 요청 1건을 받아 query 파라미터를 파싱하고, 사용자에게 보여줄
    /// 한국어 안내 페이지를 응답한다. 그 후 소켓을 닫는다.
    pub async fn accept_one(self, timeout: Duration) -> Result<CallbackParams, IpcError> {
        let (mut socket, _) = tokio::time::timeout(timeout, self.listener.accept())
            .await
            .map_err(|_| IpcError::Auth("OAuth 콜백 대기 시간이 초과되었습니다.".into()))?
            .map_err(|e| IpcError::Internal(format!("로컬 콜백 accept 실패: {e}")))?;

        // HTTP 요청을 헤더 끝(\r\n\r\n)까지 읽는다. 본문은 GET이라 보지 않는다.
        let mut buf = vec![0u8; 4096];
        let mut total = 0usize;
        while total < buf.len() {
            let n =
                match tokio::time::timeout(Duration::from_secs(5), socket.read(&mut buf[total..]))
                    .await
                {
                    Ok(Ok(n)) => n,
                    Ok(Err(e)) => {
                        return Err(IpcError::Network(format!("콜백 소켓 읽기 실패: {e}")));
                    }
                    Err(_) => break,
                };
            if n == 0 {
                break;
            }
            total += n;
            if buf[..total].windows(4).any(|w| w == b"\r\n\r\n") {
                break;
            }
        }
        let header_text = std::str::from_utf8(&buf[..total]).unwrap_or("");
        let request_line = header_text.lines().next().unwrap_or("");
        let path_and_query = request_line.split_whitespace().nth(1).unwrap_or("/");

        let params = parse_query(path_and_query);

        let body = response_html(&params);
        let response = format!(
            "HTTP/1.1 200 OK\r\n\
             Content-Type: text/html; charset=utf-8\r\n\
             Content-Length: {}\r\n\
             Cache-Control: no-store\r\n\
             Connection: close\r\n\
             \r\n{}",
            body.len(),
            body
        );
        let _ = socket.write_all(response.as_bytes()).await;
        let _ = socket.shutdown().await;

        Ok(params)
    }
}

fn parse_query(path_and_query: &str) -> CallbackParams {
    let mut params = CallbackParams::default();
    let url = match url::Url::parse(&format!("http://127.0.0.1{path_and_query}")) {
        Ok(u) => u,
        Err(_) => return params,
    };
    for (k, v) in url.query_pairs() {
        match k.as_ref() {
            "code" => params.code = Some(v.into_owned()),
            "state" => params.state = Some(v.into_owned()),
            "error" => params.error = Some(v.into_owned()),
            "error_description" => params.error_description = Some(v.into_owned()),
            _ => {}
        }
    }
    params
}

/// 콜백 후 브라우저 탭에 표시되는 페이지. 화면리더가 즉시 의미를 읽도록
/// 단일 h1 + 짧은 한국어 문장 + lang 속성. 시각 장식 없음.
fn response_html(params: &CallbackParams) -> String {
    let (title, body_p) = if params.error.is_some() {
        let detail = params.error_description.as_deref().unwrap_or("");
        let err = params.error.as_deref().unwrap_or("");
        (
            "씨미 인증이 거부되었습니다",
            if detail.is_empty() {
                format!("오류: {err}. 점좌봇 창으로 돌아가 다시 시도해주세요.")
            } else {
                format!("오류: {err}. {detail}. 점좌봇 창으로 돌아가 다시 시도해주세요.")
            },
        )
    } else if params.code.is_some() {
        (
            "씨미 인증이 완료되었습니다",
            "이 창은 닫으셔도 됩니다. 점좌봇 창으로 돌아가 진행 상태를 확인해주세요.".to_string(),
        )
    } else {
        (
            "잘못된 콜백입니다",
            "code 파라미터가 없습니다. 점좌봇 창으로 돌아가 다시 시도해주세요.".to_string(),
        )
    };
    format!(
        "<!doctype html>\n\
         <html lang=\"ko\"><head><meta charset=\"utf-8\">\n\
         <title>{title}</title>\n\
         </head><body>\n\
         <main>\n\
         <h1>{title}</h1>\n\
         <p>{body_p}</p>\n\
         </main>\n\
         </body></html>",
    )
}
