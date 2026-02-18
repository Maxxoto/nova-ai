#!/bin/bash
cd /Users/automation-mac/Documents/Research/nova-api

if [ ! -f nova.pid ]; then
    echo "❌ Nova Agent is NOT running (no PID file)"
    exit 1
fi

PID=$(cat nova.pid)

if ps -p $PID > /dev/null 2>&1; then
    echo "✅ Nova Agent is RUNNING"
    echo "   PID: $PID"
    echo "   Uptime: $(ps -p $PID -o etime= | tr -d ' ')"

    # Show latest log file
    LATEST_LOG=$(ls -t logs/nova-*.log 2>/dev/null | head -1)
    if [ -n "$LATEST_LOG" ]; then
        echo "   Log: $LATEST_LOG"
        echo "   Recent logs:"
        tail -n 5 "$LATEST_LOG" | sed 's/^/   /'
    fi
else
    echo "❌ Nova Agent is NOT running (stale PID file)"
    echo "   Removing stale PID file..."
    rm nova.pid
    exit 1
fi
