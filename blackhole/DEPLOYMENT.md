# Blackhole Mail - Deployment Guide

## Quick Start (Local Development)

### 1. Prerequisites
- Docker and Docker Compose
- Or: Rust 1.75+, Node.js 20+, PostgreSQL 15+

### 2. Using Docker (Recommended)

```bash
# Clone the repository
cd blackhole-v2

# Create environment file
cp .env.example .env

# Edit .env and add your Resend API key (optional for testing)
nano .env

# Start all services
docker-compose up -d

# View logs
docker-compose logs -f

# Stop services
docker-compose down
```

**Access the application:**
- Frontend: http://localhost:3000
- API: http://localhost:8080
- MailHog UI: http://localhost:8025 (for testing SMTP)
- PostgreSQL: localhost:5432

**Default credentials:**
- Email: admin@blackhole.dev
- Password: admin123

### 3. Manual Setup (Without Docker)

#### Backend Setup

```bash
cd backend

# Install Rust (if not installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Setup PostgreSQL
psql -U postgres -c "CREATE DATABASE blackhole;"

# Create .env file
cp ../.env.example ../.env
# Edit DATABASE_URL in .env

# Run migrations
cargo install sqlx-cli
sqlx migrate run

# Start backend
cargo run --release
```

#### Frontend Setup

```bash
cd frontend

# Install dependencies
npm install

# Start development server
npm run dev
```

## Testing Email Delivery

### Using MailHog (Included in Docker)

Send a test email via SMTP:

```bash
# Install swaks (SMTP testing tool)
# On Ubuntu/Debian:
sudo apt-get install swaks

# On macOS:
brew install swaks

# Send test email
swaks --to support@blackhole.dev \
  --from test@example.com \
  --server localhost:2525 \
  --header "Subject: Test Email" \
  --body "This is a test email"
```

### Using Resend (Production)

1. Sign up at https://resend.com (100 free emails/day)
2. Get your API key
3. Add it to `.env`:
   ```
   RESEND_API_KEY=re_your_key_here
   ```

## Production Deployment

### Option 1: Railway.app (Easiest)

1. Install Railway CLI:
   ```bash
   npm install -g @railway/cli
   ```

2. Login and initialize:
   ```bash
   railway login
   railway init
   ```

3. Add PostgreSQL:
   ```bash
   railway add postgres
   ```

4. Set environment variables:
   ```bash
   railway variables set RESEND_API_KEY=your_key
   ```

5. Deploy:
   ```bash
   railway up
   ```

### Option 2: Render.com

1. Create a new Web Service
2. Connect your GitHub repository
3. Add PostgreSQL database
4. Set environment variables:
   - `DATABASE_URL` (automatically set by Render)
   - `RESEND_API_KEY` (your API key)
5. Deploy!

### Option 3: DigitalOcean App Platform

1. Create new app from GitHub
2. Add PostgreSQL database
3. Configure environment variables
4. Deploy

### Option 4: VPS (Full Control)

```bash
# On your VPS (Ubuntu 22.04+)
sudo apt update
sudo apt install docker docker-compose

# Clone repository
git clone <your-repo-url>
cd blackhole-v2

# Configure environment
cp .env.example .env
nano .env  # Add your settings

# Start with docker-compose
docker-compose -f docker-compose.prod.yml up -d

# Setup nginx reverse proxy (optional)
sudo apt install nginx
# Configure nginx to proxy to localhost:8080
```

## Environment Variables

### Required
- `DATABASE_URL`: PostgreSQL connection string
- `RESEND_API_KEY`: Your Resend API key (for sending emails)

### Optional
- `SMTP_PORT`: SMTP server port (default: 2525)
- `HOST`: Server host (default: 0.0.0.0)
- `PORT`: HTTP server port (default: 8080)
- `JWT_SECRET`: Secret for JWT tokens
- `RUST_LOG`: Logging level (default: info)

## Monitoring & Maintenance

### View Logs

```bash
# All services
docker-compose logs -f

# Specific service
docker-compose logs -f backend
docker-compose logs -f frontend
```

### Database Backup

```bash
# Backup
docker-compose exec postgres pg_dump -U postgres blackhole > backup.sql

# Restore
docker-compose exec -T postgres psql -U postgres blackhole < backup.sql
```

### Update Application

```bash
git pull
docker-compose build
docker-compose up -d
```

## Performance Tuning

### PostgreSQL
Edit `docker-compose.yml` to add:
```yaml
environment:
  - POSTGRES_SHARED_BUFFERS=256MB
  - POSTGRES_MAX_CONNECTIONS=100
```

### Backend
Increase worker threads in `main.rs`:
```rust
tokio::runtime::Builder::new_multi_thread()
    .worker_threads(8)
    .build()
```

## Troubleshooting

### Backend won't start
- Check DATABASE_URL is correct
- Ensure PostgreSQL is running
- Check port 8080 is not in use

### Frontend can't connect to API
- Check VITE_API_URL in frontend/.env
- Ensure CORS is configured correctly
- Check browser console for errors

### Emails not arriving
- Check SMTP port 2525 is open
- Verify MailHog is running (docker-compose ps)
- Check backend logs for errors

### Database connection errors
- Ensure PostgreSQL is running
- Check DATABASE_URL format
- Verify database exists

## Security Checklist for Production

- [ ] Change JWT_SECRET in .env
- [ ] Use strong database password
- [ ] Enable HTTPS (use Caddy or nginx with Let's Encrypt)
- [ ] Set up firewall rules
- [ ] Configure rate limiting
- [ ] Enable database backups
- [ ] Use environment-specific .env files
- [ ] Review and restrict CORS origins
- [ ] Enable audit logging
- [ ] Set up monitoring (Sentry, DataDog, etc.)

## Support

For issues, please check:
1. Application logs
2. Database connectivity
3. Environment variables
4. GitHub Issues

## Next Steps

After deployment:
1. Test email flow end-to-end
2. Configure rules for your use case
3. Set up webhooks if needed
4. Customize branding
5. Train your team

🕳️ Welcome to Blackhole Mail!
