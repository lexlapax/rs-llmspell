-- Test memory usage
local data = {}
for i = 1, 1000000 do
    data[i] = string.rep("x", 100)
end
print("Allocated large data structure")
