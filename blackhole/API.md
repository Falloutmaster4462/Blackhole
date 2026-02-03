# Blackhole Mail API Documentation

Base URL: `http://localhost:8080/api`

## Authentication

Most endpoints require authentication via Bearer token.

### POST /auth/login
Login and get authentication token.

**Request:**
```json
{
  "email": "admin@blackhole.dev",
  "password": "admin123"
}
```

**Response:**
```json
{
  "token": "jwt_token_here",
  "user": {
    "id": "uuid",
    "email": "admin@blackhole.dev",
    "name": "Admin User",
    "role": "ADMIN"
  }
}
```

### GET /auth/me
Get current authenticated user.

**Headers:** `Authorization: Bearer {token}`

## Emails

### GET /emails
List all emails with optional filters.

**Query Parameters:**
- `inbox_id` (optional): Filter by inbox
- `state` (optional): Filter by state (NEW, CLAIMED, RESPONDED, RESOLVED, ARCHIVED)
- `assigned_to` (optional): Filter by assigned user
- `search` (optional): Search in subject and body
- `page` (optional): Page number (default: 1)
- `page_size` (optional): Items per page (default: 50, max: 100)

**Response:**
```json
{
  "emails": [...],
  "total": 100,
  "page": 1,
  "page_size": 50
}
```

### GET /emails/:id
Get detailed email information.

**Response:**
```json
{
  "email": { ... },
  "attachments": [...],
  "internal_notes": [...],
  "state_history": [...],
  "rule_executions": [...]
}
```

### PUT /emails/:id
Update email properties.

**Request:**
```json
{
  "state": "CLAIMED",
  "assigned_to": "user_id",
  "tags": ["urgent", "bug"]
}
```

### PUT /emails/:id/state
Update only email state.

**Request:**
```json
{
  "state": "RESOLVED"
}
```

### PUT /emails/:id/assign
Assign email to a user.

**Request:**
```json
{
  "user_id": "uuid or null"
}
```

### GET /emails/:id/notes
Get internal notes for an email.

### POST /emails/:id/notes
Add an internal note.

**Request:**
```json
{
  "content": "This is an internal note"
}
```

### GET /emails/:id/routing
Explain why this email was routed to its current location.

**Response:** Plain text explanation

### DELETE /emails/:id
Delete an email.

## Inboxes

### GET /inboxes
List all inboxes.

### GET /inboxes/:id
Get inbox details.

### POST /inboxes
Create a new inbox.

**Request:**
```json
{
  "name": "Customer Support",
  "email_address": "support@example.com",
  "description": "Main support inbox",
  "is_shared": true,
  "color": "#3B82F6"
}
```

### PUT /inboxes/:id
Update inbox.

### DELETE /inboxes/:id
Delete inbox.

### GET /inboxes/:id/emails
List emails in a specific inbox.

## Rules

### GET /rules
List all rules.

### GET /rules/:id
Get rule details.

### POST /rules
Create a new rule.

**Request:**
```json
{
  "name": "Route support emails",
  "description": "Automatically route to support inbox",
  "priority": 100,
  "conditions": {
    "to_contains": ["support@example.com"],
    "subject_contains": ["help", "issue"]
  },
  "actions": {
    "assign_inbox": "support@example.com",
    "add_tags": ["support"],
    "set_sla_hours": 24
  }
}
```

### PUT /rules/:id
Update rule.

### DELETE /rules/:id
Delete rule.

### PUT /rules/:id/toggle
Toggle rule active/inactive state.

## Users

### GET /users
List all users.

### GET /users/:id
Get user details.

### POST /users
Create a new user.

**Request:**
```json
{
  "email": "user@example.com",
  "name": "John Doe",
  "password": "secure_password",
  "role": "AGENT"
}
```

### PUT /users/:id
Update user.

## Statistics

### GET /stats/overview
Get overview statistics.

**Response:**
```json
{
  "total_emails": 1234,
  "new_emails": 45,
  "claimed_emails": 23,
  "resolved_emails": 1100,
  "overdue_emails": 6,
  "avg_response_time_hours": 2.5
}
```

### GET /stats/latency
Get latency statistics.

**Response:**
```json
{
  "avg_ingest_ms": 45.0,
  "avg_delivery_ms": 78.0,
  "p95_ingest_ms": 95,
  "p95_delivery_ms": 150,
  "p99_ingest_ms": 180,
  "p99_delivery_ms": 280
}
```

## Audit Logs

### GET /audit
List audit log entries.

**Query Parameters:**
- `page` (optional)
- `page_size` (optional)

## Real-time Events

### GET /events (SSE)
Server-Sent Events stream for real-time updates.

**Event Types:**
- `EmailReceived`: New email arrived
- `EmailUpdated`: Email was modified
- `EmailStateChanged`: Email state transition
- `EmailAssigned`: Email was assigned
- `InternalNoteAdded`: New internal note

**Example:**
```javascript
const eventSource = new EventSource('/api/events');
eventSource.onmessage = (event) => {
  const data = JSON.parse(event.data);
  console.log('Event:', data.type, data);
};
```

## Webhooks

Configure webhooks in the database or via admin panel to receive events at your endpoints.

**Webhook Payload:**
```json
{
  "event": "EmailReceived",
  "email_id": "uuid",
  "email": { ... },
  "timestamp": "2024-01-29T10:00:00Z"
}
```

## Error Responses

All errors follow this format:

```json
{
  "error": "Error message here"
}
```

**Status Codes:**
- `200`: Success
- `400`: Bad Request
- `401`: Unauthorized
- `404`: Not Found
- `500`: Internal Server Error

## Rate Limiting

API requests are rate limited to 60 requests per minute per IP address.

## SMTP Integration

Send emails via SMTP to `localhost:2525` or configure your SMTP client to use Blackhole as the server.

**Example with Node.js:**
```javascript
const nodemailer = require('nodemailer');

const transporter = nodemailer.createTransporter({
  host: 'localhost',
  port: 2525,
  secure: false,
});

await transporter.sendMail({
  from: 'sender@example.com',
  to: 'support@blackhole.dev',
  subject: 'Test Email',
  text: 'Hello from Blackhole!'
});
```
