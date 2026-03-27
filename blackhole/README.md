# 🕳️ Blackhole Mail - Production Demo

A deterministic, observable, enterprise-grade email system with sub-second delivery.

## 🚀 Quick Start

### Prerequisites
- Docker & Docker Compose
- Resend API key (free tier: 100 emails/day) - Get it at https://resend.com

### Setup

1. **Clone and configure**
```bash
cp .env.example .env
# Edit .env and add your RESEND_API_KEY
```

2. **Start the system**
```bash
docker-compose up -d
```

3. **Access the application**
- Frontend: http://localhost:3000
- API: http://localhost:8080
- MailHog UI: http://localhost:8025

### Default Credentials
- Email: admin@blackhole.dev
- Password: admin123

## 🏗️ Architecture

### Backend (Rust)
- **SMTP Ingest**: Custom SMTP server on port 2525
- **REST API**: Full CRUD on emails, inboxes, rules
- **WebSocket**: Real-time email delivery (SSE)
- **Rules Engine**: Deterministic routing with bytecode execution
- **Storage**: PostgreSQL + object storage simulation

### Frontend (React + TypeScript)
- **Modular UI**: Drag-and-drop panels
- **Real-time Updates**: SSE integration
- **Keyboard Navigation**: Vim-style shortcuts
- **Dark Mode**: Default theme
- **State Machine UI**: Visual lifecycle tracking

### Database
- PostgreSQL 15
- Full migrations included
- Optimized indexes for performance

## 📋 Features Included

### ✅ Core Email
- [x] SMTP ingest (TLS-ready)
- [x] Sub-second internal delivery
- [x] Threaded conversation view
- [x] Attachment support
- [x] Full-text search
- [x] Infinite scroll

### ✅ Team Features
- [x] Shared inboxes
- [x] Email assignment
- [x] Internal notes
- [x] Claim/unclaim workflow
- [x] SLA tracking
- [x] State lifecycle (NEW → CLAIMED → RESPONDED → RESOLVED → ARCHIVED)

### ✅ Automation
- [x] Visual rule builder
- [x] Deterministic routing
- [x] Auto-tagging
- [x] Webhook triggers
- [x] Auto-replies
- [x] Rule explainability panel

### ✅ Admin & Enterprise
- [x] Multi-tenant architecture
- [x] Role-based access control
- [x] Immutable audit logs
- [x] Domain management
- [x] Storage usage tracking
- [x] Retention policies

### ✅ Developer Tools
- [x] REST API
- [x] Webhook system
- [x] OAuth2 ready
- [x] Rate limiting
- [x] API documentation

## 🎯 Key Differentiators

1. **Deterministic Pipeline**: Every email's path is traceable
2. **Observable**: Real-time latency tracking per email
3. **Modular UI**: Hot-swappable panels and views
4. **Enterprise-Ready**: Multi-tenant, RBAC, audit logs
5. **Developer-Friendly**: Full API, webhooks, CLI tools

## 📊 Performance Targets

- Internal delivery: <100ms
- UI update latency: <50ms
- Search response: <200ms
- API response: <100ms (p95)

## 🔒 Security

- TLS everywhere (SMTP, HTTPS, WSS)
- DKIM/SPF/DMARC validation
- Per-tenant encryption keys
- Immutable audit trail
- Rate limiting and DDoS protection

## 📚 API Examples

### Send Email
```bash
curl -X POST http://localhost:8080/api/emails \
  -H "Authorization: Bearer YOUR_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "to": "user@example.com",
    "subject": "Hello from Blackhole",
    "body": "This is a test email"
  }'
```

### Create Rule
```bash
curl -X POST http://localhost:8080/api/rules \
  -H "Authorization: Bearer YOUR_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Support emails to support inbox",
    "conditions": {
      "to": "support@blackhole.dev"
    },
    "actions": {
      "assign_inbox": "support",
      "add_tags": ["support", "new"]
    }
  }'
```

## 🛠️ Development

### Backend
```bash
cd backend
cargo run
```

### Frontend
```bash
cd frontend
npm install
npm run dev
```

### Database Migrations
```bash
cd backend
sqlx migrate run
```

## 📦 Deployment

### Using Docker (Recommended)
```bash
docker-compose -f docker-compose.prod.yml up -d
```

### Environment Variables
See `.env.example` for all configuration options.

## 🗺️ Project Structure

```
blackhole-v2/
├── backend/           # Rust backend
│   ├── src/
│   │   ├── main.rs
│   │   ├── smtp/     # SMTP server
│   │   ├── api/      # REST API
│   │   ├── rules/    # Rules engine
│   │   ├── models/   # Data models
│   │   └── db/       # Database layer
│   ├── migrations/   # SQL migrations
│   └── Cargo.toml
├── frontend/         # React frontend
│   ├── src/
│   │   ├── components/
│   │   ├── modules/
│   │   ├── hooks/
│   │   └── App.tsx
│   └── package.json
├── docker-compose.yml
└── README.md
```

## 🤝 Contributing

This is a production demo. For the full product roadmap, see ROADMAP.md.

## 📄 License

MIT License - see LICENSE file for details.

