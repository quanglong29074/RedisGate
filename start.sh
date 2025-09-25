#!/bin/bash

# RedisGate Unified Startup Script
set -e

echo "Starting RedisGate application (Backend + Frontend on port 3000)..."

# Function to handle cleanup on exit
cleanup() {
    echo "Shutting down RedisGate..."
    kill $BACKEND_PID 2>/dev/null || true
    wait $BACKEND_PID 2>/dev/null || true
    echo "RedisGate stopped"
}

# Set trap to cleanup on script exit
trap cleanup EXIT INT TERM

echo "Starting unified server (API + Static Files)..."
/usr/local/bin/redisgate &
BACKEND_PID=$!

echo "RedisGate is running on http://0.0.0.0:3000"
echo "- API endpoints: /api/*, /auth/*, /redis/*, /health, /version, /stats"
echo "- Frontend: All other routes serve static files"

# Wait for the process
wait $BACKEND_PID