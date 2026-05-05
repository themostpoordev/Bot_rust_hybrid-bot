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

# Check if screen session exists
if ! screen -S "$SESSION" -Q select . > /dev/null 2>&1; then
    echo "Error: screen session '$SESSION' not found."
    echo "Create it first with: screen -S $SESSION"
    exit 1
fi

# Kill any existing service windows (keep window 0 which is bash)
for win in db-manager ai-core gateway-discord gateway-line; do
    # Try to kill by title (if exists)
    screen -S "$SESSION" -p "$win" -X kill 2>/dev/null || true
done
sleep 1

# The key insight: `screen -S SESSION -X screen -t title command`
# runs `command` as the shell for that window.
# BUT: we need to cd to BINARY_DIR first.
# Better: use a small inline script per window.

# Window 1: db-manager
echo "[1/4] Starting db-manager..."
screen -S "$SESSION" -X screen -t db-manager bash -c "cd $BINARY_DIR && exec ./db-manager"
echo "  -> Waiting 3s for db-manager to start..."
sleep 3

# Window 2: ai-core
echo "[2/4] Starting ai-core..."
screen -S "$SESSION" -X screen -t ai-core bash -c "cd $BINARY_DIR && exec ./ai-core"
echo "  -> Waiting 3s for ai-core to start..."
sleep 3

# Window 3: gateway-discord
echo "[3/4] Starting gateway-discord..."
screen -S "$SESSION" -X screen -t gateway-discord bash -c "cd $BINARY_DIR && exec ./gateway-discord"

# Window 4: gateway-line
echo "[4/4] Starting gateway-line..."
screen -S "$SESSION" -X screen -t gateway-line bash -c "cd $BINARY_DIR && exec ./gateway-line"

echo ""
echo "========================================="
echo "  All services deployed!"
echo "========================================="
echo ""
echo "How to use:"
echo "  - Attach to session:  screen -r $SESSION"
echo "  - Switch windows:     Ctrl+A, then 0-9"
echo "  - Detach:            Ctrl+A, then d"
echo ""
echo "Current windows:"
screen -S "$SESSION" -Q windows 2>&1 || true
echo ""
