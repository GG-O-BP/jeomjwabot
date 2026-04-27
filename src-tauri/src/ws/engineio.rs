use shared::IpcError;

/// Engine.IO v3 packet (raw text 위).
#[derive(Debug)]
pub enum EnginePacket<'a> {
    Open,
    Close,
    /// 서버가 보낸 ping payload — pong에 그대로 echo한다.
    Ping(&'a str),
    Pong,
    Message(SocketIoPacket<'a>),
    Upgrade,
    Noop,
}

/// Socket.IO v2 packet (Engine.IO Message payload 위).
#[derive(Debug)]
pub enum SocketIoPacket<'a> {
    Connect,
    Disconnect,
    /// `42[name, data...]` payload — bracket 포함 raw JSON 배열.
    Event(&'a str),
}

pub fn parse(raw: &str) -> Result<EnginePacket<'_>, IpcError> {
    let mut chars = raw.chars();
    let head = chars
        .next()
        .ok_or_else(|| IpcError::Protocol("빈 패킷".into()))?;
    let rest = &raw[head.len_utf8()..];
    match head {
        '0' => Ok(EnginePacket::Open),
        '1' => Ok(EnginePacket::Close),
        '2' => Ok(EnginePacket::Ping(rest)),
        '3' => Ok(EnginePacket::Pong),
        '4' => Ok(EnginePacket::Message(parse_socketio(rest)?)),
        '5' => Ok(EnginePacket::Upgrade),
        '6' => Ok(EnginePacket::Noop),
        _ => Err(IpcError::Protocol(format!(
            "알 수 없는 Engine.IO packet: {head}"
        ))),
    }
}

fn parse_socketio(raw: &str) -> Result<SocketIoPacket<'_>, IpcError> {
    let mut chars = raw.chars();
    let head = chars
        .next()
        .ok_or_else(|| IpcError::Protocol("빈 Socket.IO packet".into()))?;
    let rest = &raw[head.len_utf8()..];
    match head {
        '0' => Ok(SocketIoPacket::Connect),
        '1' => Ok(SocketIoPacket::Disconnect),
        '2' => Ok(SocketIoPacket::Event(rest)),
        _ => Err(IpcError::Protocol(format!(
            "알 수 없는 Socket.IO packet: {head}"
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_open() {
        let p = parse("0{\"sid\":\"abc\",\"pingInterval\":25000}").unwrap();
        assert!(matches!(p, EnginePacket::Open));
    }

    #[test]
    fn parses_ping_pong() {
        assert!(matches!(parse("2").unwrap(), EnginePacket::Ping(_)));
        assert!(matches!(parse("3").unwrap(), EnginePacket::Pong));
    }

    #[test]
    fn parses_event() {
        let p = parse("42[\"CHAT\",{\"content\":\"hi\"}]").unwrap();
        match p {
            EnginePacket::Message(SocketIoPacket::Event(payload)) => {
                assert!(payload.starts_with('['));
            }
            _ => panic!("expected event"),
        }
    }
}
