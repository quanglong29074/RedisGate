#!/bin/bash

# RedisGate Startup Script
set -e

echo "Starting RedisGate application..."

# Start frontend development server in background
echo "Starting frontend development server..."
cd /app/frontend-redis
bun run dev --host 0.0.0.0 --port 3000 &
FRONTEND_PID=$!

# Go back to root directory
cd /app

# Wait a bit for frontend to start
sleep 5

echo "Starting backend server..."
/usr/local/bin/redisgate &
BACKEND_PID=$!

# Function to handle cleanup on exit
cleanup() {
    echo "Shutting down services..."
    kill $FRONTEND_PID $BACKEND_PID 2>/dev/null || true
    wait $FRONTEND_PID $BACKEND_PID 2>/dev/null || true
    echo "Services stopped"
}

# Set trap to cleanup on script exit
trap cleanup EXIT INT TERM

# Wait for both processes
wait $FRONTEND_PID $BACKEND_PID
