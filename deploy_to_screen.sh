#!/bin/bash
set -e

SESSION="27088"
PROJECT_ROOT="$(cd "$(dirname "$0")" && pwd)"
BINARY_DIR="$PROJECT_ROOT/target/release"

echo "========================================="
echo "  Building workspace (cargo build --release)"
echo "========================================="
cd "$PROJECT_ROOT"
cargo build --release
echo ""

echo "========================================="
echo "  Deploying to screen session: $SESSION"
echo "========================================="

# Create screen session if it doesn't exist
if ! screen -S "$SESSION" -Q select . > /dev/null 2>&1; then
    echo "Creating new screen session: $SESSION"
    screen -dmS "$SESSION"
    sleep 1
fi

# Kill any existing service windows (keep window 0 which is bash)
for win in db-manager ai-core gateway-discord web-dashboard; do
    screen -S "$SESSION" -p "$win" -X kill 2>/dev/null || true
done
sleep 1

# Each service runs from its own directory so .env is picked up

# Window 1: db-manager
echo "[1/4] Starting db-manager..."
screen -S "$SESSION" -X screen -t db-manager bash -c "cd $PROJECT_ROOT/services/db-manager && exec $BINARY_DIR/db-manager"
echo "  -> Waiting 3s..."
sleep 3

# Window 2: ai-core
echo "[2/4] Starting ai-core..."
screen -S "$SESSION" -X screen -t ai-core bash -c "cd $PROJECT_ROOT/services/ai-core && exec $BINARY_DIR/ai-core"
echo "  -> Waiting 3s..."
sleep 3

# Window 3: gateway-discord
echo "[3/4] Starting gateway-discord..."
screen -S "$SESSION" -X screen -t gateway-discord bash -c "cd $PROJECT_ROOT/services/gateway-discord && exec $BINARY_DIR/gateway-discord"
sleep 2

# Window 4: web-dashboard
echo "[4/4] Starting web-dashboard..."
screen -S "$SESSION" -X screen -t web-dashboard bash -c "cd $PROJECT_ROOT/services/web-dashboard && exec $BINARY_DIR/web-dashboard"
sleep 2

echo ""
echo "========================================="
echo "  All services deployed!"
echo "========================================="
echo ""
echo "Current windows:"
screen -S "$SESSION" -Q windows 2>&1 || true
echo ""
