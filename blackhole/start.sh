#!/bin/bash

set -e

echo "🕳️  Blackhole Mail - Quick Start Setup"
echo "========================================"
echo ""

# Check for Docker
if ! command -v docker &> /dev/null; then
    echo "❌ Docker is not installed. Please install Docker first."
    echo "   Visit: https://docs.docker.com/get-docker/"
    exit 1
fi

if ! command -v docker-compose &> /dev/null && ! docker compose version &> /dev/null; then
    echo "❌ Docker Compose is not installed. Please install Docker Compose first."
    exit 1
fi

echo "✅ Docker found"
echo ""

# Create .env if it doesn't exist
if [ ! -f .env ]; then
    echo "📝 Creating .env file..."
    cp .env.example .env
    echo "✅ .env file created"
    echo ""
    echo "⚠️  IMPORTANT: Edit .env and add your RESEND_API_KEY for production email sending"
    echo "   (Optional for local testing with MailHog)"
    echo ""
else
    echo "✅ .env file already exists"
    echo ""
fi

# Stop any existing containers
echo "🛑 Stopping existing containers..."
docker-compose down 2>/dev/null || true
echo ""

# Pull images
echo "📦 Pulling required images..."
docker-compose pull postgres mailhog
echo ""

# Build and start services
echo "🚀 Building and starting services..."
docker-compose up -d --build
echo ""

# Wait for services to be healthy
echo "⏳ Waiting for services to be ready..."
sleep 10

# Check if services are running
if docker-compose ps | grep -q "Up"; then
    echo ""
    echo "✅ Blackhole Mail is running!"
    echo ""
    echo "📍 Access Points:"
    echo "   Frontend:  http://localhost:3000"
    echo "   API:       http://localhost:8080"
    echo "   MailHog:   http://localhost:8025"
    echo "   Database:  localhost:5432"
    echo ""
    echo "🔐 Default Credentials:"
    echo "   Email:     admin@blackhole.dev"
    echo "   Password:  admin123"
    echo ""
    echo "📧 Test Email Sending:"
    echo "   Send to:   support@blackhole.dev"
    echo "   SMTP:      localhost:2525"
    echo ""
    echo "📝 View Logs:"
    echo "   docker-compose logs -f"
    echo ""
    echo "🛑 Stop Services:"
    echo "   docker-compose down"
    echo ""
    echo "🕳️  Happy emailing!"
else
    echo ""
    echo "❌ Some services failed to start. Check logs with:"
    echo "   docker-compose logs"
    exit 1
fi
