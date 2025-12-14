#!/bin/bash
export LLMSPELL_STORAGE_DATABASE_PATH=/tmp/llmspell_test_concurrent.db
rm -f $LLMSPELL_STORAGE_DATABASE_PATH $LLMSPELL_STORAGE_DATABASE_PATH-wal $LLMSPELL_STORAGE_DATABASE_PATH-shm

# Start web server (Process A)
echo "Starting web server..."
./target/debug/llmspell web start --port 3333 > /tmp/web.log 2>&1 &
WEB_PID=$!

# Wait for it to initialize
sleep 5

# Check if web server is okay
if ! kill -0 $WEB_PID; then
    echo "Web server failed to start"
    cat /tmp/web.log
    exit 1
fi

echo "Web server running with PID $WEB_PID"

# Run CLI command with state enabled (Process B)
# We use -p state to ensure DB connection is made
echo "Running CLI command..."
if ./target/debug/llmspell run examples/script-users/getting-started/00-hello-world.lua -p state; then
    echo "CLI command succeeded"
    EXIT_CODE=0
else
    echo "CLI command failed"
    EXIT_CODE=1
fi

echo "CLI command exited with $EXIT_CODE"

kill $WEB_PID
wait $WEB_PID 2>/dev/null

if [ $EXIT_CODE -eq 0 ]; then
    echo "SUCCESS: Concurrent access worked"
else
    echo "FAILURE: Concurrent access failed"
fi
