# Nova Agent - Background Service Scripts

Collection of scripts for running Nova Agent in the background on Linux/macOS.

## 🚀 Quick Start

### Start with Auto-Restart (Recommended)
```bash
./scripts/nova-runner.sh &
```

### Or Start Simple (No Auto-Restart)
```bash
./scripts/nova-start.sh
```

## 📋 Available Scripts

### Basic Commands
| Script | Description |
|--------|-------------|
| `./scripts/nova-start.sh` | Start bot in background |
| `./scripts/nova-stop.sh` | Stop bot gracefully |
| `./scripts/nova-status.sh` | Check if bot is running |
| `./scripts/nova-watch.sh` | Watch logs in real-time |
| `./scripts/nova-restart.sh` | Restart bot |

### Advanced
| Script | Description |
|--------|-------------|
| `./scripts/nova-runner.sh` | Start with auto-restart on crash (background) |
| `./scripts/rotate-logs.sh` | Clean old logs (1-day retention) |
| `./scripts/cron-setup.sh` | Setup cron job for auto-cleanup |

## 📝 Usage Examples

### Check Status
```bash
./scripts/nova-status.sh
```

Output:
```
✅ Nova Agent is RUNNING
   PID: 12345
   Uptime: 2:30:15
   Log: logs/nova-2026-02-19.log
   Recent logs:
   2026-02-19 10:30:15 - INFO - Nova Agent is running!
```

### Watch Logs
```bash
./scripts/nova-watch.sh
```

### Start with Auto-Restart
```bash
# Start in background
nohup ./scripts/nova-runner.sh > logs/runner.log 2>&1 &

# Check runner status
tail -f logs/runner.log
```

### Stop Bot
```bash
./scripts/nova-stop.sh
```

## 📂 Directory Structure

```
nova-api/
├── logs/                          # Log files
│   ├── nova-2026-02-19.log        # Today's bot log
│   ├── nova-2026-02-18.log        # Yesterday's log (auto-deleted after 1 day)
│   ├── runner.log                 # Runner script log
│   └── rotate.log                 # Log rotation log
├── scripts/
│   ├── nova-start.sh              # Start bot
│   ├── nova-stop.sh               # Stop bot
│   ├── nova-status.sh             # Check status
│   ├── nova-watch.sh              # Watch logs
│   ├── nova-restart.sh            # Restart bot
│   ├── nova-runner.sh             # Auto-restart runner
│   ├── rotate-logs.sh             # Clean old logs
│   └── cron-setup.sh              # Setup cron job
└── nova.pid                       # Running process ID
```

## ⚙️ Features

### ✅ Auto-Restart on Crash
The `nova-runner.sh` script automatically restarts the bot if it crashes:
```bash
nohup ./scripts/nova-runner.sh > logs/runner.log 2>&1 &
```

### ✅ Daily Log Rotation
Logs are automatically named by date: `nova-YYYY-MM-DD.log`

### ✅ Auto-Cleanup (1-Day Retention)
Setup cron to auto-delete logs older than 1 day:
```bash
./scripts/cron-setup.sh
```

### ✅ PID Management
All scripts track the process ID to prevent multiple instances.

## 🔧 Troubleshooting

### Bot Won't Start
```bash
# Check if already running
./scripts/nova-status.sh

# Check logs
./scripts/nova-watch.sh
```

### Multiple Instances
```bash
# Kill all instances
ps aux | grep "src.app.main" | grep -v grep | awk '{print $2}' | xargs kill

# Start fresh
./scripts/nova-start.sh
```

### View All Logs
```bash
# Today's log
cat logs/nova-$(date +%Y-%m-%d).log

# All recent logs
ls -lt logs/nova-*.log | head -5
```

## 📊 Monitoring

### Check Disk Usage
```bash
du -sh logs/
```

### Count Log Files
```bash
ls -1 logs/nova-*.log | wc -l
```

### Find Errors
```bash
grep "ERROR" logs/nova-*.log
```

## 🔄 Cron Job

View cron jobs:
```bash
crontab -l
```

Remove cron job:
```bash
crontab -e
# Delete the line with rotate-logs.sh
```

## 💡 Tips

1. **Always use nova-runner.sh** for development (auto-restart)
2. **Check logs** when troubleshooting: `./scripts/nova-watch.sh`
3. **Monitor disk space** if running long-term
4. **Use nova-status.sh** to quickly check if bot is alive
5. **Logs rotate daily** - old logs auto-delete after 1 day

## 🛠️ Development Workflow

```bash
# Start with auto-restart
nohup ./scripts/nova-runner.sh > logs/runner.log 2>&1 &

# Make changes to code
vim src/app/main.py

# Restart
./scripts/nova-restart.sh

# Watch logs
./scripts/nova-watch.sh
```
