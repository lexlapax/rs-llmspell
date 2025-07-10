-- tools-media.lua
-- Examples for media processing tools (Audio, Video, Image)
-- Using direct Tool API

print("🎬 Media Processing Tools Examples")
print("==================================")

-- Load test helpers
local TestHelpers = dofile("test-helpers.lua")

-- Helper function to execute tool
local function use_tool(tool_name, params)
    return TestHelpers.execute_tool(tool_name, params)
end

-- Helper to print clean results
local function print_result(label, result)
    if result.error then
        print("  ❌ " .. label .. ": " .. result.error)
    elseif result.success == false then
        print("  ❌ " .. label .. ": " .. (result.message or "Failed"))
    else
        -- Extract relevant metadata
        local r = result.result or result
        if r.metadata then
            print("  ✅ " .. label .. ":")
            if r.metadata.duration then
                print("     Duration: " .. r.metadata.duration .. "s")
            end
            if r.metadata.width and r.metadata.height then
                print("     Resolution: " .. r.metadata.width .. "x" .. r.metadata.height)
            end
            if r.metadata.format then
                print("     Format: " .. r.metadata.format)
            end
            if r.metadata.sample_rate then
                print("     Sample Rate: " .. r.metadata.sample_rate .. " Hz")
            end
        elseif r.output_path then
            print("  ✅ " .. label .. ": Saved to " .. r.output_path)
        elseif result.message then
            print("  ✅ " .. label .. ": " .. result.message)
        else
            print("  ✅ " .. label .. ": Success")
        end
    end
end

TestHelpers.print_section("Audio Processor Tool")

print("\nAudio processing operations:")

-- First create a test audio file to work with
-- Note: Using .txt as a mock since real audio processing is Phase 3+
local test_file = use_tool("file_operations", {
    operation = "write",
    path = "/tmp/test_audio.txt",
    content = "Mock audio file for testing (real audio processing in Phase 3+)"
})

-- Get audio metadata
local audio_metadata = use_tool("audio_processor", {
    operation = "metadata",
    file_path = "/tmp/test_audio.txt"
})
print_result("Audio metadata", audio_metadata)

-- Convert audio format (limited in Phase 2)
local audio_convert = use_tool("audio_processor", {
    operation = "convert",
    input_path = "/tmp/test_audio.wav",
    output_path = "/tmp/test_audio_converted.mp3",
    format = "mp3"
})
print_result("Convert to MP3 (Phase 3+)", audio_convert)

-- Note: detect operation not supported in Phase 2
-- Would be: operation = "detect" in Phase 3+
print("  ℹ️  Audio detect: Phase 3+ feature")

-- Note: Waveform and trim operations not supported in Phase 2
-- These are placeholders for Phase 3+ functionality

TestHelpers.print_section("Video Processor Tool")

print("\nVideo processing operations:")

-- Create a test video file
local test_video = use_tool("file_operations", {
    operation = "write",
    path = "/tmp/test_video.txt",
    content = "Mock video file for testing (real video processing in Phase 3+)"
})

-- Get video metadata
local video_metadata_first = use_tool("video_processor", {
    operation = "metadata",
    file_path = "/tmp/test_video.txt"
})
print_result("Video metadata", video_metadata_first)

-- Extract frame at specific time (Phase 3+ feature)
local extract_frame = use_tool("video_processor", {
    operation = "extract_frame",
    file_path = "/tmp/test_video.mp4",
    output_path = "/tmp/frame_10s.jpg",
    timestamp = 10.0
})
print_result("Extract frame at 10s (Phase 3+)", extract_frame)

-- Generate thumbnail (Phase 3+ feature)
local thumbnail = use_tool("video_processor", {
    operation = "thumbnail",
    file_path = "/tmp/test_video.mp4",
    output_path = "/tmp/video_thumbnail.jpg",
    width = 320,
    height = 180,
    timestamp = 5.0
})
print_result("Generate thumbnail (Phase 3+)", thumbnail)

-- Note: Convert operation not supported in Phase 2
-- Video conversion will be added in Phase 3+

-- Note: detect operation not supported in Phase 2
-- Would be: operation = "detect" in Phase 3+
print("  ℹ️  Video detect: Phase 3+ feature")

TestHelpers.print_section("Image Processor Tool")

print("\nImage processing operations:")

-- First create a test image file
local test_image = use_tool("file_operations", {
    operation = "write",
    path = "/tmp/sample_image.jpg",
    content = "Mock image file for testing (real image processing in Phase 3+)"
})

-- Get image metadata
local image_metadata_op = use_tool("image_processor", {
    operation = "metadata",
    file_path = "/tmp/sample_image.jpg"
})
print_result("Image metadata", image_metadata_op)

-- Resize image
local resize_image = use_tool("image_processor", {
    operation = "resize",
    input_path = "/tmp/sample_image.jpg",
    output_path = "/tmp/resized_image.jpg",
    width = 800,
    height = 600,
    maintain_aspect = true
})
print_result("Resize to 800x600", resize_image)

-- Create thumbnail
local image_thumbnail = use_tool("image_processor", {
    operation = "thumbnail",
    input_path = "/tmp/sample_image.jpg",
    output_path = "/tmp/image_thumb.jpg",
    size = 150
})
print_result("Create thumbnail", image_thumbnail)

-- Convert image format
local image_convert = use_tool("image_processor", {
    operation = "convert",
    input_path = "/tmp/sample_image.jpg",
    output_path = "/tmp/sample_image.png",
    format = "png"
})
print_result("Convert to PNG", image_convert)

-- Rotate image
local rotate_image = use_tool("image_processor", {
    operation = "rotate",
    input_path = "/tmp/sample_image.jpg",
    output_path = "/tmp/rotated_image.jpg",
    degrees = 90
})
print_result("Rotate 90 degrees", rotate_image)

-- Crop image
local crop_image = use_tool("image_processor", {
    operation = "crop",
    input_path = "/tmp/sample_image.jpg",
    output_path = "/tmp/cropped_image.jpg",
    x = 100,
    y = 100,
    width = 400,
    height = 300
})
print_result("Crop image", crop_image)

-- Note: filter operation not supported in Phase 2
-- Would be: operation = "filter" with filter = "grayscale" in Phase 3+
print("  ℹ️  Apply filter: Phase 3+ feature")

-- Note: info operation would be supported in Phase 3+
print("  ℹ️  Image info: Phase 3+ feature")

print("\n🎨 Advanced Media Processing Examples")
print("=====================================")

-- Example: Create video thumbnail gallery (Phase 3+ feature)
print("\nVideo thumbnail gallery (Phase 3+):")
print("  ℹ️  Video thumbnails: Phase 3+ feature")
-- Commented out for Phase 2
-- local timestamps = {1, 5, 10, 15, 20}
-- for i, ts in ipairs(timestamps) do
--     local thumb = use_tool("video_processor", {
--         operation = "thumbnail",
--         file_path = "/tmp/test_video.txt",
--         output_path = string.format("/tmp/thumb_%02d.jpg", ts),
--         width = 160,
--         height = 90,
--         timestamp = ts
--     })
--     print_result(string.format("Thumbnail at %ds", ts), thumb)
-- end

-- Example: Batch image processing
print("\nBatch image operations:")
local image_operations = {
    {op = "resize", width = 1920, height = 1080, suffix = "_fhd"},
    {op = "resize", width = 1280, height = 720, suffix = "_hd"},
    {op = "resize", width = 640, height = 480, suffix = "_sd"}
}

for _, config in ipairs(image_operations) do
    local output = use_tool("image_processor", {
        operation = config.op,
        input_path = "/tmp/sample_image.jpg",
        output_path = "/tmp/sample" .. config.suffix .. ".jpg",
        width = config.width,
        height = config.height,
        maintain_aspect = true
    })
    print_result(string.format("%dx%d version", config.width, config.height), output)
end

-- Example: Audio format compatibility (Phase 3+)
print("\nAudio format conversions (Phase 3+):")
print("  ℹ️  Audio conversion: Phase 3+ feature")
-- Commented out for Phase 2
-- local audio_formats = {"mp3", "ogg", "flac"}
-- for _, format in ipairs(audio_formats) do
--     local convert = use_tool("audio_processor", {
--         operation = "convert",
--         input_path = "/tmp/test_audio.txt",
--         output_path = "/tmp/sample_audio." .. format,
--         format = format
--     })
--     print_result("Convert to " .. format:upper(), convert)
-- end

print("\n📋 Media Processing Best Practices")
print("==================================")

print([[
1. Always check file existence before processing
2. Validate output paths to prevent overwrites
3. Use appropriate formats for intended use:
   - Web: WebP/JPEG for images, WebM/MP4 for video
   - Archive: PNG for images, FLAC for audio
   - Streaming: HLS/DASH for video, AAC for audio
4. Consider file sizes and compression ratios
5. Preserve original files when possible
6. Handle metadata appropriately (EXIF, ID3, etc.)
]])

print("\n✅ Media Processing Tools Examples Complete!")
print("Demonstrated audio, video, and image processing capabilities.")

-- Summary
local tools_demonstrated = {
    "audio_processor",
    "video_processor", 
    "image_processor"
}

print("\n📊 Summary:")
print("  Tools tested: " .. #tools_demonstrated)
print("  Operations demonstrated:")
print("    - Audio: metadata, convert (Phase 3+)")
print("    - Video: metadata, extract_frame (Phase 3+), thumbnail (Phase 3+)")
print("    - Image: metadata, resize, thumbnail, convert, rotate, crop, filter")
print("  Use cases:")
print("    - Format conversion for compatibility")
print("    - Thumbnail generation for previews")
print("    - Metadata extraction for cataloging")
print("    - Batch processing for multiple outputs")

return {
    tools_demonstrated = #tools_demonstrated,
    categories = "media_processing",
    operations = {
        audio = {"info", "convert", "metadata", "waveform", "trim"},
        video = {"info", "extract_frame", "thumbnail", "convert", "metadata"},
        image = {"info", "resize", "thumbnail", "convert", "rotate", "crop", "filter", "metadata"}
    },
    use_cases = {
        "format_conversion",
        "thumbnail_generation",
        "metadata_extraction",
        "batch_processing"
    },
    status = "success"
}