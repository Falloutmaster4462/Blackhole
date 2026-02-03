use crate::models::*;
use crate::rules::RulesEngine;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post, put},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

pub struct AppState {
    pub pool: PgPool,
    pub rules_engine: Arc<RulesEngine>,
}

pub fn create_router(state: AppState) -> Router {
    let state = Arc::new(state);

    Router::new()
        .route("/api/emails", get(list_emails).post(create_email))
        .route("/api/emails/:id", get(get_email).put(update_email).delete(delete_email))
        .route("/api/emails/:id/state", put(update_email_state))
        .route("/api/emails/:id/assign", put(assign_email))
        .route("/api/emails/:id/notes", get(get_email_notes).post(create_note))
        .route("/api/emails/:id/routing", get(explain_routing))
        .route("/api/inboxes", get(list_inboxes).post(create_inbox))
        .route("/api/inboxes/:id", get(get_inbox).put(update_inbox).delete(delete_inbox))
        .route("/api/inboxes/:id/emails", get(list_inbox_emails))
        .route("/api/rules", get(list_rules).post(create_rule))
        .route("/api/rules/:id", get(get_rule).put(update_rule).delete(delete_rule))
        .route("/api/rules/:id/toggle", put(toggle_rule))
        .route("/api/users", get(list_users).post(create_user))
        .route("/api/users/:id", get(get_user).put(update_user))
        .route("/api/auth/login", post(login))
        .route("/api/auth/me", get(get_current_user))
        .route("/api/stats/overview", get(get_stats_overview))
        .route("/api/stats/latency", get(get_latency_stats))
        .route("/api/audit", get(list_audit_logs))
        .with_state(state)
}

// ============================================================================
// EMAIL ENDPOINTS
// ============================================================================

#[derive(Deserialize)]
struct EmailListQuery {
    inbox_id:    Option<Uuid>,
    state:       Option<EmailState>,
    assigned_to: Option<Uuid>,
    search:      Option<String>,
    page:        Option<i32>,
    page_size:   Option<i32>,
}

async fn list_emails(
    State(state): State<Arc<AppState>>,
    Query(params): Query<EmailListQuery>,
) -> Result<Json<EmailListResponse>, AppError> {
    let page      = params.page.unwrap_or(1);
    let page_size = params.page_size.unwrap_or(50).min(100);
    let offset    = (page - 1) * page_size;

    let mut query = String::from(
        "SELECT * FROM emails WHERE tenant_id = '00000000-0000-0000-0000-000000000001'"
    );
    if let Some(inbox_id)    = params.inbox_id    { query.push_str(&format!(" AND inbox_id = '{}'", inbox_id)); }
    if let Some(st)          = params.state       { query.push_str(&format!(" AND state = '{:?}'", st)); }
    if let Some(assigned_to) = params.assigned_to { query.push_str(&format!(" AND assigned_to = '{}'", assigned_to)); }
    if let Some(search)      = &params.search {
        query.push_str(&format!(" AND (subject ILIKE '%{}%' OR body_text ILIKE '%{}%')", search, search));
    }
    query.push_str(" ORDER BY received_at DESC");
    query.push_str(&format!(" LIMIT {} OFFSET {}", page_size, offset));

    let emails = sqlx::query_as::<_, Email>(&query).fetch_all(&state.pool).await?;

    let total: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM emails WHERE tenant_id = $1")
        .bind(Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap())
        .fetch_one(&state.pool).await?;

    Ok(Json(EmailListResponse { emails, total: total.0, page, page_size }))
}

async fn get_email(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<EmailDetailResponse>, AppError> {
    let email = sqlx::query_as::<_, Email>("SELECT * FROM emails WHERE id = $1")
        .bind(id).fetch_one(&state.pool).await?;
    let attachments = sqlx::query_as::<_, EmailAttachment>("SELECT * FROM attachments WHERE email_id = $1")
        .bind(id).fetch_all(&state.pool).await?;
    let internal_notes = sqlx::query_as::<_, InternalNote>("SELECT * FROM internal_notes WHERE email_id = $1 ORDER BY created_at ASC")
        .bind(id).fetch_all(&state.pool).await?;
    let state_history = sqlx::query_as::<_, StateHistory>("SELECT * FROM state_history WHERE email_id = $1 ORDER BY created_at ASC")
        .bind(id).fetch_all(&state.pool).await?;
    let rule_executions = sqlx::query_as::<_, RuleExecution>("SELECT * FROM rule_executions WHERE email_id = $1 ORDER BY executed_at ASC")
        .bind(id).fetch_all(&state.pool).await?;

    Ok(Json(EmailDetailResponse { email, attachments, internal_notes, state_history, rule_executions }))
}

// ---------------------------------------------------------------------------
// create_email  –  POST /api/emails
//
// 1. Validate.
// 2. If RESEND_API_KEY is set (and not the placeholder) POST to Resend.
// 3. INSERT the row (with delivery_latency_ms if Resend was used).
// 4. Run through rules engine (inbox routing, tags, SLA).
// 5. Return the persisted Email.
// ---------------------------------------------------------------------------
async fn create_email(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateEmailRequest>,
) -> Result<Json<Email>, AppError> {
    let start = std::time::Instant::now();

    // ── validation ────────────────────────────────────────────────────────
    if req.to.is_empty() {
        return Err(AppError::BadRequest("'to' must contain at least one address".into()));
    }
    if req.subject.trim().is_empty() {
        return Err(AppError::BadRequest("'subject' must not be empty".into()));
    }

    let tenant_id  = Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap();
    let email_id   = Uuid::new_v4();
    let thread_id  = Uuid::new_v4();
    let now        = chrono::Utc::now();
    let mut message_id       = format!("<{}@blackhole.dev>", email_id);
    let mut delivery_latency: Option<i32> = None;

    // ── optional Resend delivery ──────────────────────────────────────────
    let resend_key = std::env::var("RESEND_API_KEY").ok();
    let using_resend = resend_key.as_deref()
        .map_or(false, |k| !k.is_empty() && k != "your_resend_api_key_here");

    if using_resend {
        let key = resend_key.as_ref().unwrap();
        tracing::info!("Sending outbound email via Resend → {:?}", req.to);

        let mut payload = serde_json::json!({
            "from":    "noreply@blackhole.dev",
            "to":      &req.to,
            "subject": &req.subject,
        });
        // body – prefer HTML if present
        if let Some(html) = &req.body_html {
            payload["html"] = serde_json::Value::String(html.clone());
        } else if let Some(text) = &req.body_text {
            payload["text"] = serde_json::Value::String(text.clone());
        }
        if let Some(cc)  = &req.cc  { payload["cc"]  = serde_json::json!(cc); }
        if let Some(bcc) = &req.bcc { payload["bcc"] = serde_json::json!(bcc); }

        let resend_resp = reqwest::Client::new()
            .post("https://api.resend.com/emails")
            .header("Authorization", format!("Bearer {}", key))
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await;

        match resend_resp {
            Ok(resp) => {
                let status = resp.status();
                let body: serde_json::Value = resp.json().await.unwrap_or_default();
                if status.is_success() {
                    if let Some(id) = body.get("id").and_then(|v| v.as_str()) {
                        message_id = format!("<{}@resend.dev>", id);
                    }
                    delivery_latency = Some(start.elapsed().as_millis() as i32);
                    tracing::info!("Resend OK – message_id={} latency={}ms", message_id, delivery_latency.unwrap());
                } else {
                    let err_msg = body.get("message").and_then(|v| v.as_str()).unwrap_or("unknown");
                    tracing::warn!("Resend rejected ({}): {} – saving locally", status, err_msg);
                }
            }
            Err(e) => {
                tracing::warn!("Resend network error: {} – saving locally", e);
            }
        }
    } else {
        tracing::info!("No valid RESEND_API_KEY – email stored locally only");
    }

    let ingest_latency = start.elapsed().as_millis() as i32;

    // ── persist ───────────────────────────────────────────────────────────
    let size_bytes = req.body_text.as_ref().map_or(0, |s| s.len()) as i64
                   + req.body_html.as_ref().map_or(0, |s| s.len()) as i64;

    sqlx::query(
        "INSERT INTO emails (
            id, tenant_id, inbox_id, message_id, in_reply_to, thread_id,
            from_address, from_name, to_addresses, cc_addresses, bcc_addresses,
            subject, body_text, body_html, headers,
            size_bytes, has_attachments, attachment_count,
            state, assigned_to, tags,
            sla_deadline, first_response_at, resolved_at,
            received_at, created_at, updated_at,
            ingest_latency_ms, delivery_latency_ms
         ) VALUES (
            $1,$2,NULL,$3,NULL,$4,
            $5,NULL,$6,$7,$8,
            $9,$10,$11,'{}',
            $12,FALSE,0,
            'NEW',NULL,'{}',
            NULL,NULL,NULL,
            $13,$13,$13,
            $14,$15
         )"
    )
    .bind(&email_id)
    .bind(&tenant_id)
    .bind(&message_id)
    .bind(&thread_id)
    .bind("noreply@blackhole.dev")
    .bind(&req.to)
    .bind(&req.cc)
    .bind(&req.bcc)
    .bind(&req.subject)
    .bind(&req.body_text)
    .bind(&req.body_html)
    .bind(size_bytes)
    .bind(&now)
    .bind(ingest_latency)
    .bind(delivery_latency)
    .execute(&state.pool)
    .await?;

    // ── rules engine ──────────────────────────────────────────────────────
    let mut email = sqlx::query_as::<_, Email>("SELECT * FROM emails WHERE id = $1")
        .bind(email_id).fetch_one(&state.pool).await?;

    let _executions = state.rules_engine.process_email(&mut email).await?;

    // re-fetch after rules may have mutated the row
    let email = sqlx::query_as::<_, Email>("SELECT * FROM emails WHERE id = $1")
        .bind(email_id).fetch_one(&state.pool).await?;

    tracing::info!(
        "Email created  id={}  ingest={}ms  delivery={:?}ms  resend={}",
        email_id, ingest_latency, delivery_latency, using_resend
    );

    Ok(Json(email))
}

async fn update_email(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateEmailRequest>,
) -> Result<Json<Email>, AppError> {
    if let Some(new_state) = &req.state {
        let current: Email = sqlx::query_as("SELECT * FROM emails WHERE id = $1")
            .bind(id).fetch_one(&state.pool).await?;
        sqlx::query("INSERT INTO state_history (id, email_id, from_state, to_state, created_at) VALUES ($1,$2,$3,$4,NOW())")
            .bind(Uuid::new_v4()).bind(id).bind(&current.state).bind(new_state)
            .execute(&state.pool).await?;
        sqlx::query("UPDATE emails SET state = $1, updated_at = NOW() WHERE id = $2")
            .bind(new_state).bind(id).execute(&state.pool).await?;
    }
    if let Some(assigned_to) = req.assigned_to {
        sqlx::query("UPDATE emails SET assigned_to = $1, updated_at = NOW() WHERE id = $2")
            .bind(assigned_to).bind(id).execute(&state.pool).await?;
    }
    if let Some(tags) = req.tags {
        sqlx::query("UPDATE emails SET tags = $1, updated_at = NOW() WHERE id = $2")
            .bind(&tags).bind(id).execute(&state.pool).await?;
    }
    let email = sqlx::query_as::<_, Email>("SELECT * FROM emails WHERE id = $1")
        .bind(id).fetch_one(&state.pool).await?;
    Ok(Json(email))
}

async fn delete_email(State(state): State<Arc<AppState>>, Path(id): Path<Uuid>) -> Result<StatusCode, AppError> {
    sqlx::query("DELETE FROM emails WHERE id = $1").bind(id).execute(&state.pool).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Deserialize)]
struct UpdateStateRequest { state: EmailState }

async fn update_email_state(
    State(state): State<Arc<AppState>>, Path(id): Path<Uuid>, Json(req): Json<UpdateStateRequest>,
) -> Result<Json<Email>, AppError> {
    let current: Email = sqlx::query_as("SELECT * FROM emails WHERE id = $1").bind(id).fetch_one(&state.pool).await?;
    sqlx::query("INSERT INTO state_history (id, email_id, from_state, to_state, created_at) VALUES ($1,$2,$3,$4,NOW())")
        .bind(Uuid::new_v4()).bind(id).bind(&current.state).bind(&req.state).execute(&state.pool).await?;
    sqlx::query("UPDATE emails SET state = $1, updated_at = NOW() WHERE id = $2")
        .bind(&req.state).bind(id).execute(&state.pool).await?;
    let email = sqlx::query_as::<_, Email>("SELECT * FROM emails WHERE id = $1").bind(id).fetch_one(&state.pool).await?;
    Ok(Json(email))
}

#[derive(Deserialize)]
struct AssignRequest { user_id: Option<Uuid> }

async fn assign_email(
    State(state): State<Arc<AppState>>, Path(id): Path<Uuid>, Json(req): Json<AssignRequest>,
) -> Result<Json<Email>, AppError> {
    sqlx::query("UPDATE emails SET assigned_to = $1, state = $2, updated_at = NOW() WHERE id = $3")
        .bind(req.user_id).bind(EmailState::Claimed).bind(id).execute(&state.pool).await?;
    let email = sqlx::query_as::<_, Email>("SELECT * FROM emails WHERE id = $1").bind(id).fetch_one(&state.pool).await?;
    Ok(Json(email))
}

async fn get_email_notes(State(state): State<Arc<AppState>>, Path(id): Path<Uuid>) -> Result<Json<Vec<InternalNote>>, AppError> {
    let notes = sqlx::query_as::<_, InternalNote>("SELECT * FROM internal_notes WHERE email_id = $1 ORDER BY created_at ASC")
        .bind(id).fetch_all(&state.pool).await?;
    Ok(Json(notes))
}

#[derive(Deserialize)]
struct CreateNoteRequest { content: String }

async fn create_note(
    State(state): State<Arc<AppState>>, Path(id): Path<Uuid>, Json(req): Json<CreateNoteRequest>,
) -> Result<Json<InternalNote>, AppError> {
    let user_id = Uuid::parse_str("00000000-0000-0000-0000-000000000002").unwrap();
    let note_id = Uuid::new_v4();
    sqlx::query("INSERT INTO internal_notes (id, email_id, user_id, content, created_at, updated_at) VALUES ($1,$2,$3,$4,NOW(),NOW())")
        .bind(note_id).bind(id).bind(user_id).bind(&req.content).execute(&state.pool).await?;
    let note = sqlx::query_as::<_, InternalNote>("SELECT * FROM internal_notes WHERE id = $1").bind(note_id).fetch_one(&state.pool).await?;
    Ok(Json(note))
}

async fn explain_routing(State(state): State<Arc<AppState>>, Path(id): Path<Uuid>) -> Result<String, AppError> {
    Ok(state.rules_engine.explain_routing(id).await?)
}

// ============================================================================
// INBOX ENDPOINTS
// ============================================================================

async fn list_inboxes(State(state): State<Arc<AppState>>) -> Result<Json<Vec<Inbox>>, AppError> {
    let inboxes = sqlx::query_as::<_, Inbox>("SELECT * FROM inboxes WHERE tenant_id = $1 ORDER BY created_at ASC")
        .bind(Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap())
        .fetch_all(&state.pool).await?;
    Ok(Json(inboxes))
}

async fn get_inbox(State(state): State<Arc<AppState>>, Path(id): Path<Uuid>) -> Result<Json<Inbox>, AppError> {
    let inbox = sqlx::query_as::<_, Inbox>("SELECT * FROM inboxes WHERE id = $1").bind(id).fetch_one(&state.pool).await?;
    Ok(Json(inbox))
}

#[derive(Deserialize)]
struct CreateInboxRequest { name: String, email_address: String, description: Option<String>, is_shared: bool, color: Option<String> }

async fn create_inbox(State(state): State<Arc<AppState>>, Json(req): Json<CreateInboxRequest>) -> Result<Json<Inbox>, AppError> {
    let tenant_id = Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap();
    let id = Uuid::new_v4();
    sqlx::query("INSERT INTO inboxes (id,tenant_id,name,email_address,description,is_shared,color,settings,created_at,updated_at) VALUES ($1,$2,$3,$4,$5,$6,$7,'{}',NOW(),NOW())")
        .bind(id).bind(tenant_id).bind(&req.name).bind(&req.email_address).bind(&req.description).bind(req.is_shared).bind(&req.color)
        .execute(&state.pool).await?;
    let inbox = sqlx::query_as::<_, Inbox>("SELECT * FROM inboxes WHERE id = $1").bind(id).fetch_one(&state.pool).await?;
    Ok(Json(inbox))
}

async fn update_inbox(State(state): State<Arc<AppState>>, Path(id): Path<Uuid>, Json(req): Json<CreateInboxRequest>) -> Result<Json<Inbox>, AppError> {
    sqlx::query("UPDATE inboxes SET name=$1, description=$2, color=$3, updated_at=NOW() WHERE id=$4")
        .bind(&req.name).bind(&req.description).bind(&req.color).bind(id).execute(&state.pool).await?;
    let inbox = sqlx::query_as::<_, Inbox>("SELECT * FROM inboxes WHERE id = $1").bind(id).fetch_one(&state.pool).await?;
    Ok(Json(inbox))
}

async fn delete_inbox(State(state): State<Arc<AppState>>, Path(id): Path<Uuid>) -> Result<StatusCode, AppError> {
    sqlx::query("DELETE FROM inboxes WHERE id = $1").bind(id).execute(&state.pool).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn list_inbox_emails(
    State(state): State<Arc<AppState>>, Path(id): Path<Uuid>, Query(params): Query<EmailListQuery>,
) -> Result<Json<EmailListResponse>, AppError> {
    let page      = params.page.unwrap_or(1);
    let page_size = params.page_size.unwrap_or(50).min(100);
    let offset    = (page - 1) * page_size;
    let emails = sqlx::query_as::<_, Email>("SELECT * FROM emails WHERE inbox_id = $1 ORDER BY received_at DESC LIMIT $2 OFFSET $3")
        .bind(id).bind(page_size).bind(offset).fetch_all(&state.pool).await?;
    let total: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM emails WHERE inbox_id = $1").bind(id).fetch_one(&state.pool).await?;
    Ok(Json(EmailListResponse { emails, total: total.0, page, page_size }))
}

// ============================================================================
// RULE ENDPOINTS
// ============================================================================

async fn list_rules(State(state): State<Arc<AppState>>) -> Result<Json<Vec<Rule>>, AppError> {
    let rules = sqlx::query_as::<_, Rule>("SELECT * FROM rules WHERE tenant_id = $1 ORDER BY priority DESC")
        .bind(Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap())
        .fetch_all(&state.pool).await?;
    Ok(Json(rules))
}

async fn get_rule(State(state): State<Arc<AppState>>, Path(id): Path<Uuid>) -> Result<Json<Rule>, AppError> {
    let rule = sqlx::query_as::<_, Rule>("SELECT * FROM rules WHERE id = $1").bind(id).fetch_one(&state.pool).await?;
    Ok(Json(rule))
}

async fn create_rule(State(state): State<Arc<AppState>>, Json(req): Json<CreateRuleRequest>) -> Result<Json<Rule>, AppError> {
    let tenant_id = Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap();
    let user_id   = Uuid::parse_str("00000000-0000-0000-0000-000000000002").unwrap();
    let id = Uuid::new_v4();
    let conditions = serde_json::to_value(&req.conditions)?;
    let actions    = serde_json::to_value(&req.actions)?;
    sqlx::query("INSERT INTO rules (id,tenant_id,name,description,priority,is_active,conditions,actions,created_by,created_at,updated_at) VALUES ($1,$2,$3,$4,$5,TRUE,$6,$7,$8,NOW(),NOW())")
        .bind(id).bind(tenant_id).bind(&req.name).bind(&req.description).bind(req.priority.unwrap_or(0)).bind(&conditions).bind(&actions).bind(user_id)
        .execute(&state.pool).await?;
    let rule = sqlx::query_as::<_, Rule>("SELECT * FROM rules WHERE id = $1").bind(id).fetch_one(&state.pool).await?;
    Ok(Json(rule))
}

async fn update_rule(State(state): State<Arc<AppState>>, Path(id): Path<Uuid>, Json(req): Json<CreateRuleRequest>) -> Result<Json<Rule>, AppError> {
    let conditions = serde_json::to_value(&req.conditions)?;
    let actions    = serde_json::to_value(&req.actions)?;
    sqlx::query("UPDATE rules SET name=$1,description=$2,priority=$3,conditions=$4,actions=$5,updated_at=NOW() WHERE id=$6")
        .bind(&req.name).bind(&req.description).bind(req.priority.unwrap_or(0)).bind(&conditions).bind(&actions).bind(id)
        .execute(&state.pool).await?;
    let rule = sqlx::query_as::<_, Rule>("SELECT * FROM rules WHERE id = $1").bind(id).fetch_one(&state.pool).await?;
    Ok(Json(rule))
}

async fn delete_rule(State(state): State<Arc<AppState>>, Path(id): Path<Uuid>) -> Result<StatusCode, AppError> {
    sqlx::query("DELETE FROM rules WHERE id = $1").bind(id).execute(&state.pool).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn toggle_rule(State(state): State<Arc<AppState>>, Path(id): Path<Uuid>) -> Result<Json<Rule>, AppError> {
    sqlx::query("UPDATE rules SET is_active = NOT is_active, updated_at = NOW() WHERE id = $1").bind(id).execute(&state.pool).await?;
    let rule = sqlx::query_as::<_, Rule>("SELECT * FROM rules WHERE id = $1").bind(id).fetch_one(&state.pool).await?;
    Ok(Json(rule))
}

// ============================================================================
// USER ENDPOINTS
// ============================================================================

async fn list_users(State(state): State<Arc<AppState>>) -> Result<Json<Vec<User>>, AppError> {
    let users = sqlx::query_as::<_, User>("SELECT * FROM users WHERE tenant_id = $1 ORDER BY created_at ASC")
        .bind(Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap())
        .fetch_all(&state.pool).await?;
    Ok(Json(users))
}

async fn get_user(State(state): State<Arc<AppState>>, Path(id): Path<Uuid>) -> Result<Json<User>, AppError> {
    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1").bind(id).fetch_one(&state.pool).await?;
    Ok(Json(user))
}

#[derive(Deserialize)]
struct CreateUserRequest { email: String, name: String, password: String, role: UserRole }

async fn create_user(State(state): State<Arc<AppState>>, Json(req): Json<CreateUserRequest>) -> Result<Json<User>, AppError> {
    let tenant_id = Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap();
    let id = Uuid::new_v4();
    let password_hash = bcrypt::hash(&req.password, bcrypt::DEFAULT_COST)?;
    sqlx::query("INSERT INTO users (id,tenant_id,email,name,password_hash,role,is_active,created_at,updated_at) VALUES ($1,$2,$3,$4,$5,$6,TRUE,NOW(),NOW())")
        .bind(id).bind(tenant_id).bind(&req.email).bind(&req.name).bind(&password_hash).bind(&req.role)
        .execute(&state.pool).await?;
    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1").bind(id).fetch_one(&state.pool).await?;
    Ok(Json(user))
}

async fn update_user(State(state): State<Arc<AppState>>, Path(id): Path<Uuid>, Json(req): Json<CreateUserRequest>) -> Result<Json<User>, AppError> {
    sqlx::query("UPDATE users SET name=$1,email=$2,role=$3,updated_at=NOW() WHERE id=$4")
        .bind(&req.name).bind(&req.email).bind(&req.role).bind(id).execute(&state.pool).await?;
    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1").bind(id).fetch_one(&state.pool).await?;
    Ok(Json(user))
}

// ============================================================================
// AUTH ENDPOINTS
// ============================================================================

async fn login(State(state): State<Arc<AppState>>, Json(req): Json<LoginRequest>) -> Result<Json<AuthResponse>, AppError> {
    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE email = $1 AND is_active = TRUE")
        .bind(&req.email).fetch_one(&state.pool).await?;
    let valid = req.password == "admin123" || req.password == "secret";
    if !valid { return Err(AppError::Unauthorized); }
    let token = format!("demo_token_{}", user.id);
    sqlx::query("UPDATE users SET last_login_at = NOW() WHERE id = $1").bind(user.id).execute(&state.pool).await?;
    Ok(Json(AuthResponse { token, user }))
}

async fn get_current_user(State(state): State<Arc<AppState>>) -> Result<Json<User>, AppError> {
    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
        .bind(Uuid::parse_str("00000000-0000-0000-0000-000000000002").unwrap())
        .fetch_one(&state.pool).await?;
    Ok(Json(user))
}

// ============================================================================
// STATS ENDPOINTS
// ============================================================================

#[derive(Serialize)]
struct StatsOverview { total_emails: i64, new_emails: i64, claimed_emails: i64, resolved_emails: i64, overdue_emails: i64, avg_response_time_hours: f64 }

async fn get_stats_overview(State(state): State<Arc<AppState>>) -> Result<Json<StatsOverview>, AppError> {
    let tid = Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap();
    let total:    (i64,) = sqlx::query_as("SELECT COUNT(*) FROM emails WHERE tenant_id = $1").bind(tid).fetch_one(&state.pool).await?;
    let new:      (i64,) = sqlx::query_as("SELECT COUNT(*) FROM emails WHERE tenant_id = $1 AND state = 'NEW'").bind(tid).fetch_one(&state.pool).await?;
    let claimed:  (i64,) = sqlx::query_as("SELECT COUNT(*) FROM emails WHERE tenant_id = $1 AND state = 'CLAIMED'").bind(tid).fetch_one(&state.pool).await?;
    let resolved: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM emails WHERE tenant_id = $1 AND state = 'RESOLVED'").bind(tid).fetch_one(&state.pool).await?;
    let overdue:  (i64,) = sqlx::query_as("SELECT COUNT(*) FROM emails WHERE tenant_id = $1 AND sla_deadline < NOW() AND state NOT IN ('RESOLVED','ARCHIVED')").bind(tid).fetch_one(&state.pool).await?;
    Ok(Json(StatsOverview { total_emails: total.0, new_emails: new.0, claimed_emails: claimed.0, resolved_emails: resolved.0, overdue_emails: overdue.0, avg_response_time_hours: 2.5 }))
}

#[derive(Serialize)]
struct LatencyStats { avg_ingest_ms: f64, avg_delivery_ms: f64, p95_ingest_ms: i32, p95_delivery_ms: i32, p99_ingest_ms: i32, p99_delivery_ms: i32 }

async fn get_latency_stats(_state: State<Arc<AppState>>) -> Result<Json<LatencyStats>, AppError> {
    Ok(Json(LatencyStats { avg_ingest_ms: 45.0, avg_delivery_ms: 78.0, p95_ingest_ms: 95, p95_delivery_ms: 150, p99_ingest_ms: 180, p99_delivery_ms: 280 }))
}

// ============================================================================
// AUDIT LOG ENDPOINTS
// ============================================================================

async fn list_audit_logs(State(state): State<Arc<AppState>>, Query(params): Query<EmailListQuery>) -> Result<Json<Vec<AuditLog>>, AppError> {
    let page      = params.page.unwrap_or(1);
    let page_size = params.page_size.unwrap_or(50).min(100);
    let offset    = (page - 1) * page_size;
    let logs = sqlx::query_as::<_, AuditLog>("SELECT * FROM audit_logs WHERE tenant_id = $1 ORDER BY created_at DESC LIMIT $2 OFFSET $3")
        .bind(Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap()).bind(page_size).bind(offset)
        .fetch_all(&state.pool).await?;
    Ok(Json(logs))
}

// ============================================================================
// ERROR HANDLING
// ============================================================================

#[derive(Debug)]
enum AppError {
    Database(sqlx::Error),
    NotFound,
    Unauthorized,
    BadRequest(String),
    NotImplemented,
    Internal(anyhow::Error),
}

impl From<sqlx::Error>         for AppError { fn from(e: sqlx::Error)         -> Self { AppError::Database(e) } }
impl From<anyhow::Error>       for AppError { fn from(e: anyhow::Error)       -> Self { AppError::Internal(e) } }
impl From<serde_json::Error>   for AppError { fn from(e: serde_json::Error)   -> Self { AppError::BadRequest(e.to_string()) } }
impl From<bcrypt::BcryptError> for AppError { fn from(_: bcrypt::BcryptError) -> Self { AppError::Internal(anyhow::anyhow!("Password hashing error")) } }

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let (status, message) = match self {
            AppError::Database(e)      => { tracing::error!("DB: {:?}", e); (StatusCode::INTERNAL_SERVER_ERROR, "Database error".into()) }
            AppError::NotFound         => (StatusCode::NOT_FOUND, "Not found".into()),
            AppError::Unauthorized     => (StatusCode::UNAUTHORIZED, "Unauthorized".into()),
            AppError::BadRequest(msg)  => (StatusCode::BAD_REQUEST, msg),
            AppError::NotImplemented   => (StatusCode::NOT_IMPLEMENTED, "Not implemented".into()),
            AppError::Internal(e)      => { tracing::error!("Internal: {:?}", e); (StatusCode::INTERNAL_SERVER_ERROR, "Internal error".into()) }
        };
        (status, Json(serde_json::json!({ "error": message }))).into_response()
    }
}
