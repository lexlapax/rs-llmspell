#!/bin/bash
set -e

# Setup
TEST_CONFIG="test_e2e_config.toml"
echo 'version = "0.1"' > $TEST_CONFIG
echo '[debug]' >> $TEST_CONFIG
echo 'enabled = true' >> $TEST_CONFIG

SERVER_PID=""

cleanup() {
    if [ -n "$SERVER_PID" ]; then
        kill $SERVER_PID 2>/dev/null || true
    fi
    rm -f $TEST_CONFIG server.log server_restart.log response.txt
}
trap cleanup EXIT

echo "1. Starting Server..."
# Start server in background
cargo run --bin llmspell -- web start --config $TEST_CONFIG > server.log 2>&1 &
SERVER_PID=$!

echo "Waiting for server to start..."
# Poll for port (allow long time for compilation)
for i in {1..300}; do
    if curl -s http://localhost:3000/health > /dev/null; then
        echo "Server is up!"
        break
    fi
    sleep 1
done

echo "2. Verifying Initial State..."
# Capture response for debugging
curl -v -s -H "X-API-Key: dev-key-123" http://localhost:3000/api/config/source > response.txt 2>&1
INITIAL_VAL=$(cat response.txt | grep "version")
if [[ "$INITIAL_VAL" != *"0.1"* ]]; then
    echo "FAILED: Initial version mismatch. Got: $INITIAL_VAL"
    echo "--- Response Content ---"
    cat response.txt
    echo "--- Server Log ---"
    cat server.log
    exit 1
fi

echo "3. Updating Config (Valid)..."
# Update version to 0.2
curl -s -X PUT http://localhost:3000/api/config/source \
    -H "X-API-Key: dev-key-123" \
    -H "Content-Type: text/plain" \
    --data '
version = "0.2"
[debug]
enabled = true
'

echo "4. Testing Corruption Safety (Invalid TOML)..."
HTTP_CODE=$(curl -s -o /dev/null -w "%{http_code}" -X PUT http://localhost:3000/api/config/source \
    -H "X-API-Key: dev-key-123" \
    -H "Content-Type: text/plain" \
    --data '
version = "0.3"
[debug
enabled = broken
')

if [ "$HTTP_CODE" != "400" ]; then
    echo "FAILED: Invalid TOML was not rejected (HTTP $HTTP_CODE)"
    exit 1
else
    echo "PASSED: Invalid TOML rejected with 400"
fi

echo "5. Testing Restart API..."
curl -X POST http://localhost:3000/api/config/restart -H "X-API-Key: dev-key-123"
# Wait for it to die
wait $SERVER_PID 2>/dev/null || true
SERVER_PID=""

# Start again (simulating process manager restart)
cargo run --bin llmspell -- web start --config $TEST_CONFIG > server_restart.log 2>&1 &
SERVER_PID=$!

echo "Waiting for server to restart..."
for i in {1..300}; do
    if curl -s http://localhost:3000/health > /dev/null; then
        echo "Server is up!"
        break
    fi
    sleep 1
done

echo "6. Verifying Persistence..."
FINAL_VAL=$(curl -s -H "X-API-Key: dev-key-123" http://localhost:3000/api/config/source | grep "version")
if [[ "$FINAL_VAL" != *"0.2"* ]]; then
    echo "FAILED: Config update did not persist. Got: $FINAL_VAL"
    echo "--- Server Restart Log ---"
    cat server_restart.log
    exit 1
fi

echo "✅ SUCCESS: End-to-End Persistence and Safety Verified"
