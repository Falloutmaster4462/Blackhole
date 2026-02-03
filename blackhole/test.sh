#!/bin/bash

set -e

echo "🧪 Blackhole Mail - Testing Suite"
echo "=================================="
echo ""

API_URL="http://localhost:8080"
SMTP_HOST="localhost"
SMTP_PORT="2525"

# Color codes
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Test counter
TESTS_RUN=0
TESTS_PASSED=0
TESTS_FAILED=0

run_test() {
    local test_name="$1"
    local test_command="$2"
    
    TESTS_RUN=$((TESTS_RUN + 1))
    echo -n "Testing: $test_name... "
    
    if eval "$test_command" &> /dev/null; then
        echo -e "${GREEN}✓ PASS${NC}"
        TESTS_PASSED=$((TESTS_PASSED + 1))
    else
        echo -e "${RED}✗ FAIL${NC}"
        TESTS_FAILED=$((TESTS_FAILED + 1))
    fi
}

echo "1️⃣  Testing API Endpoints"
echo "-------------------------"

# Test API health
run_test "API health check" "curl -f $API_URL/api/stats/overview"

# Test authentication
run_test "User authentication" "curl -f -X POST $API_URL/api/auth/login -H 'Content-Type: application/json' -d '{\"email\":\"admin@blackhole.dev\",\"password\":\"admin123\"}'"

# Test email listing
run_test "List emails" "curl -f $API_URL/api/emails"

# Test inbox listing
run_test "List inboxes" "curl -f $API_URL/api/inboxes"

# Test rules listing
run_test "List rules" "curl -f $API_URL/api/rules"

# Test stats
run_test "Get statistics" "curl -f $API_URL/api/stats/overview"

echo ""
echo "2️⃣  Testing SMTP Server"
echo "----------------------"

# Check if nc (netcat) is available
if command -v nc &> /dev/null; then
    run_test "SMTP port open" "nc -zv $SMTP_HOST $SMTP_PORT"
else
    echo -e "${YELLOW}⚠ Skipping SMTP test (netcat not installed)${NC}"
fi

# Test email sending if swaks is available
if command -v swaks &> /dev/null; then
    run_test "Send test email" "swaks --to test@example.com --from sender@example.com --server $SMTP_HOST:$SMTP_PORT --header 'Subject: Test' --body 'Test email' --hide-all"
else
    echo -e "${YELLOW}⚠ Skipping email send test (swaks not installed)${NC}"
    echo "   Install with: sudo apt-get install swaks  OR  brew install swaks"
fi

echo ""
echo "3️⃣  Testing Database"
echo "-------------------"

# Test database connection via Docker
if docker-compose ps postgres | grep -q "Up"; then
    run_test "PostgreSQL running" "docker-compose exec -T postgres pg_isready -U postgres"
    run_test "Database exists" "docker-compose exec -T postgres psql -U postgres -lqt | grep -q blackhole"
else
    echo -e "${RED}✗ PostgreSQL container not running${NC}"
    TESTS_FAILED=$((TESTS_FAILED + 2))
fi

echo ""
echo "4️⃣  Testing Services"
echo "-------------------"

# Check Docker containers
run_test "Backend container" "docker-compose ps backend | grep -q 'Up'"
run_test "Frontend container" "docker-compose ps frontend | grep -q 'Up'"
run_test "MailHog container" "docker-compose ps mailhog | grep -q 'Up'"

echo ""
echo "5️⃣  Testing Frontend"
echo "-------------------"

run_test "Frontend accessible" "curl -f http://localhost:3000"

echo ""
echo "======================================"
echo "Test Results"
echo "======================================"
echo "Total Tests:  $TESTS_RUN"
echo -e "Passed:       ${GREEN}$TESTS_PASSED${NC}"
echo -e "Failed:       ${RED}$TESTS_FAILED${NC}"
echo ""

if [ $TESTS_FAILED -eq 0 ]; then
    echo -e "${GREEN}✅ All tests passed!${NC}"
    echo ""
    echo "🕳️  Blackhole Mail is working correctly!"
    exit 0
else
    echo -e "${RED}❌ Some tests failed${NC}"
    echo ""
    echo "Troubleshooting:"
    echo "1. Check if all services are running: docker-compose ps"
    echo "2. View logs: docker-compose logs"
    echo "3. Restart services: docker-compose restart"
    exit 1
fi
