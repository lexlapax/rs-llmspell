-- multimodal-stub.lua
-- Demonstrates multimodal type structures (Phase 1 - Types only, no actual processing)
-- Full multimodal support coming in later phases

print("=== Multimodal Types Demo (Stub) ===\n")

-- Example of how multimodal content will be structured
local multimodal_examples = {
    -- Image example
    {
        type = "image",
        content = {
            format = "jpeg",
            width = 1920,
            height = 1080,
            size_bytes = 2048576,  -- 2MB
            caption = "A beautiful sunset over the mountains"
        }
    },
    
    -- Audio example
    {
        type = "audio", 
        content = {
            format = "mp3",
            duration_seconds = 180,  -- 3 minutes
            bitrate = 128000,       -- 128 kbps
            size_bytes = 2880000,   -- ~2.8MB
            transcript = "This is a sample audio transcript..."
        }
    },
    
    -- Video example
    {
        type = "video",
        content = {
            format = "mp4",
            width = 1280,
            height = 720,
            duration_seconds = 60,
            fps = 30,
            size_bytes = 10485760,  -- 10MB
            description = "Tutorial video showing LLMSpell usage"
        }
    },
    
    -- Binary data example
    {
        type = "binary",
        content = {
            mime_type = "application/pdf",
            size_bytes = 524288,  -- 512KB
            encoding = "base64",
            preview = "PDF document containing technical specifications"
        }
    }
}

-- Display information about each media type
for i, media in ipairs(multimodal_examples) do
    print(string.format("Media #%d: %s", i, media.type:upper()))
    
    if media.type == "image" then
        local img = media.content
        print(string.format("  Format: %s", img.format))
        print(string.format("  Dimensions: %dx%d", img.width, img.height))
        print(string.format("  Size: %.2f MB", img.size_bytes / 1048576))
        if img.caption then
            print(string.format("  Caption: %s", img.caption))
        end
        
    elseif media.type == "audio" then
        local audio = media.content
        print(string.format("  Format: %s", audio.format))
        print(string.format("  Duration: %d seconds", audio.duration_seconds))
        print(string.format("  Bitrate: %d kbps", audio.bitrate / 1000))
        print(string.format("  Size: %.2f MB", audio.size_bytes / 1048576))
        
    elseif media.type == "video" then
        local video = media.content
        print(string.format("  Format: %s", video.format))
        print(string.format("  Resolution: %dx%d @ %d fps", video.width, video.height, video.fps))
        print(string.format("  Duration: %d seconds", video.duration_seconds))
        print(string.format("  Size: %.2f MB", video.size_bytes / 1048576))
        
    elseif media.type == "binary" then
        local binary = media.content
        print(string.format("  MIME Type: %s", binary.mime_type))
        print(string.format("  Size: %.2f KB", binary.size_bytes / 1024))
        print(string.format("  Encoding: %s", binary.encoding))
    end
    
    print()  -- Empty line
end

-- Demonstrate size validation
print("=== Size Validation Examples ===\n")

local size_limits = {
    image = 104857600,     -- 100MB
    audio = 524288000,     -- 500MB  
    video = 5368709120,    -- 5GB
    binary = 104857600     -- 100MB
}

local function validate_media_size(media_type, size_bytes)
    local limit = size_limits[media_type]
    if not limit then
        return false, "Unknown media type"
    end
    
    if size_bytes > limit then
        return false, string.format(
            "%s size %.2f MB exceeds limit of %.2f MB",
            media_type,
            size_bytes / 1048576,
            limit / 1048576
        )
    end
    
    return true, "Size is within limits"
end

-- Test size validation
local test_cases = {
    { type = "image", size = 50 * 1048576 },    -- 50MB - OK
    { type = "image", size = 150 * 1048576 },   -- 150MB - Too large
    { type = "video", size = 1024 * 1048576 },  -- 1GB - OK
    { type = "video", size = 6144 * 1048576 },  -- 6GB - Too large
}

for _, test in ipairs(test_cases) do
    local valid, message = validate_media_size(test.type, test.size)
    print(string.format(
        "%s %.0f MB: %s - %s",
        test.type,
        test.size / 1048576,
        valid and "VALID" or "INVALID",
        message
    ))
end

-- Future API preview (not functional in Phase 1)
print("\n=== Future Multimodal API (Preview) ===\n")
print([[
-- This is how multimodal content will work in future phases:

local agent = Agent.createAsync({
    name = "vision-agent",
    supports_multimodal = true
})

local response = agent:execute({
    text = "What's in this image?",
    media = {
        {
            type = "image",
            data = image_data,
            format = "jpeg"
        }
    }
})

print(response.text)  -- "I can see a sunset over mountains..."
]])

-- Return summary
return {
    demo_complete = true,
    media_types_shown = #multimodal_examples,
    validation_tests = #test_cases,
    note = "This is a Phase 1 stub - full multimodal support coming in later phases"
}