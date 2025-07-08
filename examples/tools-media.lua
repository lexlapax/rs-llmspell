-- tools-media.lua
-- Examples for media processing tools (Audio, Video, Image)
-- Using direct Tool API

print("🎬 Media Processing Tools Examples")
print("==================================")

-- Load test helpers
local TestHelpers = dofile("examples/test-helpers.lua")

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

-- Get audio file information
local audio_info = use_tool("audio_processor", {
    operation = "info",
    file_path = "/tmp/sample_audio.wav"
})
print_result("Audio info", audio_info)

-- Convert audio format
local audio_convert = use_tool("audio_processor", {
    operation = "convert",
    input_path = "/tmp/sample_audio.wav",
    output_path = "/tmp/sample_audio.mp3",
    format = "mp3",
    options = {
        bitrate = "192k",
        sample_rate = 44100
    }
})
print_result("Convert to MP3", audio_convert)

-- Extract audio metadata
local audio_metadata = use_tool("audio_processor", {
    operation = "metadata",
    file_path = "/tmp/sample_audio.mp3"
})
print_result("Audio metadata", audio_metadata)

-- Generate waveform visualization
local waveform = use_tool("audio_processor", {
    operation = "waveform",
    file_path = "/tmp/sample_audio.wav",
    output_path = "/tmp/waveform.png",
    width = 800,
    height = 200
})
print_result("Generate waveform", waveform)

-- Trim audio
local audio_trim = use_tool("audio_processor", {
    operation = "trim",
    input_path = "/tmp/sample_audio.wav",
    output_path = "/tmp/trimmed_audio.wav",
    start_time = 5.0,
    end_time = 15.0
})
print_result("Trim audio (5-15s)", audio_trim)

TestHelpers.print_section("Video Processor Tool")

print("\nVideo processing operations:")

-- Get video information
local video_info = use_tool("video_processor", {
    operation = "info",
    file_path = "/tmp/sample_video.mp4"
})
print_result("Video info", video_info)

-- Extract frame at specific time
local extract_frame = use_tool("video_processor", {
    operation = "extract_frame",
    video_path = "/tmp/sample_video.mp4",
    output_path = "/tmp/frame_10s.jpg",
    timestamp = 10.0
})
print_result("Extract frame at 10s", extract_frame)

-- Generate thumbnail
local thumbnail = use_tool("video_processor", {
    operation = "thumbnail",
    video_path = "/tmp/sample_video.mp4",
    output_path = "/tmp/video_thumbnail.jpg",
    width = 320,
    height = 180,
    timestamp = 5.0
})
print_result("Generate thumbnail", thumbnail)

-- Convert video format
local video_convert = use_tool("video_processor", {
    operation = "convert",
    input_path = "/tmp/sample_video.mp4",
    output_path = "/tmp/sample_video.webm",
    format = "webm",
    options = {
        codec = "vp9",
        quality = "good",
        bitrate = "1M"
    }
})
print_result("Convert to WebM", video_convert)

-- Extract video metadata
local video_metadata = use_tool("video_processor", {
    operation = "metadata",
    file_path = "/tmp/sample_video.mp4"
})
print_result("Video metadata", video_metadata)

TestHelpers.print_section("Image Processor Tool")

print("\nImage processing operations:")

-- Get image information
local image_info = use_tool("image_processor", {
    operation = "info",
    file_path = "/tmp/sample_image.jpg"
})
print_result("Image info", image_info)

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

-- Apply filter
local filter_image = use_tool("image_processor", {
    operation = "filter",
    input_path = "/tmp/sample_image.jpg",
    output_path = "/tmp/filtered_image.jpg",
    filter = "grayscale"
})
print_result("Apply grayscale", filter_image)

-- Extract metadata
local image_metadata = use_tool("image_processor", {
    operation = "metadata",
    file_path = "/tmp/sample_image.jpg"
})
print_result("Image metadata", image_metadata)

print("\n🎨 Advanced Media Processing Examples")
print("=====================================")

-- Example: Create video thumbnail gallery
print("\nVideo thumbnail gallery:")
local timestamps = {1, 5, 10, 15, 20}
for i, ts in ipairs(timestamps) do
    local thumb = use_tool("video_processor", {
        operation = "thumbnail",
        video_path = "/tmp/sample_video.mp4",
        output_path = string.format("/tmp/thumb_%02d.jpg", ts),
        width = 160,
        height = 90,
        timestamp = ts
    })
    print_result(string.format("Thumbnail at %ds", ts), thumb)
end

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

-- Example: Audio format compatibility
print("\nAudio format conversions:")
local audio_formats = {"mp3", "ogg", "flac"}
for _, format in ipairs(audio_formats) do
    local convert = use_tool("audio_processor", {
        operation = "convert",
        input_path = "/tmp/sample_audio.wav",
        output_path = "/tmp/sample_audio." .. format,
        format = format
    })
    print_result("Convert to " .. format:upper(), convert)
end

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
print("    - Audio: info, convert, metadata, waveform, trim")
print("    - Video: info, extract_frame, thumbnail, convert, metadata")
print("    - Image: info, resize, thumbnail, convert, rotate, crop, filter, metadata")
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