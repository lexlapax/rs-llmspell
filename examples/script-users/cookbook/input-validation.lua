-- Cookbook: Input Validation - Sanitize User Input
-- Purpose: Implement comprehensive input validation patterns for security and data integrity
-- Prerequisites: Tools optional for enhanced validation features
-- Expected Output: Demonstration of input validation patterns
-- Version: 0.7.0
-- Tags: cookbook, input-validation, security, sanitization, data-integrity

print("=== Input Validation Patterns ===\n")

-- ============================================================
-- Pattern 1: Basic Input Sanitization
-- ============================================================

print("1. Basic Input Sanitization")
print("-" .. string.rep("-", 40))

local InputSanitizer = {}
InputSanitizer.__index = InputSanitizer

function InputSanitizer:new()
    return setmetatable({
        patterns = {
            email = "^[a-zA-Z0-9._%%-]+@[a-zA-Z0-9.-]+%.[a-zA-Z]{2,}$",
            phone = "^%+?[1-9]%d{1,14}$",
            alphanumeric = "^[a-zA-Z0-9]+$",
            alpha = "^[a-zA-Z]+$",
            numeric = "^%d+$",
            url = "^https?://[%w.-]+%.%w+[%w._~:/?#%[%]@!$&'%(%)%*%+,;=-]*$"
        },
        dangerous_patterns = {
            sql_injection = {"'", "\"", ";", "--", "/*", "*/", "xp_", "sp_", "union", "select", "insert", "update", "delete", "drop", "create", "alter"},
            xss = {"<script", "</script>", "javascript:", "vbscript:", "onload=", "onerror=", "onclick=", "onmouseover="},
            path_traversal = {"..", "/", "\\", "~", "$"},
            command_injection = {"|", "&", ";", "$", "`", "$(", "||", "&&"}
        }
    }, self)
end

function InputSanitizer:sanitize_string(input, options)
    options = options or {}
    
    if not input or type(input) ~= "string" then
        return {
            valid = false,
            error = "Input must be a string",
            sanitized = ""
        }
    end
    
    local sanitized = input
    local issues = {}
    
    -- Length validation
    if options.max_length and #sanitized > options.max_length then
        sanitized = string.sub(sanitized, 1, options.max_length)
        table.insert(issues, "Truncated to " .. options.max_length .. " characters")
    end
    
    if options.min_length and #sanitized < options.min_length then
        return {
            valid = false,
            error = "Input too short (minimum " .. options.min_length .. " characters)",
            sanitized = sanitized
        }
    end
    
    -- Remove null bytes and control characters
    sanitized = string.gsub(sanitized, "%z", "")
    sanitized = string.gsub(sanitized, "[\1-\8\11\12\14-\31\127]", "")
    
    -- HTML entity encoding for dangerous characters
    if options.html_encode then
        sanitized = string.gsub(sanitized, "&", "&amp;")
        sanitized = string.gsub(sanitized, "<", "&lt;")
        sanitized = string.gsub(sanitized, ">", "&gt;")
        sanitized = string.gsub(sanitized, "\"", "&quot;")
        sanitized = string.gsub(sanitized, "'", "&#x27;")
    end
    
    -- SQL injection prevention
    if options.prevent_sql_injection then
        local sql_detected = false
        for _, pattern in ipairs(self.dangerous_patterns.sql_injection) do
            if string.find(string.lower(sanitized), string.lower(pattern), 1, true) then
                sql_detected = true
                table.insert(issues, "Potential SQL injection pattern detected: " .. pattern)
            end
        end
        
        if sql_detected and options.strict then
            return {
                valid = false,
                error = "Input contains potentially dangerous SQL patterns",
                sanitized = sanitized,
                issues = issues
            }
        end
    end
    
    -- Pattern validation
    if options.pattern then
        local pattern = self.patterns[options.pattern] or options.pattern
        if not string.match(sanitized, pattern) then
            return {
                valid = false,
                error = "Input does not match required pattern: " .. options.pattern,
                sanitized = sanitized
            }
        end
    end
    
    return {
        valid = true,
        sanitized = sanitized,
        issues = issues,
        original_length = #input,
        sanitized_length = #sanitized
    }
end

function InputSanitizer:validate_email(email)
    local result = self:sanitize_string(email, {
        pattern = "email",
        max_length = 254,
        min_length = 5
    })
    
    if result.valid then
        -- Additional email-specific checks
        local parts = {}
        for part in string.gmatch(result.sanitized, "[^@]+") do
            table.insert(parts, part)
        end
        
        if #parts ~= 2 then
            result.valid = false
            result.error = "Invalid email format"
        elseif #parts[1] > 64 then
            result.valid = false
            result.error = "Email local part too long"
        end
    end
    
    return result
end

function InputSanitizer:validate_password(password, requirements)
    requirements = requirements or {}
    
    local issues = {}
    local score = 0
    
    if not password or type(password) ~= "string" then
        return {
            valid = false,
            error = "Password must be a string",
            strength_score = 0
        }
    end
    
    -- Length requirements
    local min_length = requirements.min_length or 8
    if #password < min_length then
        table.insert(issues, "Password must be at least " .. min_length .. " characters")
    else
        score = score + 1
    end
    
    -- Character variety requirements
    if requirements.require_uppercase and not string.match(password, "[A-Z]") then
        table.insert(issues, "Password must contain uppercase letters")
    elseif string.match(password, "[A-Z]") then
        score = score + 1
    end
    
    if requirements.require_lowercase and not string.match(password, "[a-z]") then
        table.insert(issues, "Password must contain lowercase letters")
    elseif string.match(password, "[a-z]") then
        score = score + 1
    end
    
    if requirements.require_numbers and not string.match(password, "%d") then
        table.insert(issues, "Password must contain numbers")
    elseif string.match(password, "%d") then
        score = score + 1
    end
    
    if requirements.require_special and not string.match(password, "[^%w]") then
        table.insert(issues, "Password must contain special characters")
    elseif string.match(password, "[^%w]") then
        score = score + 1
    end
    
    -- Common password checks
    local common_passwords = {"password", "123456", "qwerty", "admin", "letmein"}
    for _, common in ipairs(common_passwords) do
        if string.lower(password) == common then
            table.insert(issues, "Password is too common")
            score = math.max(0, score - 2)
            break
        end
    end
    
    return {
        valid = #issues == 0,
        issues = issues,
        strength_score = math.min(5, score),
        length = #password
    }
end

-- Test basic input sanitization
local sanitizer = InputSanitizer:new()

print("   Testing input sanitization:")

local test_inputs = {
    {input = "normal_user_input", desc = "Normal input"},
    {input = "user@example.com", desc = "Email input"},
    {input = "'; DROP TABLE users; --", desc = "SQL injection attempt"},
    {input = "<script>alert('xss')</script>", desc = "XSS attempt"},
    {input = string.rep("a", 1000), desc = "Very long input"}
}

for _, test in ipairs(test_inputs) do
    print(string.format("\n   %s:", test.desc))
    print(string.format("     Input: %s", string.sub(test.input, 1, 50) .. 
        (#test.input > 50 and "..." or "")))
    
    local result = sanitizer:sanitize_string(test.input, {
        max_length = 100,
        html_encode = true,
        prevent_sql_injection = true,
        strict = false
    })
    
    print(string.format("     Valid: %s", result.valid))
    if not result.valid then
        print(string.format("     Error: %s", result.error))
    end
    if #result.issues > 0 then
        print(string.format("     Issues: %s", table.concat(result.issues, ", ")))
    end
    print(string.format("     Length: %d → %d", result.original_length, result.sanitized_length))
end

-- Test email validation
print("\n   Testing email validation:")
local emails = {"valid@example.com", "invalid-email", "user@", "toolong" .. string.rep("x", 250) .. "@example.com"}

for _, email in ipairs(emails) do
    local result = sanitizer:validate_email(email)
    print(string.format("     %s: %s", email, result.valid and "✅ Valid" or "❌ " .. result.error))
end

-- Test password validation
print("\n   Testing password validation:")
local passwords = {"weak", "StrongPass123!", "password", "Ab1!"}

local requirements = {
    min_length = 8,
    require_uppercase = true,
    require_lowercase = true,
    require_numbers = true,
    require_special = true
}

for _, password in ipairs(passwords) do
    local result = sanitizer:validate_password(password, requirements)
    print(string.format("     %s: %s (strength: %d/5)", 
        password, result.valid and "✅ Valid" or "❌ Invalid", result.strength_score))
end

print()

-- ============================================================
-- Pattern 2: Schema-Based Validation
-- ============================================================

print("2. Schema-Based Validation")
print("-" .. string.rep("-", 40))

local SchemaValidator = {}
SchemaValidator.__index = SchemaValidator

function SchemaValidator:new()
    return setmetatable({
        schemas = {},
        type_validators = {
            string = function(value, constraints)
                if type(value) ~= "string" then
                    return false, "Expected string, got " .. type(value)
                end
                
                if constraints.min_length and #value < constraints.min_length then
                    return false, "String too short (minimum " .. constraints.min_length .. ")"
                end
                
                if constraints.max_length and #value > constraints.max_length then
                    return false, "String too long (maximum " .. constraints.max_length .. ")"
                end
                
                if constraints.pattern and not string.match(value, constraints.pattern) then
                    return false, "String does not match pattern"
                end
                
                return true, nil
            end,
            
            number = function(value, constraints)
                local num = tonumber(value)
                if not num then
                    return false, "Expected number, got " .. type(value)
                end
                
                if constraints.min and num < constraints.min then
                    return false, "Number too small (minimum " .. constraints.min .. ")"
                end
                
                if constraints.max and num > constraints.max then
                    return false, "Number too large (maximum " .. constraints.max .. ")"
                end
                
                if constraints.integer and num ~= math.floor(num) then
                    return false, "Expected integer"
                end
                
                return true, nil
            end,
            
            boolean = function(value, constraints)
                if type(value) ~= "boolean" then
                    return false, "Expected boolean, got " .. type(value)
                end
                return true, nil
            end,
            
            array = function(value, constraints)
                if type(value) ~= "table" or next(value) and type(next(value)) ~= "number" then
                    return false, "Expected array"
                end
                
                if constraints.min_items and #value < constraints.min_items then
                    return false, "Array too short (minimum " .. constraints.min_items .. " items)"
                end
                
                if constraints.max_items and #value > constraints.max_items then
                    return false, "Array too long (maximum " .. constraints.max_items .. " items)"
                end
                
                return true, nil
            end
        }
    }, self)
end

function SchemaValidator:define_schema(name, schema)
    self.schemas[name] = schema
    print(string.format("   📋 Defined schema: %s", name))
end

function SchemaValidator:validate_data(data, schema_name)
    local schema = self.schemas[schema_name]
    if not schema then
        return {
            valid = false,
            error = "Schema not found: " .. schema_name,
            field_errors = {}
        }
    end
    
    return self:validate_against_schema(data, schema)
end

function SchemaValidator:validate_against_schema(data, schema)
    local field_errors = {}
    local valid = true
    
    -- Check required fields
    if schema.required then
        for _, field_name in ipairs(schema.required) do
            if data[field_name] == nil then
                field_errors[field_name] = "Required field missing"
                valid = false
            end
        end
    end
    
    -- Validate each field according to schema
    if schema.properties then
        for field_name, field_schema in pairs(schema.properties) do
            local field_value = data[field_name]
            
            if field_value ~= nil then
                local field_valid, field_error = self:validate_field(field_value, field_schema)
                if not field_valid then
                    field_errors[field_name] = field_error
                    valid = false
                end
            end
        end
    end
    
    -- Check for unknown fields if strict mode
    if schema.strict then
        for field_name, _ in pairs(data) do
            if not schema.properties or not schema.properties[field_name] then
                field_errors[field_name] = "Unknown field in strict mode"
                valid = false
            end
        end
    end
    
    return {
        valid = valid,
        field_errors = field_errors,
        validated_data = data
    }
end

function SchemaValidator:validate_field(value, field_schema)
    local field_type = field_schema.type
    local constraints = field_schema.constraints or {}
    
    if self.type_validators[field_type] then
        return self.type_validators[field_type](value, constraints)
    else
        return false, "Unknown field type: " .. field_type
    end
end

-- Test schema validation
local validator = SchemaValidator:new()

-- Define user registration schema
validator:define_schema("user_registration", {
    required = {"username", "email", "password"},
    strict = true,
    properties = {
        username = {
            type = "string",
            constraints = {
                min_length = 3,
                max_length = 20,
                pattern = "^[a-zA-Z0-9_]+$"
            }
        },
        email = {
            type = "string",
            constraints = {
                pattern = "^[a-zA-Z0-9._%%-]+@[a-zA-Z0-9.-]+%.[a-zA-Z]{2,}$"
            }
        },
        password = {
            type = "string",
            constraints = {
                min_length = 8
            }
        },
        age = {
            type = "number",
            constraints = {
                min = 13,
                max = 120,
                integer = true
            }
        },
        notifications = {
            type = "boolean"
        }
    }
})

print("   Testing schema validation:")

local test_users = {
    {
        username = "john_doe",
        email = "john@example.com",
        password = "securepass123",
        age = 25,
        notifications = true
    },
    {
        username = "ab", -- Too short
        email = "invalid-email",
        password = "weak",
        age = "not_a_number"
    },
    {
        email = "missing@username.com",
        password = "goodpassword"
        -- Missing required username
    }
}

for i, user_data in ipairs(test_users) do
    print(string.format("\n   User %d validation:", i))
    
    local result = validator:validate_data(user_data, "user_registration")
    print(string.format("     Valid: %s", result.valid))
    
    if not result.valid then
        print("     Errors:")
        for field, error in pairs(result.field_errors) do
            print(string.format("       %s: %s", field, error))
        end
    end
end

print()

-- ============================================================
-- Pattern 3: File Upload Validation
-- ============================================================

print("3. File Upload Validation")
print("-" .. string.rep("-", 40))

local FileValidator = {}
FileValidator.__index = FileValidator

function FileValidator:new()
    return setmetatable({
        allowed_extensions = {
            images = {"jpg", "jpeg", "png", "gif", "webp"},
            documents = {"pdf", "doc", "docx", "txt", "rtf"},
            archives = {"zip", "tar", "gz", "rar"},
            code = {"lua", "js", "py", "rs", "go", "java"}
        },
        mime_types = {
            ["image/jpeg"] = "jpg",
            ["image/png"] = "png", 
            ["image/gif"] = "gif",
            ["application/pdf"] = "pdf",
            ["text/plain"] = "txt",
            ["application/zip"] = "zip"
        },
        dangerous_extensions = {"exe", "bat", "cmd", "scr", "com", "pif", "vbs", "js", "jar"},
        max_file_size = 10 * 1024 * 1024 -- 10MB default
    }, self)
end

function FileValidator:validate_file_upload(file_info)
    local validation_result = {
        valid = true,
        errors = {},
        warnings = {},
        file_info = file_info
    }
    
    -- Validate filename
    if not file_info.filename or file_info.filename == "" then
        table.insert(validation_result.errors, "Filename is required")
        validation_result.valid = false
    else
        -- Check for dangerous characters in filename
        local dangerous_chars = {"<", ">", "|", ":", "*", "?", "\"", "/", "\\"}
        for _, char in ipairs(dangerous_chars) do
            if string.find(file_info.filename, char, 1, true) then
                table.insert(validation_result.errors, "Filename contains dangerous character: " .. char)
                validation_result.valid = false
            end
        end
        
        -- Check filename length
        if #file_info.filename > 255 then
            table.insert(validation_result.errors, "Filename too long (maximum 255 characters)")
            validation_result.valid = false
        end
    end
    
    -- Validate file extension
    if file_info.filename then
        local extension = string.lower(string.match(file_info.filename, "%.([^%.]+)$") or "")
        
        if extension == "" then
            table.insert(validation_result.warnings, "File has no extension")
        else
            -- Check if extension is dangerous
            for _, dangerous_ext in ipairs(self.dangerous_extensions) do
                if extension == dangerous_ext then
                    table.insert(validation_result.errors, "Dangerous file extension: " .. extension)
                    validation_result.valid = false
                    break
                end
            end
            
            -- Validate against allowed extensions if specified
            if file_info.allowed_category then
                local allowed = self.allowed_extensions[file_info.allowed_category]
                if allowed then
                    local extension_allowed = false
                    for _, allowed_ext in ipairs(allowed) do
                        if extension == allowed_ext then
                            extension_allowed = true
                            break
                        end
                    end
                    
                    if not extension_allowed then
                        table.insert(validation_result.errors, 
                            string.format("Extension %s not allowed for category %s", 
                                extension, file_info.allowed_category))
                        validation_result.valid = false
                    end
                end
            end
        end
    end
    
    -- Validate file size
    if file_info.size then
        if file_info.size <= 0 then
            table.insert(validation_result.errors, "File is empty")
            validation_result.valid = false
        elseif file_info.size > (file_info.max_size or self.max_file_size) then
            local max_mb = (file_info.max_size or self.max_file_size) / (1024 * 1024)
            table.insert(validation_result.errors, 
                string.format("File too large (maximum %.1f MB)", max_mb))
            validation_result.valid = false
        end
    end
    
    -- Validate MIME type if provided
    if file_info.mime_type then
        local expected_extension = self.mime_types[file_info.mime_type]
        if file_info.filename then
            local actual_extension = string.lower(string.match(file_info.filename, "%.([^%.]+)$") or "")
            
            if expected_extension and actual_extension ~= expected_extension then
                table.insert(validation_result.warnings, 
                    string.format("MIME type %s doesn't match extension %s", 
                        file_info.mime_type, actual_extension))
            end
        end
    end
    
    return validation_result
end

function FileValidator:sanitize_filename(filename)
    if not filename then
        return "unnamed_file"
    end
    
    -- Remove dangerous characters
    local sanitized = string.gsub(filename, "[<>|:*?\"\\\\]", "_")
    
    -- Remove leading/trailing dots and spaces
    sanitized = string.gsub(sanitized, "^[%. ]+", "")
    sanitized = string.gsub(sanitized, "[%. ]+$", "")
    
    -- Limit length
    if #sanitized > 200 then
        local extension = string.match(sanitized, "%.([^%.]+)$")
        local base = string.sub(sanitized, 1, 200 - (extension and (#extension + 1) or 0))
        sanitized = extension and (base .. "." .. extension) or base
    end
    
    -- Ensure we have a valid filename
    if sanitized == "" or sanitized == "." or sanitized == ".." then
        sanitized = "unnamed_file"
    end
    
    return sanitized
end

-- Test file upload validation
local file_validator = FileValidator:new()

print("   Testing file upload validation:")

local test_files = {
    {
        filename = "document.pdf",
        size = 1024 * 1024, -- 1MB
        mime_type = "application/pdf",
        allowed_category = "documents"
    },
    {
        filename = "malware.exe",
        size = 500 * 1024, -- 500KB
        mime_type = "application/octet-stream"
    },
    {
        filename = "image_with<dangerous>chars.jpg",
        size = 2 * 1024 * 1024, -- 2MB
        mime_type = "image/jpeg",
        allowed_category = "images"
    },
    {
        filename = "toolarge.zip",
        size = 50 * 1024 * 1024, -- 50MB
        mime_type = "application/zip",
        max_size = 10 * 1024 * 1024 -- 10MB limit
    },
    {
        filename = "", -- Empty filename
        size = 1024
    }
}

for i, file_info in ipairs(test_files) do
    print(string.format("\n   File %d: %s", i, file_info.filename or "(no name)"))
    
    local result = file_validator:validate_file_upload(file_info)
    print(string.format("     Valid: %s", result.valid))
    
    if #result.errors > 0 then
        print("     Errors:")
        for _, error in ipairs(result.errors) do
            print(string.format("       • %s", error))
        end
    end
    
    if #result.warnings > 0 then
        print("     Warnings:")
        for _, warning in ipairs(result.warnings) do
            print(string.format("       • %s", warning))
        end
    end
    
    if file_info.filename then
        local sanitized = file_validator:sanitize_filename(file_info.filename)
        if sanitized ~= file_info.filename then
            print(string.format("     Sanitized name: %s", sanitized))
        end
    end
end

print()
print("🎯 Key Takeaways:")
print("   • Always validate and sanitize user input")
print("   • Use allow-lists rather than deny-lists when possible")
print("   • Implement proper length and format validation")
print("   • Check for injection attacks (SQL, XSS, etc.)")
print("   • Validate file uploads rigorously")
print("   • Provide clear error messages for failed validation")