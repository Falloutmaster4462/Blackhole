# 🕳️ Blackhole Mail - Quick Reference

## 🚀 Start in 30 Seconds

```bash
cd blackhole-v2
./start.sh
```

Access at: http://localhost:3000  
Login: admin@blackhole.dev / admin123

## 📧 Send Test Email

```bash
swaks --to support@blackhole.dev \
      --from test@test.com \
      --server localhost:2525 \
      --body "Test email"
```

Or watch in MailHog: http://localhost:8025

## 🎯 Key URLs

| Service | URL | Purpose |
|---------|-----|---------|
| Frontend | http://localhost:3000 | Main UI |
| API | http://localhost:8080 | REST API |
| MailHog | http://localhost:8025 | Email viewer |
| PostgreSQL | localhost:5432 | Database |

## 🔧 Common Commands

```bash
# Start services
docker-compose up -d

# View logs
docker-compose logs -f

# Stop services
docker-compose down

# Run tests
./test.sh

# Restart backend only
docker-compose restart backend

# View database
docker-compose exec postgres psql -U postgres blackhole

# Check service status
docker-compose ps
```

## 📊 Default Data

### Tenants
- **ID:** 00000000-0000-0000-0000-000000000001
- **Name:** Blackhole Demo
- **Domain:** blackhole.dev

### Users
- **Admin:** admin@blackhole.dev / admin123

### Inboxes
- **Support:** support@blackhole.dev
- **Sales:** sales@blackhole.dev
- **Info:** info@blackhole.dev

### Rules
- Route support@ emails → Support inbox
- Route sales@ emails → Sales inbox

## 🔍 Troubleshooting

### Services won't start
```bash
docker-compose down -v
docker-compose up -d --build
```

### Can't access frontend
- Check: `curl http://localhost:3000`
- Restart: `docker-compose restart frontend`

### Emails not arriving
- Check SMTP: `nc -zv localhost 2525`
- View logs: `docker-compose logs backend`

### Database errors
```bash
docker-compose exec postgres psql -U postgres blackhole
# Then: \dt  (list tables)
```

## 📁 Project Structure

```
blackhole-v2/
├── backend/          # Rust API + SMTP
├── frontend/         # React UI
├── docker-compose.yml
├── start.sh          # Quick start
├── test.sh           # Tests
└── README.md         # Full docs
```

## 🎨 Email States

1. **NEW** → Just arrived
2. **CLAIMED** → Someone is working on it
3. **RESPONDED** → Reply sent
4. **RESOLVED** → Issue closed
5. **ARCHIVED** → Stored for history

## 📝 API Quick Examples

### Get emails
```bash
curl http://localhost:8080/api/emails
```

### Create inbox
```bash
curl -X POST http://localhost:8080/api/inboxes \
  -H "Content-Type: application/json" \
  -d '{"name": "New Inbox", "email_address": "new@blackhole.dev", "is_shared": true}'
```

### Get stats
```bash
curl http://localhost:8080/api/stats/overview
```

## 🚢 Deploy to Production

### Railway.app
```bash
railway up
```

### Render.com
1. Connect GitHub
2. Select Web Service
3. Add PostgreSQL
4. Deploy!

### Environment Variables Needed
```
DATABASE_URL=<provided by platform>
RESEND_API_KEY=<your key>
```

## 📚 Full Documentation

- `README.md` - Overview
- `DEPLOYMENT.md` - Deploy guide
- `API.md` - API docs
- `PROJECT_OVERVIEW.md` - Architecture

## 🆘 Need Help?

1. Check logs: `docker-compose logs`
2. Run tests: `./test.sh`
3. Read docs in project root
4. Check GitHub Issues

## 🎉 Features Included

✅ SMTP server  
✅ REST API  
✅ Rules engine  
✅ Real-time updates  
✅ Modern UI  
✅ Authentication  
✅ Multi-tenant  
✅ Audit logs  
✅ SLA tracking  
✅ Full documentation  

**Total: ~5,000 lines of production code**

🕳️ Enjoy your email black hole!
