#!/usr/bin/env bash
set -Eeuo pipefail

# ───────────────────────────────────────────────────────────
# Installs the Palworld dedicated server into /home/steam/palworld.
# Used directly as the entrypoint of the `installer` image target
# (e.g. as a Kubernetes init container), and sourced by entrypoint.sh
# for the standalone `palworld` image.
# ───────────────────────────────────────────────────────────

echo "──────────────────────────────────────────────────────────"
echo "📦 Palworld Installer - $(date)"
echo "──────────────────────────────────────────────────────────"

if [ ! -d "/home/steam/palworld" ]; then
    echo "⚠️ Directory /home/steam/palworld does not exist. Creating..."
    mkdir -p /home/steam/palworld/logs
fi

echo "🔄 Updating ownership to match user..."
sudo chown -R "$(id -u):$(id -g)" /home/steam/palworld 2>/dev/null || true

echo "🧹 Cleaning up cache..."
rm -rf /home/steam/.cache

echo "📦 Ensuring necessary directories exist..."
mkdir -p /home/steam/palworld
mkdir -p /home/steam/palworld/logs

echo "🔧 Running SteamCMD to ensure dependencies are up to date..."
steamcmd +quit

INSTALL_MAX_ATTEMPTS="${INSTALL_MAX_ATTEMPTS:-5}"
INSTALL_RETRY_DELAY="${INSTALL_RETRY_DELAY:-10}"
# SteamCMD occasionally deadlocks mid-verify ("stalled cross-thread pipe") and
# never returns; `palworld install` has no internal timeout, so bound each
# attempt ourselves and hard-kill the whole process group if it stalls.
INSTALL_ATTEMPT_TIMEOUT="${INSTALL_ATTEMPT_TIMEOUT:-1800}"

CURRENT_PID=""

terminate_current_attempt() {
    echo "🛑 Caught termination signal, stopping SteamCMD..."
    if [ -n "$CURRENT_PID" ]; then
        kill -TERM "-$CURRENT_PID" 2>/dev/null || true
        sleep 2
        kill -KILL "-$CURRENT_PID" 2>/dev/null || true
    fi
    exit 143
}
trap terminate_current_attempt SIGTERM SIGINT

clear_steamcmd_cache() {
    echo "🧽 Clearing SteamCMD app/depot cache to avoid re-using a stalled or corrupt manifest..."
    rm -rf /home/steam/Steam/appcache /home/steam/Steam/depotcache
}

# Runs `palworld install` in its own session/process group so a hung SteamCMD
# subprocess can be killed as a unit, and races it against a watchdog timer.
run_install_attempt() {
    setsid palworld install &
    local pid=$!
    CURRENT_PID="$pid"

    (
        sleep "$INSTALL_ATTEMPT_TIMEOUT"
        if kill -0 "$pid" 2>/dev/null; then
            echo "⏱️ Install attempt exceeded ${INSTALL_ATTEMPT_TIMEOUT}s (SteamCMD appears stalled), killing it..."
            kill -TERM "-$pid" 2>/dev/null || true
            sleep 5
            kill -KILL "-$pid" 2>/dev/null || true
        fi
    ) &
    local watchdog=$!

    local status=0
    wait "$pid" || status=$?

    kill "$watchdog" 2>/dev/null || true
    wait "$watchdog" 2>/dev/null || true
    CURRENT_PID=""
    return "$status"
}

attempt=1
until run_install_attempt; do
    if [ "$attempt" -ge "$INSTALL_MAX_ATTEMPTS" ]; then
        echo "❌ Install failed after $attempt attempts, giving up."
        exit 1
    fi
    echo "⚠️ Install attempt $attempt/$INSTALL_MAX_ATTEMPTS failed, retrying in ${INSTALL_RETRY_DELAY}s..."
    clear_steamcmd_cache
    sleep "$INSTALL_RETRY_DELAY"
    attempt=$((attempt + 1))
done

echo "✅ Install complete."
