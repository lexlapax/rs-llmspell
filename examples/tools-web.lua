-- tools-web.lua
-- Examples for web-related tools (WebScraper, UrlAnalyzer, ApiTester, etc.)
-- Using direct Tool API

print("🌐 Web Tools Examples")
print("===================")

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
        print("  ❌ " .. label .. ": " .. (result.message or result.error or "Failed"))
    else
        -- Extract relevant output
        local r = result.result or result
        if r.title then
            print("  ✅ " .. label .. ": Title - " .. r.title)
        elseif r.valid ~= nil then
            print("  ✅ " .. label .. ": Valid=" .. tostring(r.valid))
        elseif r.status_code then
            print("  ✅ " .. label .. ": HTTP " .. r.status_code)
        elseif r.urls then
            print("  ✅ " .. label .. ": Found " .. #r.urls .. " URLs")
        elseif r.current_content then
            print("  ✅ " .. label .. ": Content length=" .. string.len(r.current_content))
        elseif result.message then
            print("  ✅ " .. label .. ": " .. result.message)
        else
            print("  ✅ " .. label .. ": Success")
            if type(r) == "table" then
                TestHelpers.print_table(r, 2)
            end
        end
    end
end

TestHelpers.print_section("URL Analyzer Tool")

print("\nAnalyzing different URLs:")

-- Valid HTTP URL
local url_result = use_tool("url-analyzer", {
    input = "https://example.com/path?query=value#section",
    decode_params = true
})
print_result("Valid URL analysis", url_result)

-- URL with authentication
local auth_url_result = use_tool("url-analyzer", {
    input = "https://user:pass@example.com/secure",
    decode_params = true
})
print_result("URL with auth", auth_url_result)

-- Invalid URL
local invalid_url_result = use_tool("url-analyzer", {
    input = "not-a-url",
    decode_params = false
})
print_result("Invalid URL", invalid_url_result)

TestHelpers.print_section("Web Scraper Tool")

print("\nScraping web content:")

-- Basic scraping
local scrape_result = use_tool("web-scraper", {
    input = "https://example.com",
    extract_links = true,
    extract_images = false,
    extract_meta = true
})
print_result("Basic web scraping", scrape_result)

-- Scraping with CSS selector
local selector_result = use_tool("web-scraper", {
    input = "https://example.com",
    selector = "h1",
    timeout = 10
})
print_result("Scraping with selector", selector_result)

-- Scraping with all metadata
local full_scrape_result = use_tool("web-scraper", {
    input = "https://httpbin.org/html",
    extract_links = true,
    extract_images = true,
    extract_meta = true,
    timeout = 15
})
print_result("Full metadata extraction", full_scrape_result)

TestHelpers.print_section("API Tester Tool")

print("\nTesting REST APIs:")

-- GET request
local get_result = use_tool("api-tester", {
    input = "https://httpbin.org/get",
    method = "GET",
    headers = {
        ["User-Agent"] = "LLMSpell-Examples/1.0"
    }
})
print_result("GET request", get_result)

-- POST request with JSON
local post_result = use_tool("api-tester", {
    input = "https://httpbin.org/post",
    method = "POST",
    body = {
        name = "Test User",
        email = "test@example.com"
    },
    headers = {
        ["Content-Type"] = "application/json"
    }
})
print_result("POST with JSON", post_result)

-- Testing response validation
local validation_result = use_tool("api-tester", {
    input = "https://httpbin.org/status/200",
    method = "GET",
    expected_status = 200,
    timeout = 5
})
print_result("Status validation", validation_result)

TestHelpers.print_section("Webhook Caller Tool")

print("\nCalling webhooks:")

-- Basic webhook call
local webhook_result = use_tool("webhook-caller", {
    input = "https://httpbin.org/post",
    method = "POST",
    payload = {
        event = "user.created",
        user_id = 12345,
        timestamp = os.time()
    }
})
print_result("Basic webhook", webhook_result)

-- Webhook with custom headers
local webhook_headers_result = use_tool("webhook-caller", {
    input = "https://httpbin.org/post",
    headers = {
        ["X-Webhook-Secret"] = "my-secret-key",
        ["X-Event-Type"] = "payment.completed"
    },
    payload = {
        amount = 99.99,
        currency = "USD"
    },
    retry_count = 3,
    retry_delay = 1000
})
print_result("Webhook with headers", webhook_headers_result)

TestHelpers.print_section("Webpage Monitor Tool")

print("\nMonitoring webpage changes:")

-- Initial content check
local monitor_result = use_tool("webpage-monitor", {
    input = "https://example.com",
    ignore_whitespace = true
})
print_result("Initial content check", monitor_result)

-- Check with previous content (simulated)
local monitor_diff_result = use_tool("webpage-monitor", {
    input = "https://example.com",
    previous_content = "Example Domain This domain is for use in illustrative examples",
    ignore_whitespace = true
})
print_result("Content comparison", monitor_diff_result)

-- Monitor specific selector
local monitor_selector_result = use_tool("webpage-monitor", {
    input = "https://example.com",
    selector = "h1",
    ignore_whitespace = false
})
print_result("Monitor specific element", monitor_selector_result)

TestHelpers.print_section("Sitemap Crawler Tool")

print("\nCrawling sitemaps:")

-- Basic sitemap crawl
local sitemap_result = use_tool("sitemap-crawler", {
    input = "https://example.com/sitemap.xml",
    max_urls = 10,
    follow_sitemaps = true
})
print_result("Basic sitemap crawl", sitemap_result)

-- Sitemap with timeout
local sitemap_timeout_result = use_tool("sitemap-crawler", {
    input = "https://httpbin.org/sitemap.xml",
    max_urls = 5,
    timeout = 10,
    follow_sitemaps = false
})
print_result("Sitemap with limits", sitemap_timeout_result)

TestHelpers.print_section("Enhanced Web Search Tool")

print("\nUsing enhanced web search with providers:")

-- Basic web search (will use DuckDuckGo by default)
local search_result = use_tool("web_search", {
    input = "Rust programming language",
    provider = "duckduckgo",
    max_results = 5
})
print_result("DuckDuckGo search", search_result)

-- Search with specific provider (if API key is configured)
local google_search_result = use_tool("web_search", {
    input = "machine learning tutorials",
    provider = "google",
    max_results = 3
})
print_result("Google search", google_search_result)

-- Search with filters
local filtered_search_result = use_tool("web_search", {
    input = "climate change",
    provider = "brave",
    max_results = 5,
    safe_search = true
})
print_result("Brave search with filters", filtered_search_result)

print("\n✨ Web tools examples completed!")