**How to Start Kernel Daemon for Testing:**
```bash

# Kill any existing kernel
pkill -f "llmspell.*kernel" || true

rm -rf /tmp/llmspell-test

mkdir -p /tmp/llmspell-test

# Start with full tracing
./target/debug/llmspell kernel start \
  --daemon \
  --trace trace \
  --port 8888 \
  --connection-file /tmp/llmspell-test/kernel.json \
  --log-file /tmp/llmspell-test/kernel.log \
  --pid-file /tmp/llmspell-test/kernel.pid \
  --idle-timeout 0

# Check connection file
cat /tmp/llmspell-test/kernel.json

# Test with jupyter_client
python3 tests/python/test_jupyter_client.py