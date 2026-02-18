#!/bin/bash

# Add cron job to rotate logs daily at 3 AM
CRON_JOB="0 3 * * * /Users/automation-mac/Documents/Research/nova-api/scripts/rotate-logs.sh >> /Users/automation-mac/Documents/Research/nova-api/logs/rotate.log 2>&1"

# Check if cron job already exists
if crontab -l 2>/dev/null | grep -q "rotate-logs.sh"; then
    echo "⚠️  Cron job already exists"
    echo "Current cron jobs:"
    crontab -l | grep "rotate-logs.sh"
else
    # Add to crontab
    (crontab -l 2>/dev/null; echo "$CRON_JOB") | crontab -
    echo "✅ Cron job added - logs will be cleaned daily at 3 AM"
    echo "View with: crontab -l"
fi
