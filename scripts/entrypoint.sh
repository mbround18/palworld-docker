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

# ───────────────────────────────────────────────────────────
# Install / validate
# ───────────────────────────────────────────────────────────
# Safe to call even if a k8s/compose init container using the
# `installer` image target already installed the server into the
# shared volume; SteamCMD validates and skips files that are current.
#
# Run in the background and `wait` on it (rather than a plain foreground
# call) so that PID 1 stays responsive to SIGTERM/SIGINT the whole time and
# `docker stop` doesn't have to wait out the full grace period + SIGKILL if
# install.sh is still running (or stuck).
INSTALL_PID=""
trap 'echo "🛑 Received termination signal during install, stopping..."; [ -n "$INSTALL_PID" ] && kill -TERM "$INSTALL_PID" 2>/dev/null; wait "$INSTALL_PID" 2>/dev/null; exit 143' SIGTERM SIGINT

/install.sh &
INSTALL_PID=$!
wait "$INSTALL_PID"
trap - SIGTERM SIGINT

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

# Set trap to run cleanup and kill the monitor process if needed.
# Deliberately not trapping ERR here: `kill $MONITOR_PID` is expected to
# fail once the monitor is already gone, and under `set -e` an ERR trap
# would re-fire this same handler, running `palworld stop` a second time.
trap 'palworld stop; kill $MONITOR_PID 2>/dev/null || true' SIGTERM SIGINT

# Wait for the monitor process to exit
wait $MONITOR_PID
