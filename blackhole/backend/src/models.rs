use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

// ============================================================================
// CORE EMAIL MODELS
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Email {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub inbox_id: Option<Uuid>,
    pub message_id: String,
    pub in_reply_to: Option<String>,
    pub thread_id: Uuid,
    
    // Headers
    pub from_address: String,
    pub from_name: Option<String>,
    pub to_addresses: Vec<String>,
    pub cc_addresses: Option<Vec<String>>,
    pub bcc_addresses: Option<Vec<String>>,
    pub subject: String,
    
    // Content
    pub body_text: Option<String>,
    pub body_html: Option<String>,
    pub headers: serde_json::Value,
    
    // Metadata
    pub size_bytes: i64,
    pub has_attachments: bool,
    pub attachment_count: i32,
    
    // Lifecycle
    pub state: EmailState,
    pub assigned_to: Option<Uuid>,
    pub tags: Vec<String>,
    
    // SLA & Timing
    pub sla_deadline: Option<DateTime<Utc>>,
    pub first_response_at: Option<DateTime<Utc>>,
    pub resolved_at: Option<DateTime<Utc>>,
    
    // Timestamps
    pub received_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    
    // Observability
    pub ingest_latency_ms: Option<i32>,
    pub delivery_latency_ms: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq)]
#[sqlx(type_name = "email_state", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EmailState {
    New,
    Claimed,
    Responded,
    Resolved,
    Archived,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct EmailAttachment {
    pub id: Uuid,
    pub email_id: Uuid,
    pub filename: String,
    pub content_type: String,
    pub size_bytes: i64,
    pub storage_path: String,
    pub created_at: DateTime<Utc>,
}

// ============================================================================
// INBOX MODELS
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Inbox {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub name: String,
    pub email_address: String,
    pub description: Option<String>,
    pub is_shared: bool,
    pub color: Option<String>,
    pub icon: Option<String>,
    pub settings: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ============================================================================
// USER & TENANT MODELS
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub email: String,
    pub name: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub role: UserRole,
    pub is_active: bool,
    pub last_login_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "user_role", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum UserRole {
    Admin,
    Agent,
    Viewer,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Tenant {
    pub id: Uuid,
    pub name: String,
    pub domain: String,
    pub settings: serde_json::Value,
    pub storage_used_bytes: i64,
    pub storage_limit_bytes: i64,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ============================================================================
// RULES ENGINE MODELS
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Rule {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub priority: i32,
    pub is_active: bool,
    
    // Conditions (stored as JSON)
    pub conditions: serde_json::Value,
    
    // Actions (stored as JSON)
    pub actions: serde_json::Value,
    
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleConditions {
    pub from_contains: Option<Vec<String>>,
    pub to_contains: Option<Vec<String>>,
    pub subject_contains: Option<Vec<String>>,
    pub body_contains: Option<Vec<String>>,
    pub has_attachments: Option<bool>,
    pub size_greater_than: Option<i64>,
    pub received_after: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleActions {
    pub assign_inbox: Option<String>,
    pub assign_user: Option<Uuid>,
    pub add_tags: Option<Vec<String>>,
    pub set_sla_hours: Option<i32>,
    pub trigger_webhook: Option<String>,
    pub auto_reply: Option<String>,
    pub mark_as: Option<EmailState>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct RuleExecution {
    pub id: Uuid,
    pub email_id: Uuid,
    pub rule_id: Uuid,
    pub matched: bool,
    pub actions_taken: serde_json::Value,
    pub executed_at: DateTime<Utc>,
}

// ============================================================================
// AUDIT LOG MODELS
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AuditLog {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub user_id: Option<Uuid>,
    pub action: String,
    pub resource_type: String,
    pub resource_id: Option<Uuid>,
    pub details: serde_json::Value,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub created_at: DateTime<Utc>,
}

// ============================================================================
// INTERNAL NOTES
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct InternalNote {
    pub id: Uuid,
    pub email_id: Uuid,
    pub user_id: Uuid,
    pub content: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ============================================================================
// STATE HISTORY
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct StateHistory {
    pub id: Uuid,
    pub email_id: Uuid,
    pub from_state: EmailState,
    pub to_state: EmailState,
    pub changed_by: Option<Uuid>,
    pub reason: Option<String>,
    pub created_at: DateTime<Utc>,
}

// ============================================================================
// API REQUEST/RESPONSE MODELS
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CreateEmailRequest {
    pub to: Vec<String>,
    pub cc: Option<Vec<String>>,
    pub bcc: Option<Vec<String>>,
    pub subject: String,
    pub body_text: Option<String>,
    pub body_html: Option<String>,
    pub reply_to: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateEmailRequest {
    pub state: Option<EmailState>,
    pub assigned_to: Option<Uuid>,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct CreateRuleRequest {
    pub name: String,
    pub description: Option<String>,
    pub priority: Option<i32>,
    pub conditions: RuleConditions,
    pub actions: RuleActions,
}

#[derive(Debug, Serialize)]
pub struct EmailListResponse {
    pub emails: Vec<Email>,
    pub total: i64,
    pub page: i32,
    pub page_size: i32,
}

#[derive(Debug, Serialize)]
pub struct EmailDetailResponse {
    pub email: Email,
    pub attachments: Vec<EmailAttachment>,
    pub internal_notes: Vec<InternalNote>,
    pub state_history: Vec<StateHistory>,
    pub rule_executions: Vec<RuleExecution>,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub user: User,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

// ============================================================================
// WEBHOOK MODELS
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Webhook {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub url: String,
    pub events: Vec<String>,
    pub is_active: bool,
    pub secret: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct WebhookPayload {
    pub event: String,
    pub email_id: Uuid,
    pub email: Email,
    pub timestamp: DateTime<Utc>,
}

// ============================================================================
// REALTIME EVENTS
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum RealtimeEvent {
    EmailReceived { email: Email },
    EmailUpdated { email: Email },
    EmailStateChanged { email_id: Uuid, from: EmailState, to: EmailState },
    EmailAssigned { email_id: Uuid, assigned_to: Uuid },
    InternalNoteAdded { email_id: Uuid, note: InternalNote },
}
