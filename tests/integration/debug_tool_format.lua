-- Debug tool output format

print("=== Debugging Tool Output Format ===\n")

-- Test UUID generator
print("1. Testing uuid_generator:")
local uuid_tool = Tool.get("uuid_generator") 
if uuid_tool then
    local result = uuid_tool.execute({operation = "generate"})
    print("  Raw result type:", type(result))
    if type(result) == "table" then
        print("  Keys in result:")
        for k, v in pairs(result) do
            print("    " .. tostring(k) .. " = " .. tostring(v) .. " (" .. type(v) .. ")")
            if type(v) == "table" and k == "result" then
                print("    Inner result keys:")
                for k2, v2 in pairs(v) do
                    print("      " .. tostring(k2) .. " = " .. tostring(v2))
                end
            end
        end
    end
else
    print("  Tool not found!")
end

-- Test hash calculator
print("\n2. Testing hash_calculator:")
local hash_tool = Tool.get("hash_calculator")
if hash_tool then
    local result = hash_tool.execute({
        operation = "hash",
        algorithm = "SHA-256", 
        input = "test"
    })
    print("  Raw result type:", type(result))
    if type(result) == "table" then
        print("  Keys in result:")
        for k, v in pairs(result) do
            print("    " .. tostring(k) .. " = " .. tostring(v) .. " (" .. type(v) .. ")")
            if type(v) == "table" and k == "result" then
                print("    Inner result keys:")
                for k2, v2 in pairs(v) do
                    print("      " .. tostring(k2) .. " = " .. tostring(v2))
                end
            end
        end
    end
else
    print("  Tool not found!")
end

-- Test base64 encoder
print("\n3. Testing base64_encoder:")
local b64_tool = Tool.get("base64_encoder")
if b64_tool then
    local result = b64_tool.execute({
        operation = "encode",
        input = "DRY test"
    })
    print("  Raw result type:", type(result))
    if type(result) == "table" then
        print("  Keys in result:")
        for k, v in pairs(result) do
            print("    " .. tostring(k) .. " = " .. tostring(v) .. " (" .. type(v) .. ")")
            if type(v) == "table" and k == "result" then
                print("    Inner result keys:")
                for k2, v2 in pairs(v) do
                    print("      " .. tostring(k2) .. " = " .. tostring(v2))
                end
            end
        end
    end
else
    print("  Tool not found!")
end

-- Test with executeAsync
print("\n4. Testing with executeAsync:")
if Tool.executeAsync then
    local result = Tool.executeAsync("calculator", {
        operation = "evaluate",
        expression = "2 + 2"
    })
    print("  Raw result type:", type(result))
    if type(result) == "table" then
        print("  Keys in result:")
        for k, v in pairs(result) do
            print("    " .. tostring(k) .. " = " .. tostring(v) .. " (" .. type(v) .. ")")
            if type(v) == "table" and k == "result" then
                print("    Inner result keys:")
                for k2, v2 in pairs(v) do
                    print("      " .. tostring(k2) .. " = " .. tostring(v2))
                end
            end
        end
    end
else
    print("  executeAsync not available")
end