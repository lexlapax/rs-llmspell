-- basic-math.lua
-- Demonstrates basic Lua functionality in LLMSpell

-- Simple arithmetic
local a = 10
local b = 20
local sum = a + b

print("Basic Math Operations:")
print(string.format("%d + %d = %d", a, b, sum))
print(string.format("%d - %d = %d", a, b, a - b))
print(string.format("%d * %d = %d", a, b, a * b))
print(string.format("%d / %d = %.2f", a, b, a / b))

-- Working with functions
local function factorial(n)
    if n <= 1 then
        return 1
    else
        return n * factorial(n - 1)
    end
end

print("\nFactorial calculation:")
for i = 1, 5 do
    print(string.format("%d! = %d", i, factorial(i)))
end

-- Return results
return {
    calculations = {
        sum = sum,
        difference = a - b,
        product = a * b,
        quotient = a / b
    },
    factorials = {
        factorial(5),
        factorial(6),
        factorial(7)
    }
}