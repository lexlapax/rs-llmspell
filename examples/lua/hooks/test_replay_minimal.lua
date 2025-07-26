-- ABOUTME: Minimal test to check if Replay global is available
-- ABOUTME: This should work if the engine is properly initialized with globals

-- First check if Replay exists
if Replay then
    print("SUCCESS: Replay global is available!")
    
    -- Try to access its methods
    if Replay.modes then
        print("SUCCESS: Replay.modes exists")
    else
        print("ERROR: Replay.modes is nil")
    end
    
    if type(Replay.create_config) == "function" then
        print("SUCCESS: Replay.create_config is a function")
    else
        print("ERROR: Replay.create_config is not a function")
    end
else
    print("ERROR: Replay global is NOT available")
    print("This means the global injection system is not being used by the CLI")
    print("The CLI needs to be updated to use the GlobalInjector system")
end