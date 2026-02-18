#!/bin/bash
cd /Users/automation-mac/Documents/Research/nova-api

echo "🗑️  Cleaning old logs (keeping last 1 day)..."

# Remove log files older than 1 day
find logs/ -name "nova-*.log" -mtime +1 -delete 2>/dev/null

echo "✅ Log cleanup complete"
