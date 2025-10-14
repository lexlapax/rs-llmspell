#!/usr/bin/env llmspell

--[[
Template Discovery and Introspection Example

Demonstrates the Template global API for discovering and inspecting templates:
- Template.list([category]) - List all or filtered templates
- Template.search(query, [category]) - Search templates by keyword
- Template.info(name, [show_schema]) - Get template metadata
- Template.schema(name) - Get parameter schema details

Run: llmspell lua examples/templates/discovery.lua
--]]

print("====================================")
print("  Template Discovery & Introspection")
print("====================================\n")

-- 1. List all available templates
print("1. Template.list() - All Available Templates")
print(string.rep("-", 50))
local all_templates = Template.list()
print(string.format("Found %d templates:\n", #all_templates))

for i, template in ipairs(all_templates) do
    print(string.format("%d. %s", i, template.name))
    print(string.format("   ID: %s", template.id))
    print(string.format("   Category: %s", template.category))
    print(string.format("   Version: %s", template.version))
    print(string.format("   Description: %s", template.description))

    -- Show requirements if any
    if template.requires and #template.requires > 0 then
        print(string.format("   Requires: %s", table.concat(template.requires, ", ")))
    end

    -- Show tags if any
    if template.tags and #template.tags > 0 then
        print(string.format("   Tags: %s", table.concat(template.tags, ", ")))
    end
    print()
end

-- 2. Search for templates by keyword
print("\n2. Template.search('research') - Keyword Search")
print(string.rep("-", 50))
local search_results = Template.search("research")
print(string.format("Found %d matching templates:\n", #search_results))

for i, template in ipairs(search_results) do
    print(string.format("%d. %s (%s)", i, template.name, template.id))
end

-- 3. Get detailed template info (without schema)
print("\n\n3. Template.info('research-assistant', false) - Basic Info")
print(string.rep("-", 50))
local info_basic = Template.info("research-assistant", false)
print(string.format("Name: %s", info_basic.metadata.name))
print(string.format("Description: %s", info_basic.metadata.description))
print(string.format("Category: %s", info_basic.metadata.category))
print(string.format("Version: %s", info_basic.metadata.version))
if info_basic.metadata.author then
    print(string.format("Author: %s", info_basic.metadata.author))
end

-- 4. Get template info with schema
print("\n\n4. Template.info('research-assistant', true) - Info + Schema")
print(string.rep("-", 50))
local info_full = Template.info("research-assistant", true)

if info_full.schema and info_full.schema.parameters then
    print(string.format("\nParameters (%d total):", #info_full.schema.parameters))

    for i, param in ipairs(info_full.schema.parameters) do
        local required_str = param.required and "REQUIRED" or "optional"
        print(string.format("\n  %d. %s (%s, %s)", i, param.name, param.param_type, required_str))
        print(string.format("     Description: %s", param.description))

        -- Show default value if present
        if param.default_value then
            print(string.format("     Default: %s", tostring(param.default_value)))
        end

        -- Show constraints if present
        if param.constraints then
            local constraints = param.constraints
            if constraints.min then
                print(string.format("     Min: %s", constraints.min))
            end
            if constraints.max then
                print(string.format("     Max: %s", constraints.max))
            end
            if constraints.min_length then
                print(string.format("     Min Length: %d", constraints.min_length))
            end
            if constraints.max_length then
                print(string.format("     Max Length: %d", constraints.max_length))
            end
            if constraints.pattern then
                print(string.format("     Pattern: %s", constraints.pattern))
            end
            if constraints.allowed_values and #constraints.allowed_values > 0 then
                local values = {}
                for _, val in ipairs(constraints.allowed_values) do
                    table.insert(values, tostring(val))
                end
                print(string.format("     Allowed Values: %s", table.concat(values, ", ")))
            end
        end
    end
else
    print("No schema information available")
end

-- 5. Get just the schema (alternative to info with schema=true)
print("\n\n5. Template.schema('research-assistant') - Schema Only")
print(string.rep("-", 50))
local schema = Template.schema("research-assistant")

if schema and schema.parameters then
    print(string.format("Required parameters: %d",
        #(function()
            local required = {}
            for _, p in ipairs(schema.parameters) do
                if p.required then table.insert(required, p) end
            end
            return required
        end)()
    ))

    print(string.format("Optional parameters: %d",
        #(function()
            local optional = {}
            for _, p in ipairs(schema.parameters) do
                if not p.required then table.insert(optional, p) end
            end
            return optional
        end)()
    ))
end

print("\n\n====================================")
print("  Discovery Complete")
print("====================================")
