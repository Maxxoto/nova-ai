#!/bin/bash
cd /Users/automation-mac/Documents/Research/nova-api

if [ ! -f nova.pid ]; then
    echo "❌ No PID file found - bot may not be running"
    exit 1
fi

PID=$(cat nova.pid)
echo "🛑 Stopping Nova Agent (PID: $PID)..."

kill $PID 2>/dev/null

# Wait up to 5 seconds for graceful shutdown
for i in {1..5}; do
    if ! ps -p $PID > /dev/null 2>&1; then
        rm nova.pid
        echo "✅ Nova Agent stopped"
        exit 0
    fi
    sleep 1
done

# Force kill if still running
echo "⚠️  Force killing..."
kill -9 $PID 2>/dev/null
rm nova.pid
echo "✅ Nova Agent force killed"
