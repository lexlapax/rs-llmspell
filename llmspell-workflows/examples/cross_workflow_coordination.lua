-- ABOUTME: Cross-workflow coordination patterns for complex orchestration
-- ABOUTME: Demonstrates workflow communication, synchronization, and state sharing

-- Cross-Workflow Coordination Patterns

-- 1. Producer-Consumer Pattern
-- One workflow produces data, another consumes it

-- Shared queue for coordination
State.set("work_queue", {})
State.set("queue_lock", false)

-- Producer workflow
local producer = Workflow.loop({
    name = "data_producer",
    description = "Produces data items and adds to queue",
    
    iterator = { range = { start = 1, ["end"] = 10, step = 1 } },
    
    body = {
        {
            name = "generate_item",
            type = "custom",
            execute = function(context)
                -- Generate work item
                local item = {
                    id = "item_" .. context.current_value,
                    data = math.random(100, 999),
                    timestamp = os.time(),
                    priority = math.random(1, 5)
                }
                
                -- Wait for lock
                while State.get("queue_lock") do
                    -- Simulate wait
                end
                
                -- Acquire lock
                State.set("queue_lock", true)
                
                -- Add to queue
                local queue = State.get("work_queue") or {}
                table.insert(queue, item)
                State.set("work_queue", queue)
                
                -- Release lock
                State.set("queue_lock", false)
                
                return {
                    success = true,
                    output = "Produced: " .. item.id
                }
            end
        },
        {
            name = "notify_consumers",
            type = "custom",
            execute = function()
                -- Signal that new items are available
                State.set("items_available", true)
                return { success = true, output = "Notified consumers" }
            end
        }
    }
})

-- Consumer workflow
local consumer = Workflow.loop({
    name = "data_consumer",
    description = "Consumes data items from queue",
    
    iterator = {
        while_condition = {
            type = "custom",
            evaluate = function()
                local queue = State.get("work_queue") or {}
                return #queue > 0 or State.get("items_available")
            end
        }
    },
    
    max_iterations = 20,
    
    body = {
        {
            name = "get_item",
            type = "custom",
            execute = function()
                -- Wait for lock
                while State.get("queue_lock") do
                    -- Simulate wait
                end
                
                -- Acquire lock
                State.set("queue_lock", true)
                
                -- Get item from queue
                local queue = State.get("work_queue") or {}
                local item = table.remove(queue, 1)  -- FIFO
                State.set("work_queue", queue)
                
                -- Release lock
                State.set("queue_lock", false)
                
                if item then
                    return {
                        success = true,
                        output = item
                    }
                else
                    State.set("items_available", false)
                    return {
                        success = false,
                        output = "Queue empty"
                    }
                end
            end
        },
        {
            name = "process_item",
            type = "tool",
            tool = "calculator",
            input = {
                input = "{{step:get_item:output.data}} * 2"
            },
            skip_if = function(context)
                return not context.steps.get_item.success
            end
        },
        {
            name = "store_result",
            type = "custom",
            execute = function(context)
                if not context.steps.get_item.success then
                    return { success = false, output = "No item to process" }
                end
                
                local results = State.get("processed_results") or {}
                table.insert(results, {
                    original = context.steps.get_item.output,
                    processed = context.steps.process_item.output,
                    consumer_id = "consumer_1"
                })
                State.set("processed_results", results)
                
                return {
                    success = true,
                    output = "Processed: " .. context.steps.get_item.output.id
                }
            end
        }
    }
})

-- Coordinator workflow
local coordinator = Workflow.parallel({
    name = "producer_consumer_coordinator",
    description = "Coordinates producer and consumer workflows",
    
    branches = {
        {
            name = "producer_branch",
            workflow = producer
        },
        {
            name = "consumer_branch",
            workflow = consumer
        },
        {
            name = "monitor_branch",
            steps = {
                {
                    name = "monitor_queue",
                    type = "loop",
                    workflow = Workflow.loop({
                        iterator = { range = { start = 1, ["end"] = 5, step = 1 } },
                        body = {
                            {
                                name = "check_status",
                                type = "custom",
                                execute = function()
                                    local queue = State.get("work_queue") or {}
                                    local results = State.get("processed_results") or {}
                                    
                                    print(string.format(
                                        "Queue: %d items, Processed: %d items",
                                        #queue, #results
                                    ))
                                    
                                    return {
                                        success = true,
                                        output = {
                                            queue_size = #queue,
                                            processed_count = #results
                                        }
                                    }
                                end
                            }
                        }
                    })
                }
            }
        }
    }
})

print("Starting Producer-Consumer coordination...")
local coord_result = coordinator:execute()
print("Coordination completed: " .. (coord_result.success and "Success" or "Failed"))

-- Display results
local final_results = State.get("processed_results") or {}
print("Total items processed: " .. #final_results)

-- 2. Pipeline Orchestration Pattern
-- Multiple workflows form a processing pipeline with handoffs

-- Reset state
State.set("pipeline_stage", "ready")
State.set("pipeline_data", nil)

-- Stage 1: Data Ingestion
local ingestion_workflow = Workflow.sequential({
    name = "pipeline_stage_1_ingestion",
    description = "Ingests and validates raw data",
    
    steps = {
        {
            name = "fetch_data",
            type = "custom",
            execute = function()
                -- Simulate data fetch
                local data = {
                    records = {},
                    source = "api",
                    timestamp = os.time()
                }
                
                for i = 1, 5 do
                    table.insert(data.records, {
                        id = i,
                        value = math.random(10, 100),
                        status = "raw"
                    })
                end
                
                return { success = true, output = data }
            end
        },
        {
            name = "validate_data",
            type = "tool",
            tool = "data_validation",
            input = {
                input = "{{step:fetch_data:output}}",
                schema = {
                    type = "object",
                    required = {"records", "source", "timestamp"}
                }
            }
        },
        {
            name = "handoff_to_transform",
            type = "custom",
            execute = function(context)
                -- Store data for next stage
                State.set("pipeline_data", context.steps.fetch_data.output)
                State.set("pipeline_stage", "transform_ready")
                
                -- Trigger next stage
                Event.emit("pipeline_stage_complete", {
                    stage = "ingestion",
                    next_stage = "transform",
                    record_count = #context.steps.fetch_data.output.records
                })
                
                return {
                    success = true,
                    output = "Handed off to transformation stage"
                }
            end
        }
    }
})

-- Stage 2: Data Transformation
local transform_workflow = Workflow.sequential({
    name = "pipeline_stage_2_transform",
    description = "Transforms and enriches data",
    
    -- Wait for previous stage
    pre_condition = {
        type = "custom",
        evaluate = function()
            return State.get("pipeline_stage") == "transform_ready"
        end
    },
    
    steps = {
        {
            name = "receive_data",
            type = "custom",
            execute = function()
                local data = State.get("pipeline_data")
                if not data then
                    error("No data available from previous stage")
                end
                return { success = true, output = data }
            end
        },
        {
            name = "transform_records",
            type = "loop",
            workflow = Workflow.loop({
                iterator = {
                    collection = function()
                        local data = State.get("pipeline_data")
                        return data and data.records or {}
                    end
                },
                body = {
                    {
                        name = "enrich_record",
                        type = "custom",
                        execute = function(context)
                            local record = context.current_item
                            
                            -- Transform and enrich
                            record.value = record.value * 1.1  -- Apply factor
                            record.status = "transformed"
                            record.enriched_at = os.time()
                            record.category = record.value > 50 and "high" or "low"
                            
                            return {
                                success = true,
                                output = record
                            }
                        end
                    }
                }
            })
        },
        {
            name = "handoff_to_load",
            type = "custom",
            execute = function(context)
                -- Update pipeline data
                local data = State.get("pipeline_data")
                data.stage = "transformed"
                State.set("pipeline_data", data)
                State.set("pipeline_stage", "load_ready")
                
                -- Trigger next stage
                Event.emit("pipeline_stage_complete", {
                    stage = "transform",
                    next_stage = "load"
                })
                
                return {
                    success = true,
                    output = "Handed off to loading stage"
                }
            end
        }
    }
})

-- Stage 3: Data Loading
local load_workflow = Workflow.sequential({
    name = "pipeline_stage_3_load",
    description = "Loads processed data to destination",
    
    -- Wait for previous stage
    pre_condition = {
        type = "custom",
        evaluate = function()
            return State.get("pipeline_stage") == "load_ready"
        end
    },
    
    steps = {
        {
            name = "prepare_batch",
            type = "custom",
            execute = function()
                local data = State.get("pipeline_data")
                
                -- Group by category
                local batches = { high = {}, low = {} }
                for _, record in ipairs(data.records) do
                    table.insert(batches[record.category], record)
                end
                
                return { success = true, output = batches }
            end
        },
        {
            name = "load_high_priority",
            type = "tool",
            tool = "json_processor",
            input = {
                input = "{{step:prepare_batch:output.high}}",
                operation = "stringify"
            }
        },
        {
            name = "load_low_priority",
            type = "tool",
            tool = "json_processor",
            input = {
                input = "{{step:prepare_batch:output.low}}",
                operation = "stringify"
            }
        },
        {
            name = "finalize_pipeline",
            type = "custom",
            execute = function()
                State.set("pipeline_stage", "complete")
                
                Event.emit("pipeline_complete", {
                    stages = {"ingestion", "transform", "load"},
                    total_time = os.time() - State.get("pipeline_data").timestamp
                })
                
                return {
                    success = true,
                    output = "Pipeline completed successfully"
                }
            end
        }
    }
})

-- Pipeline Orchestrator
local pipeline_orchestrator = Workflow.sequential({
    name = "pipeline_orchestrator",
    description = "Orchestrates the entire data pipeline",
    
    steps = {
        {
            name = "stage_1",
            type = "workflow",
            workflow = ingestion_workflow
        },
        {
            name = "stage_2",
            type = "workflow",
            workflow = transform_workflow
        },
        {
            name = "stage_3",
            type = "workflow",
            workflow = load_workflow
        }
    },
    
    on_step_complete = function(step_name, result)
        print("Pipeline stage completed: " .. step_name)
    end
})

print("\n\nStarting Pipeline orchestration...")
local pipeline_result = pipeline_orchestrator:execute()
print("Pipeline completed: " .. (pipeline_result.success and "Success" or "Failed"))

-- 3. Event-Driven Coordination Pattern
-- Workflows communicate through events

-- Event bus setup
State.set("event_bus", {})

-- Event publisher workflow
local event_publisher = Workflow.loop({
    name = "event_publisher",
    description = "Publishes events that trigger other workflows",
    
    iterator = { range = { start = 1, ["end"] = 5, step = 1 } },
    
    body = {
        {
            name = "generate_event",
            type = "custom",
            execute = function(context)
                local event_types = {"user_signup", "order_placed", "payment_received"}
                local event_type = event_types[math.random(#event_types)]
                
                local event = {
                    id = "evt_" .. context.current_value,
                    type = event_type,
                    data = {
                        user_id = "user_" .. math.random(100, 999),
                        amount = math.random(10, 500),
                        timestamp = os.time()
                    }
                }
                
                -- Publish to event bus
                local bus = State.get("event_bus") or {}
                bus[event_type] = bus[event_type] or {}
                table.insert(bus[event_type], event)
                State.set("event_bus", bus)
                
                -- Emit event
                Event.emit(event_type, event)
                
                return {
                    success = true,
                    output = "Published: " .. event_type
                }
            end
        }
    }
})

-- Event subscriber workflows
local signup_handler = Workflow.conditional({
    name = "signup_event_handler",
    description = "Handles user signup events",
    
    branches = {
        {
            name = "process_signup",
            condition = {
                type = "custom",
                evaluate = function()
                    local bus = State.get("event_bus") or {}
                    return bus.user_signup and #bus.user_signup > 0
                end
            },
            steps = {
                {
                    name = "get_signup_event",
                    type = "custom",
                    execute = function()
                        local bus = State.get("event_bus") or {}
                        local event = table.remove(bus.user_signup, 1)
                        State.set("event_bus", bus)
                        return { success = true, output = event }
                    end
                },
                {
                    name = "send_welcome_email",
                    type = "tool",
                    tool = "template_engine",
                    input = {
                        template = "Welcome {{user}}! Thank you for signing up.",
                        variables = {
                            user = "{{step:get_signup_event:output.data.user_id}}"
                        }
                    }
                },
                {
                    name = "create_profile",
                    type = "tool",
                    tool = "uuid_generator",
                    input = { version = "v4" }
                }
            }
        }
    }
})

-- Saga Pattern - Distributed Transaction Coordination
local saga_coordinator = Workflow.sequential({
    name = "order_saga",
    description = "Coordinates distributed order processing with compensation",
    
    steps = {
        {
            name = "reserve_inventory",
            type = "custom",
            execute = function()
                local success = math.random() > 0.2  -- 80% success rate
                
                if success then
                    State.set("inventory_reserved", true)
                    return { success = true, output = "Inventory reserved" }
                else
                    error("Inventory not available")
                end
            end,
            compensate = function()
                -- Compensation logic
                State.set("inventory_reserved", false)
                print("COMPENSATION: Released inventory reservation")
            end
        },
        {
            name = "charge_payment",
            type = "custom",
            execute = function()
                local success = math.random() > 0.3  -- 70% success rate
                
                if success then
                    State.set("payment_charged", true)
                    return { success = true, output = "Payment charged" }
                else
                    error("Payment failed")
                end
            end,
            compensate = function()
                -- Compensation logic
                State.set("payment_charged", false)
                print("COMPENSATION: Refunded payment")
            end
        },
        {
            name = "ship_order",
            type = "custom",
            execute = function()
                local success = math.random() > 0.1  -- 90% success rate
                
                if success then
                    State.set("order_shipped", true)
                    return { success = true, output = "Order shipped" }
                else
                    error("Shipping failed")
                end
            end,
            compensate = function()
                -- This might not be compensatable
                print("COMPENSATION: Cannot unship order - manual intervention required")
            end
        }
    },
    
    on_error = function(error, step_name)
        print("\nSAGA ERROR: " .. step_name .. " failed")
        print("Starting compensation...")
        
        -- Run compensation for completed steps in reverse order
        local completed_steps = State.get("saga_completed_steps") or {}
        for i = #completed_steps, 1, -1 do
            local step = completed_steps[i]
            if step.compensate then
                step.compensate()
            end
        end
        
        State.set("saga_status", "compensated")
    end,
    
    on_step_complete = function(step_name, result)
        if result.success then
            local completed = State.get("saga_completed_steps") or {}
            table.insert(completed, {
                name = step_name,
                compensate = result.compensate
            })
            State.set("saga_completed_steps", completed)
        end
    end
})

print("\n\nTesting Saga pattern...")
State.set("saga_completed_steps", {})
local saga_result = saga_coordinator:execute()
print("Saga result: " .. (saga_result.success and "Success" or "Compensated"))

print("\n\nCross-workflow coordination examples completed!")