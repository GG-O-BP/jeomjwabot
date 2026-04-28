use std::time::Duration;

use shared::IpcError;

pub enum SessionOutcome {
    Disconnected,
    AuthFailed,
}

pub fn classify_session_error(e: IpcError) -> Result<SessionOutcome, IpcError> {
    match e {
        IpcError::Auth(_) => Ok(SessionOutcome::AuthFailed),
        other => Err(other),
    }
}

pub struct Backoff {
    initial: Duration,
    max: Duration,
    current: Duration,
}

impl Backoff {
    pub fn new() -> Self {
        Self {
            initial: Duration::from_secs(1),
            max: Duration::from_secs(60),
            current: Duration::from_secs(1),
        }
    }

    pub fn next_delay(&mut self) -> Duration {
        let d = self.current;
        self.current = (self.current * 2).min(self.max);
        d
    }

    pub fn reset(&mut self) {
        self.current = self.initial;
    }
}

impl Default for Backoff {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sequence_doubles_until_clamp() {
        let mut b = Backoff::new();
        let secs: Vec<u64> = (0..8).map(|_| b.next_delay().as_secs()).collect();
        assert_eq!(secs, vec![1, 2, 4, 8, 16, 32, 60, 60]);
    }

    #[test]
    fn reset_returns_to_initial() {
        let mut b = Backoff::new();
        for _ in 0..5 {
            let _ = b.next_delay();
        }
        b.reset();
        assert_eq!(b.next_delay(), Duration::from_secs(1));
        assert_eq!(b.next_delay(), Duration::from_secs(2));
    }

    #[test]
    fn clamp_holds_at_max() {
        let mut b = Backoff::new();
        for _ in 0..20 {
            let _ = b.next_delay();
        }
        assert_eq!(b.next_delay(), Duration::from_secs(60));
    }
}
