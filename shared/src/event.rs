use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::platform::Platform;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatEvent {
    pub nickname: String,
    pub content: String,
    pub user_role: Option<UserRole>,
    pub verified: bool,
    pub message_time: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum UserRole {
    Streamer,
    Manager,
    ChatManager,
    Common,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DonationEvent {
    pub donator_nickname: Option<String>,
    pub amount: u64,
    pub message: String,
    pub donation_type: DonationType,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum DonationType {
    Chat,
    Video,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscriptionEvent {
    pub subscriber_nickname: String,
    pub tier_no: u8,
    pub month: u32,
    pub tier_name: Option<String>,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemEvent {
    pub kind: SystemKind,
    pub message: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SystemKind {
    Connected,
    Subscribed,
    Unsubscribed,
    Revoked,
    Disconnected,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum LiveEvent {
    Chat(ChatEvent),
    Donation(DonationEvent),
    Subscription(SubscriptionEvent),
    System(SystemEvent),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventEnvelope {
    pub id: String,
    pub platform: Platform,
    pub received_at: DateTime<Utc>,
    pub payload: LiveEvent,
}
