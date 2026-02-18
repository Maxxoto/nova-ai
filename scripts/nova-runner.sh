#!/bin/bash
cd /Users/automation-mac/Documents/Research/nova-api

while true; do
    # Create logs directory
    mkdir -p logs

    # Get today's log file
    LOG_DATE=$(date +%Y-%m-%d)
    LOG_FILE="logs/nova-$LOG_DATE.log"
    RUNNER_LOG="logs/runner.log"

    # Log start
    echo "[$(date)] Starting Nova Agent..." | tee -a "$RUNNER_LOG"

    # Start bot and wait for it
    python -m src.app.main 2>&1 | tee -a "$LOG_FILE"
    EXIT_CODE=${PIPESTATUS[0]}

    # Log exit
    echo "[$(date)] Nova Agent exited with code: $EXIT_CODE" | tee -a "$RUNNER_LOG"

    # Clean up PID file if exists
    if [ -f nova.pid ]; then
        rm nova.pid
    fi

    # If it was a clean exit (Ctrl+C), don't restart
    if [ $EXIT_CODE -eq 130 ]; then
        echo "[$(date)] Clean exit - not restarting" | tee -a "$RUNNER_LOG"
        break
    fi

    # Wait a bit before restarting
    echo "[$(date)] Restarting in 5 seconds..." | tee -a "$RUNNER_LOG"
    sleep 5
done

echo "[$(date)] Nova Agent runner stopped"
