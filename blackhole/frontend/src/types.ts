// Core Email Types
export type EmailState = 'NEW' | 'CLAIMED' | 'RESPONDED' | 'RESOLVED' | 'ARCHIVED';

export type UserRole = 'ADMIN' | 'AGENT' | 'VIEWER';

export interface Email {
  id: string;
  tenant_id: string;
  inbox_id: string | null;
  message_id: string;
  in_reply_to: string | null;
  thread_id: string;
  
  // Headers
  from_address: string;
  from_name: string | null;
  to_addresses: string[];
  cc_addresses: string[] | null;
  bcc_addresses: string[] | null;
  subject: string;
  
  // Content
  body_text: string | null;
  body_html: string | null;
  headers: Record<string, any>;
  
  // Metadata
  size_bytes: number;
  has_attachments: boolean;
  attachment_count: number;
  
  // Lifecycle
  state: EmailState;
  assigned_to: string | null;
  tags: string[];
  
  // SLA & Timing
  sla_deadline: string | null;
  first_response_at: string | null;
  resolved_at: string | null;
  
  // Timestamps
  received_at: string;
  created_at: string;
  updated_at: string;
  
  // Observability
  ingest_latency_ms: number | null;
  delivery_latency_ms: number | null;
}

export interface EmailAttachment {
  id: string;
  email_id: string;
  filename: string;
  content_type: string;
  size_bytes: number;
  storage_path: string;
  created_at: string;
}

export interface Inbox {
  id: string;
  tenant_id: string;
  name: string;
  email_address: string;
  description: string | null;
  is_shared: boolean;
  color: string | null;
  icon: string | null;
  settings: Record<string, any>;
  created_at: string;
  updated_at: string;
}

export interface User {
  id: string;
  tenant_id: string;
  email: string;
  name: string;
  role: UserRole;
  is_active: boolean;
  last_login_at: string | null;
  created_at: string;
  updated_at: string;
}

export interface Rule {
  id: string;
  tenant_id: string;
  name: string;
  description: string | null;
  priority: number;
  is_active: boolean;
  conditions: RuleConditions;
  actions: RuleActions;
  created_by: string;
  created_at: string;
  updated_at: string;
}

export interface RuleConditions {
  from_contains?: string[];
  to_contains?: string[];
  subject_contains?: string[];
  body_contains?: string[];
  has_attachments?: boolean;
  size_greater_than?: number;
  received_after?: string;
}

export interface RuleActions {
  assign_inbox?: string;
  assign_user?: string;
  add_tags?: string[];
  set_sla_hours?: number;
  trigger_webhook?: string;
  auto_reply?: string;
  mark_as?: EmailState;
}

export interface RuleExecution {
  id: string;
  email_id: string;
  rule_id: string;
  matched: boolean;
  actions_taken: Record<string, any>;
  executed_at: string;
}

export interface InternalNote {
  id: string;
  email_id: string;
  user_id: string;
  content: string;
  created_at: string;
  updated_at: string;
}

export interface StateHistory {
  id: string;
  email_id: string;
  from_state: EmailState;
  to_state: EmailState;
  changed_by: string | null;
  reason: string | null;
  created_at: string;
}

export interface AuditLog {
  id: string;
  tenant_id: string;
  user_id: string | null;
  action: string;
  resource_type: string;
  resource_id: string | null;
  details: Record<string, any>;
  ip_address: string | null;
  user_agent: string | null;
  created_at: string;
}

// API Response Types
export interface EmailListResponse {
  emails: Email[];
  total: number;
  page: number;
  page_size: number;
}

export interface EmailDetailResponse {
  email: Email;
  attachments: EmailAttachment[];
  internal_notes: InternalNote[];
  state_history: StateHistory[];
  rule_executions: RuleExecution[];
}

export interface AuthResponse {
  token: string;
  user: User;
}

export interface StatsOverview {
  total_emails: number;
  new_emails: number;
  claimed_emails: number;
  resolved_emails: number;
  overdue_emails: number;
  avg_response_time_hours: number;
}

export interface LatencyStats {
  avg_ingest_ms: number;
  avg_delivery_ms: number;
  p95_ingest_ms: number;
  p95_delivery_ms: number;
  p99_ingest_ms: number;
  p99_delivery_ms: number;
}

// Real-time Events
export type RealtimeEvent =
  | { type: 'EmailReceived'; email: Email }
  | { type: 'EmailUpdated'; email: Email }
  | { type: 'EmailStateChanged'; email_id: string; from: EmailState; to: EmailState }
  | { type: 'EmailAssigned'; email_id: string; assigned_to: string }
  | { type: 'InternalNoteAdded'; email_id: string; note: InternalNote };

// UI State Types
export interface InboxView {
  id: string;
  name: string;
  icon: string;
  color: string;
  filter?: (email: Email) => boolean;
}

export interface KeyboardShortcut {
  key: string;
  ctrl?: boolean;
  shift?: boolean;
  alt?: boolean;
  description: string;
  action: () => void;
}
