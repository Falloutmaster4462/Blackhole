use crate::models::{Email, Rule, RuleConditions, RuleActions, RuleExecution};
use anyhow::Result;
use chrono::Utc;
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

pub struct RulesEngine {
    pool: PgPool,
}

impl RulesEngine {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
    
    /// Process email through all active rules for the tenant
    pub async fn process_email(&self, email: &mut Email) -> Result<Vec<RuleExecution>> {
        let rules = self.get_active_rules(email.tenant_id).await?;
        let mut executions = Vec::new();
        
        for rule in rules {
            let execution = self.execute_rule(email, &rule).await?;
            let matched = execution.matched;
            executions.push(execution);
            
            // If rule matched and had a state change, stop processing
            if matched {
                if let Some(actions) = rule.actions.as_object() {
                    if actions.contains_key("mark_as") {
                        break;
                    }
                }
            }
        }
        
        Ok(executions)
    }
    
    /// Get all active rules for a tenant, ordered by priority
    async fn get_active_rules(&self, tenant_id: Uuid) -> Result<Vec<Rule>> {
        let rules = sqlx::query_as::<_, Rule>(
            "SELECT * FROM rules 
             WHERE tenant_id = $1 AND is_active = TRUE 
             ORDER BY priority DESC"
        )
        .bind(tenant_id)
        .fetch_all(&self.pool)
        .await?;
        
        Ok(rules)
    }
    
    /// Execute a single rule against an email
    async fn execute_rule(&self, email: &mut Email, rule: &Rule) -> Result<RuleExecution> {
        let conditions: RuleConditions = serde_json::from_value(rule.conditions.clone())?;
        let matched = self.evaluate_conditions(email, &conditions);
        
        let mut actions_taken = json!({});
        
        if matched {
            let actions: RuleActions = serde_json::from_value(rule.actions.clone())?;
            actions_taken = self.apply_actions(email, &actions).await?;
        }
        
        // Record execution
        let execution = RuleExecution {
            id: Uuid::new_v4(),
            email_id: email.id,
            rule_id: rule.id,
            matched,
            actions_taken,
            executed_at: Utc::now(),
        };
        
        // Save execution to database
        sqlx::query(
            "INSERT INTO rule_executions 
             (id, email_id, rule_id, matched, actions_taken, executed_at)
             VALUES ($1, $2, $3, $4, $5, $6)"
        )
        .bind(&execution.id)
        .bind(&execution.email_id)
        .bind(&execution.rule_id)
        .bind(&execution.matched)
        .bind(&execution.actions_taken)
        .bind(&execution.executed_at)
        .execute(&self.pool)
        .await?;
        
        Ok(execution)
    }
    
    /// Evaluate rule conditions against email
    fn evaluate_conditions(&self, email: &Email, conditions: &RuleConditions) -> bool {
        // Check from_contains
        if let Some(from_patterns) = &conditions.from_contains {
            let matched = from_patterns.iter().any(|pattern| {
                email.from_address.to_lowercase().contains(&pattern.to_lowercase())
            });
            if !matched {
                return false;
            }
        }
        
        // Check to_contains
        if let Some(to_patterns) = &conditions.to_contains {
            let matched = to_patterns.iter().any(|pattern| {
                email.to_addresses.iter().any(|to| {
                    to.to_lowercase().contains(&pattern.to_lowercase())
                })
            });
            if !matched {
                return false;
            }
        }
        
        // Check subject_contains
        if let Some(subject_patterns) = &conditions.subject_contains {
            let matched = subject_patterns.iter().any(|pattern| {
                email.subject.to_lowercase().contains(&pattern.to_lowercase())
            });
            if !matched {
                return false;
            }
        }
        
        // Check body_contains
        if let Some(body_patterns) = &conditions.body_contains {
            if let Some(body) = &email.body_text {
                let matched = body_patterns.iter().any(|pattern| {
                    body.to_lowercase().contains(&pattern.to_lowercase())
                });
                if !matched {
                    return false;
                }
            } else {
                return false;
            }
        }
        
        // Check has_attachments
        if let Some(requires_attachments) = conditions.has_attachments {
            if email.has_attachments != requires_attachments {
                return false;
            }
        }
        
        // Check size_greater_than
        if let Some(min_size) = conditions.size_greater_than {
            if email.size_bytes <= min_size {
                return false;
            }
        }
        
        // Check received_after
        if let Some(after) = conditions.received_after {
            if email.received_at <= after {
                return false;
            }
        }
        
        true
    }
    
    /// Apply rule actions to email
    async fn apply_actions(&self, email: &mut Email, actions: &RuleActions) -> Result<serde_json::Value> {
        let mut actions_taken = json!({});
        
        // Assign inbox
        if let Some(inbox_email) = &actions.assign_inbox {
            // Look up inbox by email address
            let inbox: Option<(Uuid,)> = sqlx::query_as(
                "SELECT id FROM inboxes WHERE email_address = $1 AND tenant_id = $2"
            )
            .bind(inbox_email)
            .bind(email.tenant_id)
            .fetch_optional(&self.pool)
            .await?;
            
            if let Some((inbox_id,)) = inbox {
                email.inbox_id = Some(inbox_id);
                actions_taken["assigned_inbox"] = json!(inbox_email);
            }
        }
        
        // Assign user
        if let Some(user_id) = actions.assign_user {
            email.assigned_to = Some(user_id);
            actions_taken["assigned_user"] = json!(user_id);
        }
        
        // Add tags
        if let Some(new_tags) = &actions.add_tags {
            for tag in new_tags {
                if !email.tags.contains(tag) {
                    email.tags.push(tag.clone());
                }
            }
            actions_taken["added_tags"] = json!(new_tags);
        }
        
        // Set SLA
        if let Some(sla_hours) = actions.set_sla_hours {
            email.sla_deadline = Some(Utc::now() + chrono::Duration::hours(sla_hours as i64));
            actions_taken["sla_hours"] = json!(sla_hours);
        }
        
        // Mark as state
        if let Some(new_state) = &actions.mark_as {
            email.state = new_state.clone();
            actions_taken["marked_as"] = json!(new_state);
        }
        
        // Trigger webhook (we'll handle this separately)
        if let Some(webhook_url) = &actions.trigger_webhook {
            actions_taken["webhook_triggered"] = json!(webhook_url);
        }
        
        // Auto-reply (we'll handle this separately)
        if let Some(reply_template) = &actions.auto_reply {
            actions_taken["auto_reply"] = json!(reply_template);
        }
        
        // Update email in database
        sqlx::query(
            "UPDATE emails SET 
             inbox_id = $1, assigned_to = $2, tags = $3, 
             sla_deadline = $4, state = $5, updated_at = NOW()
             WHERE id = $6"
        )
        .bind(&email.inbox_id)
        .bind(&email.assigned_to)
        .bind(&email.tags)
        .bind(&email.sla_deadline)
        .bind(&email.state)
        .bind(&email.id)
        .execute(&self.pool)
        .await?;
        
        Ok(actions_taken)
    }
    
    /// Get rule execution history for an email
    pub async fn get_email_rule_history(&self, email_id: Uuid) -> Result<Vec<RuleExecution>> {
        let executions = sqlx::query_as::<_, RuleExecution>(
            "SELECT * FROM rule_executions 
             WHERE email_id = $1 
             ORDER BY executed_at ASC"
        )
        .bind(email_id)
        .fetch_all(&self.pool)
        .await?;
        
        Ok(executions)
    }
    
    /// Explain why an email was routed to its current location
    pub async fn explain_routing(&self, email_id: Uuid) -> Result<String> {
        let executions = self.get_email_rule_history(email_id).await?;
        
        let mut explanation = String::from("Email routing explanation:\n\n");
        
        for (idx, exec) in executions.iter().enumerate() {
            let rule: Rule = sqlx::query_as(
                "SELECT * FROM rules WHERE id = $1"
            )
            .bind(exec.rule_id)
            .fetch_one(&self.pool)
            .await?;
            
            explanation.push_str(&format!(
                "{}. Rule: '{}' (Priority: {})\n",
                idx + 1,
                rule.name,
                rule.priority
            ));
            
            if exec.matched {
                explanation.push_str("   ✓ Matched\n");
                explanation.push_str(&format!("   Actions: {}\n", exec.actions_taken));
            } else {
                explanation.push_str("   ✗ Did not match\n");
            }
            explanation.push('\n');
        }
        
        Ok(explanation)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_conditions_evaluation() {
        // We'll add unit tests here
    }
}
