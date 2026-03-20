#!/bin/bash
cd /Users/automation-mac/Documents/Research/nova-api

if [ ! -f nova.pid ]; then
    echo "❌ Nova Agent is not running"
    exit 1
fi

# Get latest log file
LATEST_LOG=$(ls -t logs/nova-*.log 2>/dev/null | head -1)

if [ -z "$LATEST_LOG" ]; then
    echo "❌ No log files found"
    exit 1
fi

echo "📝 Watching: $LATEST_LOG"
echo "Press Ctrl+C to stop"
echo "---"
tail -f "$LATEST_LOG"
