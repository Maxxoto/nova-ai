#!/bin/bash

# Create logs directory if not exists
mkdir -p logs

# Get today's date for log file
LOG_DATE=$(date +%Y-%m-%d)
LOG_FILE="logs/nova-$LOG_DATE.log"

# Start bot in background with nohup

# Activate virtual environment
if [ -f "venv/bin/activate" ]; then
    source venv/bin/activate
elif [ -f ".venv/bin/activate" ]; then
    source .venv/bin/activate
else
    echo "❌ Error: Virtual environment not found (tried venv/ and .venv/)"
    exit 1
fi

nohup python -m src.app.main >> "$LOG_FILE" 2>&1 &

# Save PID
echo $! > nova.pid
echo "✅ Nova Agent started (PID: $(cat nova.pid))"
echo "📝 Log file: $LOG_FILE"
echo "💝 Watch logs: ./scripts/nova-watch.sh"
