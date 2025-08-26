# Performance Tips for LLMSpell

This guide provides tips and best practices for writing efficient LLMSpell scripts.

## Table of Contents

1. [Understanding Performance Limits](#understanding-performance-limits)
2. [Memory Management](#memory-management)
3. [Efficient Lua Patterns](#efficient-lua-patterns)
4. [Avoiding Common Pitfalls](#avoiding-common-pitfalls)
5. [Benchmarking Your Scripts](#benchmarking-your-scripts)

## Understanding Performance Limits

LLMSpell enforces several limits to ensure stable execution:

- **Memory Limit**: 50MB by default
- **Execution Time**: 5 minutes maximum
- **Script Size**: 10MB maximum
- **Stack Depth**: Limited to prevent infinite recursion

These limits are configurable in `llmspell.toml`:

```toml
[runtime.security]
max_memory_bytes = 52428800  # 50MB
max_execution_time_ms = 300000  # 5 minutes
```

## Memory Management

### 1. Reuse Tables When Possible

```lua
-- Inefficient: Creating new tables in loops
local results = {}
for i = 1, 10000 do
    results[i] = { value = i * 2 }  -- New table each iteration
end

-- Efficient: Reuse table structure
local results = {}
local template = {}
for i = 1, 10000 do
    template.value = i * 2
    results[i] = template.value  -- Store just the value
end
```

### 2. Clear Large Tables When Done

```lua
-- Clear large data structures when no longer needed
local function process_large_dataset()
    local data = load_data()  -- Large table
    local result = process(data)
    
    -- Clear the large table
    data = nil
    collectgarbage("collect")  -- Force garbage collection
    
    return result
end
```

### 3. Use Local Variables

Local variables are faster than global ones:

```lua
-- Slower: Global variable access
count = 0
for i = 1, 1000000 do
    count = count + 1
end

-- Faster: Local variable access
local count = 0
for i = 1, 1000000 do
    count = count + 1
end
```

## Efficient Lua Patterns

### 1. Cache Table Lookups

```lua
-- Inefficient: Multiple table lookups
for i = 1, 10000 do
    process(data.users[i].profile.name)
    update(data.users[i].profile.age)
    validate(data.users[i].profile.email)
end

-- Efficient: Cache the lookup
for i = 1, 10000 do
    local profile = data.users[i].profile
    process(profile.name)
    update(profile.age)
    validate(profile.email)
end
```

### 2. Use Table Concatenation Efficiently

```lua
-- Inefficient: String concatenation in loops
local result = ""
for i = 1, 1000 do
    result = result .. "Line " .. i .. "\n"  -- Creates new string each time
end

-- Efficient: Use table.concat
local lines = {}
for i = 1, 1000 do
    lines[i] = "Line " .. i
end
local result = table.concat(lines, "\n")
```

### 3. Preallocate Tables When Size Is Known

```lua
-- Inefficient: Growing table dynamically
local data = {}
for i = 1, 10000 do
    table.insert(data, compute_value(i))
end

-- Efficient: Preallocate when size is known
local n = 10000
local data = {}
for i = 1, n do
    data[i] = compute_value(i)  -- Direct assignment
end
```

### 4. Use Numeric For Loops

```lua
-- Slower: Generic for with ipairs
for i, value in ipairs(large_array) do
    process(value)
end

-- Faster: Numeric for loop
for i = 1, #large_array do
    process(large_array[i])
end
```

## Avoiding Common Pitfalls

### 1. Avoid Creating Functions in Loops

```lua
-- Inefficient: Creating functions in loops
local handlers = {}
for i = 1, 100 do
    handlers[i] = function(x) return x + i end  -- New function each iteration
end

-- Efficient: Create function factory
local function create_handler(n)
    return function(x) return x + n end
end

local handlers = {}
for i = 1, 100 do
    handlers[i] = create_handler(i)  -- Reuse factory
end
```

### 2. Minimize Global Access

```lua
-- Inefficient: Repeated global access
for i = 1, 10000 do
    local result = math.sin(i) + math.cos(i) + math.tan(i)
end

-- Efficient: Localize frequently used functions
local sin, cos, tan = math.sin, math.cos, math.tan
for i = 1, 10000 do
    local result = sin(i) + cos(i) + tan(i)
end
```

### 3. Avoid Unnecessary Table Creation

```lua
-- Inefficient: Creating intermediate tables
function get_user_info(user)
    return {
        name = user.name,
        age = user.age,
        email = user.email
    }
end

-- Efficient: Return multiple values
function get_user_info(user)
    return user.name, user.age, user.email
end

local name, age, email = get_user_info(user)
```

## Benchmarking Your Scripts

### Simple Timer Function

```lua
-- benchmark.lua
local function benchmark(name, func, iterations)
    iterations = iterations or 1000
    
    local start_time = os.clock()
    
    for i = 1, iterations do
        func()
    end
    
    local end_time = os.clock()
    local total_time = end_time - start_time
    local avg_time = total_time / iterations
    
    return {
        name = name,
        total_time = total_time,
        iterations = iterations,
        average_time = avg_time,
        ops_per_second = iterations / total_time
    }
end

-- Example usage
local results = {}

-- Benchmark table insertion methods
results[1] = benchmark("table.insert", function()
    local t = {}
    for i = 1, 100 do
        table.insert(t, i)
    end
end, 1000)

results[2] = benchmark("direct assignment", function()
    local t = {}
    for i = 1, 100 do
        t[i] = i
    end
end, 1000)

-- Print results
for _, result in ipairs(results) do
    print(string.format(
        "%s: %.3f ms avg, %.0f ops/sec",
        result.name,
        result.average_time * 1000,
        result.ops_per_second
    ))
end
```

### Memory Usage Tracking

```lua
-- memory-track.lua
local function track_memory(operation_name, func)
    -- Force garbage collection before measurement
    collectgarbage("collect")
    collectgarbage("stop")
    
    local before = collectgarbage("count")
    
    -- Run the function
    local result = func()
    
    local after = collectgarbage("count")
    collectgarbage("restart")
    
    return {
        operation = operation_name,
        memory_used_kb = after - before,
        result = result
    }
end

-- Example usage
local mem_info = track_memory("create large table", function()
    local data = {}
    for i = 1, 10000 do
        data[i] = { id = i, value = math.random() }
    end
    return data
end)

print(string.format(
    "Operation '%s' used %.2f KB of memory",
    mem_info.operation,
    mem_info.memory_used_kb
))
```

## Optimization Checklist

Before optimizing, always profile first! Here's a checklist:

1. **Profile First**
   - Measure before optimizing
   - Identify actual bottlenecks
   - Focus on hot paths

2. **Memory Optimization**
   - [ ] Use local variables for frequently accessed values
   - [ ] Clear large tables when done
   - [ ] Reuse objects instead of creating new ones
   - [ ] Preallocate tables when size is known

3. **Algorithm Optimization**
   - [ ] Use appropriate data structures
   - [ ] Cache computed values
   - [ ] Avoid nested loops when possible
   - [ ] Use early returns to skip unnecessary work

4. **Lua-Specific Optimization**
   - [ ] Localize library functions
   - [ ] Use numeric for loops for arrays
   - [ ] Minimize table resizing
   - [ ] Use table.concat for string building

## Performance Comparison Examples

### String Building Performance

```lua
-- string-perf.lua
local iterations = 10000

-- Method 1: Concatenation
local function concat_method()
    local result = ""
    for i = 1, iterations do
        result = result .. i .. ","
    end
    return result
end

-- Method 2: Table join
local function table_method()
    local parts = {}
    for i = 1, iterations do
        parts[i] = i
    end
    return table.concat(parts, ",")
end

-- Benchmark both methods
print("String concatenation:", benchmark("concat", concat_method, 10).average_time * 1000, "ms")
print("Table.concat method:", benchmark("table", table_method, 10).average_time * 1000, "ms")
```

### Table Access Patterns

```lua
-- table-access.lua
local data = {}
for i = 1, 10000 do
    data[i] = { value = i, square = i * i }
end

-- Method 1: Direct indexing
local function direct_access()
    local sum = 0
    for i = 1, #data do
        sum = sum + data[i].value + data[i].square
    end
    return sum
end

-- Method 2: Cached access
local function cached_access()
    local sum = 0
    for i = 1, #data do
        local item = data[i]
        sum = sum + item.value + item.square
    end
    return sum
end

-- Method 3: Destructuring (multiple return)
local function destructured_access()
    local sum = 0
    for i = 1, #data do
        local value, square = data[i].value, data[i].square
        sum = sum + value + square
    end
    return sum
end
```

## Summary

Key performance tips for LLMSpell scripts:

1. **Measure First**: Don't optimize prematurely
2. **Understand Limits**: Know your memory and time constraints
3. **Use Local Variables**: They're faster than globals
4. **Manage Memory**: Clear large data when done
5. **Choose Right Algorithms**: The algorithm matters more than micro-optimizations
6. **Cache Lookups**: Store frequently accessed values
7. **Batch Operations**: Process data in chunks when possible

Remember: Readable code is often more valuable than marginally faster code. Optimize only when necessary and after profiling!