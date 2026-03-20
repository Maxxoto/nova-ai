#!/bin/bash
cd /Users/automation-mac/Documents/Research/nova-api

echo "🔄 Restarting Nova Agent..."

# Stop if running
if [ -f nova.pid ]; then
    ./scripts/nova-stop.sh
    sleep 2
fi

# Start again
./scripts/nova-start.sh
