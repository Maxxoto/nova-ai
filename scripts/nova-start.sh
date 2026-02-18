#!/bin/bash

# Create logs directory if not exists
mkdir -p logs

# Get today's date for log file
LOG_DATE=$(date +%Y-%m-%d)
LOG_FILE="logs/nova-$LOG_DATE.log"

# Start bot in background with nohup
nohup python3 -m src.app.main >> "$LOG_FILE" 2>&1 &

# Save PID
echo $! > nova.pid
echo "✅ Nova Agent started (PID: $(cat nova.pid))"
echo "📝 Log file: $LOG_FILE"
echo "💝 Watch logs: ./scripts/nova-watch.sh"
