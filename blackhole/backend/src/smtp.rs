use crate::models::{Email, EmailState};
use crate::rules::RulesEngine;
use anyhow::{Result, anyhow};
use chrono::Utc;
use mail_parser::MessageParser;
use sqlx::PgPool;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use uuid::Uuid;

pub struct SmtpServer {
    pool: PgPool,
    rules_engine: Arc<RulesEngine>,
    port: u16,
}

impl SmtpServer {
    pub fn new(pool: PgPool, rules_engine: Arc<RulesEngine>, port: u16) -> Self {
        Self {
            pool,
            rules_engine,
            port,
        }
    }
    
    pub async fn start(self: Arc<Self>) -> Result<()> {
        let addr = format!("0.0.0.0:{}", self.port);
        let listener = TcpListener::bind(&addr).await?;
        
        tracing::info!("SMTP server listening on {}", addr);
        
        loop {
            let (socket, peer_addr) = listener.accept().await?;
            let server = Arc::clone(&self);
            
            tokio::spawn(async move {
                if let Err(e) = server.handle_connection(socket).await {
                    tracing::error!("SMTP connection error from {}: {:?}", peer_addr, e);
                }
            });
        }
    }
    
    async fn handle_connection(&self, socket: tokio::net::TcpStream) -> Result<()> {
        let mut reader = BufReader::new(socket);
        let mut line = String::new();
        
        // Send greeting
        Self::write_line(reader.get_mut(), "220 blackhole.dev ESMTP Ready\r\n").await?;
        
        let mut mail_from = None;
        let mut rcpt_to = Vec::new();
        let mut data = Vec::new();
        let mut in_data_mode = false;
        
        loop {
            line.clear();
            let n = reader.read_line(&mut line).await?;
            
            if n == 0 {
                break;
            }
            
            let cmd = line.trim();
            
            if in_data_mode {
                if cmd == "." {
                    in_data_mode = false;
                    
                    // Process the email
                    let email_data = String::from_utf8_lossy(&data).to_string();
                    match self.process_email(mail_from.as_ref(), &rcpt_to, &email_data).await {
                        Ok(_) => {
                            Self::write_line(reader.get_mut(), "250 OK: Message accepted\r\n").await?;
                        }
                        Err(e) => {
                            tracing::error!("Failed to process email: {:?}", e);
                            Self::write_line(reader.get_mut(), "451 Temporary failure\r\n").await?;
                        }
                    }
                    
                    // Reset for next message
                    mail_from = None;
                    rcpt_to.clear();
                    data.clear();
                } else {
                    data.extend_from_slice(line.as_bytes());
                }
                continue;
            }
            
            // Handle SMTP commands
            if cmd.starts_with("HELO") || cmd.starts_with("EHLO") {
                Self::write_line(reader.get_mut(), "250-blackhole.dev\r\n250-SIZE 52428800\r\n250 HELP\r\n").await?;
            } else if cmd.starts_with("MAIL FROM:") {
                mail_from = Some(Self::extract_email(cmd));
                Self::write_line(reader.get_mut(), "250 OK\r\n").await?;
            } else if cmd.starts_with("RCPT TO:") {
                rcpt_to.push(Self::extract_email(cmd));
                Self::write_line(reader.get_mut(), "250 OK\r\n").await?;
            } else if cmd == "DATA" {
                Self::write_line(reader.get_mut(), "354 Start mail input; end with <CRLF>.<CRLF>\r\n").await?;
                in_data_mode = true;
            } else if cmd == "QUIT" {
                Self::write_line(reader.get_mut(), "221 Bye\r\n").await?;
                break;
            } else if cmd == "RSET" {
                mail_from = None;
                rcpt_to.clear();
                data.clear();
                Self::write_line(reader.get_mut(), "250 OK\r\n").await?;
            } else if cmd == "NOOP" {
                Self::write_line(reader.get_mut(), "250 OK\r\n").await?;
            } else {
                Self::write_line(reader.get_mut(), "500 Command not recognized\r\n").await?;
            }
        }
        
        Ok(())
    }
    
    async fn write_line<R: AsyncWriteExt + Unpin>(writer: &mut R, msg: &str) -> Result<()> {
        writer.write_all(msg.as_bytes()).await?;
        Ok(())
    }
    
    fn extract_email(cmd: &str) -> String {
        cmd.split(':')
            .nth(1)
            .unwrap_or("")
            .trim()
            .trim_matches('<')
            .trim_matches('>')
            .to_string()
    }
    
    async fn process_email(
        &self,
        from: Option<&String>,
        to: &[String],
        raw_email: &str,
    ) -> Result<Uuid> {
        let start = std::time::Instant::now();
        
        // Parse email
        let parser = MessageParser::default();
        let message = parser.parse(raw_email.as_bytes())
            .ok_or_else(|| anyhow!("Failed to parse email"))?;
        
        // Extract fields
        let from_address = from.cloned().unwrap_or_else(|| "unknown@unknown".to_string());
        let subject = message.subject().unwrap_or("(no subject)").to_string();
        let message_id = message.message_id()
            .map(|s| s.to_string())
            .unwrap_or_else(|| format!("<{}@blackhole.dev>", Uuid::new_v4()));
        let in_reply_to = message
            .in_reply_to()
            .as_text()
            .map(str::to_string);


        // Get body
        let body_text = message.body_text(0).map(|s| s.to_string());
        let body_html = message.body_html(0).map(|s| s.to_string());
        
        // Calculate size
        let size_bytes = raw_email.len() as i64;
        
        // Check for attachments
        let attachment_count = message.attachments().count();
        let has_attachments = attachment_count > 0;
        
        // Get tenant (for demo, use default tenant)
        let tenant_id = Uuid::parse_str("00000000-0000-0000-0000-000000000001")?;
        
        // Generate thread_id (use message_id for new threads, or in_reply_to for replies)
        let thread_id = if let Some(_reply_to) = &in_reply_to {
            // In production, we'd look up the thread_id from the parent email
            Uuid::new_v4()
        } else {
            Uuid::new_v4()
        };
        
        // Create email record
        let mut email = Email {
            id: Uuid::new_v4(),
            tenant_id,
            inbox_id: None,
            message_id: message_id.clone(),
            in_reply_to,
            thread_id,
            from_address: from_address.clone(),
            from_name: None,
            to_addresses: to.to_vec(),
            cc_addresses: None,
            bcc_addresses: None,
            subject,
            body_text,
            body_html,
            headers: serde_json::json!({}),
            size_bytes,
            has_attachments,
            attachment_count: attachment_count as i32,
            state: EmailState::New,
            assigned_to: None,
            tags: vec![],
            sla_deadline: None,
            first_response_at: None,
            resolved_at: None,
            received_at: Utc::now(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            ingest_latency_ms: None,
            delivery_latency_ms: None,
        };
        
        // Insert email into database
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
                $1, $2, $3, $4, $5, $6,
                $7, $8, $9, $10, $11,
                $12, $13, $14, $15,
                $16, $17, $18,
                $19, $20, $21,
                $22, $23, $24,
                $25, $26, $27,
                $28, $29
            )"
        )
        .bind(&email.id)
        .bind(&email.tenant_id)
        .bind(&email.inbox_id)
        .bind(&email.message_id)
        .bind(&email.in_reply_to)
        .bind(&email.thread_id)
        .bind(&email.from_address)
        .bind(&email.from_name)
        .bind(&email.to_addresses)
        .bind(&email.cc_addresses)
        .bind(&email.bcc_addresses)
        .bind(&email.subject)
        .bind(&email.body_text)
        .bind(&email.body_html)
        .bind(&email.headers)
        .bind(&email.size_bytes)
        .bind(&email.has_attachments)
        .bind(&email.attachment_count)
        .bind(&email.state)
        .bind(&email.assigned_to)
        .bind(&email.tags)
        .bind(&email.sla_deadline)
        .bind(&email.first_response_at)
        .bind(&email.resolved_at)
        .bind(&email.received_at)
        .bind(&email.created_at)
        .bind(&email.updated_at)
        .bind(&email.ingest_latency_ms)
        .bind(&email.delivery_latency_ms)
        .execute(&self.pool)
        .await?;
        
        let ingest_latency = start.elapsed().as_millis() as i32;
        email.ingest_latency_ms = Some(ingest_latency);
        
        // Process through rules engine
        let _executions = self.rules_engine.process_email(&mut email).await?;
        
        let delivery_latency = start.elapsed().as_millis() as i32;
        
        // Update latencies
        sqlx::query(
            "UPDATE emails SET 
             ingest_latency_ms = $1, 
             delivery_latency_ms = $2,
             updated_at = NOW()
             WHERE id = $3"
        )
        .bind(ingest_latency)
        .bind(delivery_latency)
        .bind(&email.id)
        .execute(&self.pool)
        .await?;
        
        tracing::info!(
            "Email processed: {} (ingest: {}ms, delivery: {}ms)",
            email.id,
            ingest_latency,
            delivery_latency
        );
        
        Ok(email.id)
    }
}
