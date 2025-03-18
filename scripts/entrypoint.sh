#!/usr/bin/env bash
set -Eeuo pipefail

# ───────────────────────────────────────────────────────────
# Welcome to the Palworld Docker container
# If you are modifying this script please check contributors guide! :)
# ───────────────────────────────────────────────────────────

echo "──────────────────────────────────────────────────────────"
echo "🚀 Palworld Docker - $(date)"
echo "──────────────────────────────────────────────────────────"

# System Info
echo "🔹 Hostname: $(hostname)"
echo "🔹 Kernel: $(uname -r)"
echo "🔹 OS: $(grep PRETTY_NAME /etc/os-release | cut -d= -f2 | tr -d '\"')"
echo "🔹 CPU: $(lscpu | grep 'Model name' | cut -d: -f2 | sed 's/^ *//')"
echo "🔹 Memory: $(free -h | awk '/^Mem:/ {print $2}')"
echo "🔹 Disk Space: $(df -h / | awk 'NR==2 {print $4}')"
echo "──────────────────────────────────────────────────────────"

# User & Permission Check
echo "👤 Running as user: $(whoami) (UID: $(id -u), GID: $(id -g))"
echo "👥 Groups: $(id -Gn)"

# Directory checks
if [ ! -d "/home/steam/palworld" ]; then
    echo "⚠️ Directory /home/steam/palworld does not exist. Creating..."
    mkdir -p /home/steam/palworld/logs
fi

# Permission check
echo "🔍 Checking permissions for /home/steam/palworld..."
ls -ld /home/steam/palworld

echo "🔄 Updating ownership to match user..."
sudo chown -R "$(id -u):$(id -g)" /home/steam/palworld 2>/dev/null || true

# ───────────────────────────────────────────────────────────
# Setup and Initialization
# ───────────────────────────────────────────────────────────

echo "🧹 Cleaning up cache..."
rm -rf /home/steam/.cache

echo "📦 Ensuring necessary directories exist..."
mkdir -p /home/steam/palworld
mkdir -p /home/steam/palworld/logs

echo "🔧 Running SteamCMD to ensure dependencies are up to date..."
steamcmd +quit

# ───────────────────────────────────────────────────────────
# Install (if necessary)
# ───────────────────────────────────────────────────────────
palworld install

# ───────────────────────────────────────────────────────────
# Start the Palworld Server
# ───────────────────────────────────────────────────────────
echo "🔥 Starting Palworld server..."
palworld start

# ───────────────────────────────────────────────────────────
# Monitor the Server
# ───────────────────────────────────────────────────────────
echo "📡 Monitoring Palworld server logs..."
# Start the monitor in the background
palworld monitor &
MONITOR_PID=$!

# Set trap to run cleanup and kill the monitor process if needed
trap 'palworld stop; kill $MONITOR_PID' SIGTERM SIGINT ERR

# Wait for the monitor process to exit
wait $MONITOR_PID
