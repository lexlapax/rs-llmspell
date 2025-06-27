-- streaming-demo.lua
-- Demonstrates streaming capabilities (Phase 1 - placeholder implementation)

print("=== Streaming Demo ===\n")

-- Create a simple coroutine-based stream
local function create_stream()
    return coroutine.create(function()
        local messages = {
            "Starting stream...",
            "Processing data chunk 1",
            "Processing data chunk 2", 
            "Processing data chunk 3",
            "Stream complete!"
        }
        
        for i, msg in ipairs(messages) do
            -- Yield each message
            coroutine.yield({
                chunk_index = i - 1,
                content = msg,
                timestamp = os.time()
            })
            
            -- In real streaming, there would be async delays here
        end
    end)
end

-- Process the stream
local stream = create_stream()
local chunks = {}

print("Processing stream chunks:")
while true do
    local success, chunk = coroutine.resume(stream)
    if not success or coroutine.status(stream) == "dead" then
        break
    end
    
    if chunk then
        print(string.format("  [Chunk %d] %s", chunk.chunk_index, chunk.content))
        table.insert(chunks, chunk)
    end
end

print("\nStream processing complete!")

-- Using the Streaming API (if available)
if Streaming then
    print("\nStreaming API is available")
    
    -- Create a stream using the API
    local api_stream = Streaming.create(function()
        for i = 1, 3 do
            coroutine.yield("API chunk " .. i)
        end
    end)
    
    -- Collect all values
    -- Note: In Phase 1, this is a placeholder implementation
    print("Collecting stream values...")
end

-- Return results
return {
    chunk_count = #chunks,
    chunks = chunks,
    streaming_available = Streaming ~= nil
}