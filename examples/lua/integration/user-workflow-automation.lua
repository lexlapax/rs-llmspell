-- ABOUTME: User workflow automation with intelligent routing, approval workflows, and multi-user coordination
-- ABOUTME: Demonstrates complete user-facing automation using hooks, events, agents, and workflows for business process management

print("=== User Workflow Automation Integration Example ===")
print("Demonstrates: Complete business workflow automation with user interaction, approvals, and intelligent routing")
print()

-- Workflow automation state
local workflow_system = {
    active_workflows = {},
    user_sessions = {},
    approval_queues = {},
    routing_rules = {},
    metrics = {
        workflows_created = 0,
        workflows_completed = 0,
        user_interactions = 0,
        approvals_processed = 0,
        automated_decisions = 0
    },
    configuration = {
        approval_timeout = 300, -- 5 minutes
        max_approval_chain = 5,
        auto_escalation_enabled = true,
        intelligent_routing = true
    }
}

local subscriptions = {}
local hook_handles = {}

-- User roles and permissions
local user_directory = {
    ["user001"] = {name = "Alice Johnson", role = "employee", department = "engineering", manager = "user003"},
    ["user002"] = {name = "Bob Smith", role = "employee", department = "marketing", manager = "user004"},  
    ["user003"] = {name = "Carol Wilson", role = "manager", department = "engineering", reports = {"user001", "user005"}},
    ["user004"] = {name = "David Brown", role = "manager", department = "marketing", reports = {"user002", "user006"}},
    ["user005"] = {name = "Eve Davis", role = "employee", department = "engineering", manager = "user003"},
    ["user006"] = {name = "Frank Miller", role = "employee", department = "marketing", manager = "user004"},
    ["admin001"] = {name = "Grace Admin", role = "admin", department = "IT", permissions = {"all"}}
}

-- Helper functions
local function generate_workflow_id()
    return "wf_" .. os.time() .. "_" .. math.random(1000, 9999)
end

local function log_workflow_event(level, workflow_id, user_id, message, data)
    local timestamp = os.date("%Y-%m-%d %H:%M:%S", os.time())
    local user_name = user_directory[user_id] and user_directory[user_id].name or user_id or "SYSTEM"
    print(string.format("   [%s] %s [%s] %s: %s", timestamp, level, workflow_id or "SYSTEM", user_name, message))
    
    Event.publish("workflow.audit.log", {
        timestamp = os.time(),
        level = level,
        workflow_id = workflow_id,
        user_id = user_id,
        message = message,
        data = data or {}
    })
end

local function create_workflow(workflow_type, initiator_id, data)
    local workflow_id = generate_workflow_id()
    local workflow = {
        id = workflow_id,
        type = workflow_type,
        initiator = initiator_id,
        created_at = os.time(),
        status = "created",
        current_step = 1,
        steps = {},
        data = data,
        approvals = {},
        notifications = {}
    }
    
    workflow_system.active_workflows[workflow_id] = workflow
    workflow_system.metrics.workflows_created = workflow_system.metrics.workflows_created + 1
    
    log_workflow_event("INFO", workflow_id, initiator_id, "Workflow created", {type = workflow_type})
    
    Event.publish("workflow.user.created", {
        workflow_id = workflow_id,
        workflow_type = workflow_type,
        initiator = initiator_id,
        created_at = os.time(),
        data = data
    })
    
    return workflow
end

print("1. Setting up workflow automation infrastructure:")

print("   📡 Creating workflow event subscriptions:")

-- Set up comprehensive event subscriptions
local workflow_patterns = {
    user_requests = "workflow.user.*",
    approvals = "workflow.approval.*", 
    routing = "workflow.routing.*",
    notifications = "workflow.notification.*",
    automation = "workflow.automation.*",
    audit = "workflow.audit.*",
    escalation = "workflow.escalation.*",
    integration = "workflow.integration.*"
}

for pattern_name, pattern in pairs(workflow_patterns) do
    subscriptions[pattern_name] = Event.subscribe(pattern)
    print(string.format("   • %s: %s", pattern_name, pattern))
end

print("   ✅ Workflow automation infrastructure ready")

print()
print("2. Setting up intelligent workflow hooks:")

print("   🪝 Registering workflow automation hooks:")

-- Hook for workflow initiation and routing
hook_handles.workflow_router = Hook.register("BeforeWorkflowStart", function(context)
    local workflow_name = context.component_id.name
    local correlation_id = context.correlation_id
    
    -- Extract user information from context
    local user_id = context.metadata and context.metadata.user_id or "unknown"
    local user_info = user_directory[user_id]
    
    if user_info then
        log_workflow_event("INFO", correlation_id, user_id, "Workflow routing initiated")
        
        -- Intelligent routing based on user role and workflow type
        local routing_decision = {
            user_id = user_id,
            user_role = user_info.role,
            user_department = user_info.department,
            workflow_type = workflow_name,
            requires_approval = false,
            approval_chain = {},
            automation_level = "low"
        }
        
        -- Determine routing based on workflow type and user
        if workflow_name:find("expense") then
            routing_decision.requires_approval = true
            routing_decision.automation_level = "medium"
            
            -- Build approval chain
            if user_info.manager then
                table.insert(routing_decision.approval_chain, user_info.manager)
            end
            
            -- Add department head for large expenses
            if context.data and context.data.amount and context.data.amount > 1000 then
                routing_decision.automation_level = "low"
                table.insert(routing_decision.approval_chain, "admin001")
            end
            
        elseif workflow_name:find("vacation") then
            routing_decision.requires_approval = true
            routing_decision.automation_level = "high"
            
            if user_info.manager then
                table.insert(routing_decision.approval_chain, user_info.manager)
            end
            
        elseif workflow_name:find("procurement") then
            routing_decision.requires_approval = true
            routing_decision.automation_level = "low"
            table.insert(routing_decision.approval_chain, "admin001")
            
        else
            -- Default routing for other workflows
            routing_decision.automation_level = "high"
        end
        
        Event.publish("workflow.routing.decision", {
            workflow_id = correlation_id,
            user_id = user_id,
            routing_decision = routing_decision,
            timestamp = os.time()
        })
        
        workflow_system.metrics.automated_decisions = workflow_system.metrics.automated_decisions + 1
        
        print(string.format("   🎯 Routed %s workflow for %s (%s approval required)", 
              workflow_name, user_info.name, routing_decision.requires_approval and "approval" or "no"))
        
        return {
            type = "modified",
            data = {
                routing_decision = routing_decision,
                intelligent_routing = true
            }
        }
    end
    
    return "continue"
end, "highest")

-- Hook for approval workflow management
hook_handles.approval_manager = Hook.register("BeforeAgentExecution", function(context)
    local agent_name = context.component_id.name
    
    -- Only handle approval-related agents
    if agent_name:find("approval") or agent_name:find("review") then
        local workflow_id = context.correlation_id
        local routing_data = context.data and context.data.routing_decision
        
        if routing_data and routing_data.requires_approval then
            log_workflow_event("INFO", workflow_id, routing_data.user_id, "Approval process initiated")
            
            -- Create approval requests for each approver in chain
            for i, approver_id in ipairs(routing_data.approval_chain) do
                local approval_request = {
                    workflow_id = workflow_id,
                    approver_id = approver_id,
                    approver_name = user_directory[approver_id] and user_directory[approver_id].name or approver_id,
                    request_time = os.time(),
                    approval_level = i,
                    total_levels = #routing_data.approval_chain,
                    status = "pending",
                    timeout_at = os.time() + workflow_system.configuration.approval_timeout
                }
                
                if not workflow_system.approval_queues[approver_id] then
                    workflow_system.approval_queues[approver_id] = {}
                end
                
                table.insert(workflow_system.approval_queues[approver_id], approval_request)
                
                Event.publish("workflow.approval.request", {
                    workflow_id = workflow_id,
                    approver_id = approver_id,
                    approval_request = approval_request,
                    notification_required = true
                })
                
                print(string.format("   📋 Approval request sent to %s (level %d/%d)", 
                      approval_request.approver_name, i, #routing_data.approval_chain))
            end
            
            workflow_system.metrics.approvals_processed = workflow_system.metrics.approvals_processed + #routing_data.approval_chain
        end
    end
    
    return "continue"
end, "high")

-- Hook for automated notifications
hook_handles.notification_manager = Hook.register("AfterAgentExecution", function(context)
    local agent_name = context.component_id.name
    local workflow_id = context.correlation_id
    
    -- Send notifications for various workflow events
    if workflow_system.active_workflows[workflow_id] then
        local workflow = workflow_system.active_workflows[workflow_id]
        
        -- Determine notification type based on agent
        local notification_type = "status_update"
        local recipients = {workflow.initiator}
        
        if agent_name:find("approval") then
            notification_type = "approval_status"
            -- Add approvers to recipients
            for _, approval in pairs(workflow.approvals) do
                table.insert(recipients, approval.approver_id)
            end
        elseif agent_name:find("complete") then
            notification_type = "workflow_completed"
        elseif agent_name:find("error") then
            notification_type = "workflow_error"
            -- Add managers to recipients for errors
            local user_info = user_directory[workflow.initiator]
            if user_info and user_info.manager then
                table.insert(recipients, user_info.manager)
            end
        end
        
        -- Send notifications to all recipients
        for _, recipient_id in ipairs(recipients) do
            local recipient_info = user_directory[recipient_id]
            if recipient_info then
                Event.publish("workflow.notification.send", {
                    workflow_id = workflow_id,
                    recipient_id = recipient_id,
                    recipient_name = recipient_info.name,
                    notification_type = notification_type,
                    message = string.format("Workflow %s update: %s executed", workflow_id, agent_name),
                    timestamp = os.time(),
                    priority = agent_name:find("error") and "high" or "normal"
                })
                
                print(string.format("   📧 Notification sent to %s: %s", recipient_info.name, notification_type))
            end
        end
    end
    
    return "continue"
end, "normal")

-- Hook for workflow escalation
hook_handles.escalation_manager = Hook.register("BeforeWorkflowStep", function(context)
    local step_name = context.component_id.name
    local workflow_id = context.correlation_id
    
    -- Check for approval timeouts and escalate if needed
    if step_name:find("approval") and workflow_system.configuration.auto_escalation_enabled then
        local current_time = os.time()
        
        -- Check all approval queues for timeouts
        for approver_id, queue in pairs(workflow_system.approval_queues) do
            for i, approval_request in ipairs(queue) do
                if approval_request.workflow_id == workflow_id and 
                   approval_request.status == "pending" and
                   current_time > approval_request.timeout_at then
                    
                    log_workflow_event("WARN", workflow_id, approver_id, "Approval timeout - escalating")
                    
                    -- Mark as escalated
                    approval_request.status = "escalated"
                    approval_request.escalated_at = current_time
                    
                    -- Escalate to next level or admin
                    local escalation_target = "admin001" -- Default escalation
                    local approver_info = user_directory[approver_id]
                    
                    if approver_info and approver_info.manager and approver_info.manager ~= "admin001" then
                        escalation_target = approver_info.manager
                    end
                    
                    Event.publish("workflow.escalation.timeout", {
                        workflow_id = workflow_id,
                        original_approver = approver_id,
                        escalated_to = escalation_target,
                        escalation_reason = "approval_timeout",
                        escalation_time = current_time
                    })
                    
                    -- Create new approval request for escalation target
                    local escalated_request = {
                        workflow_id = workflow_id,
                        approver_id = escalation_target,
                        approver_name = user_directory[escalation_target] and user_directory[escalation_target].name or escalation_target,
                        request_time = current_time,
                        approval_level = approval_request.approval_level,
                        total_levels = approval_request.total_levels,
                        status = "pending",
                        timeout_at = current_time + workflow_system.configuration.approval_timeout,
                        escalated_from = approver_id
                    }
                    
                    if not workflow_system.approval_queues[escalation_target] then
                        workflow_system.approval_queues[escalation_target] = {}
                    end
                    
                    table.insert(workflow_system.approval_queues[escalation_target], escalated_request)
                    
                    print(string.format("   ⬆️  Escalated approval from %s to %s due to timeout", 
                          approver_info and approver_info.name or approver_id,
                          user_directory[escalation_target] and user_directory[escalation_target].name or escalation_target))
                end
            end
        end
    end
    
    return "continue"
end, "low")

print("   ✅ Workflow automation hooks registered")

print()
print("3. Simulating user workflow requests:")

print("   👥 Processing user workflow requests:")

-- Simulate various user workflow requests
local workflow_requests = {
    {
        user_id = "user001",
        workflow_type = "expense_report",
        data = {
            amount = 450.00,
            category = "travel",
            description = "Conference travel expenses",
            receipts = {"receipt1.pdf", "receipt2.pdf"}
        }
    },
    {
        user_id = "user002", 
        workflow_type = "vacation_request",
        data = {
            start_date = "2024-03-15",
            end_date = "2024-03-20",
            days = 5,
            reason = "Family vacation",
            coverage_plan = "user006"
        }
    },
    {
        user_id = "user005",
        workflow_type = "expense_report", 
        data = {
            amount = 1250.00,
            category = "equipment",
            description = "New laptop for development",
            receipts = {"laptop_receipt.pdf"}
        }
    },
    {
        user_id = "user003",
        workflow_type = "procurement_request",
        data = {
            item = "Software licenses",
            quantity = 10,
            estimated_cost = 2500.00,
            vendor = "TechCorp",
            justification = "Team productivity tools"
        }
    },
    {
        user_id = "user006",
        workflow_type = "training_request",
        data = {
            course = "Advanced Marketing Analytics",
            provider = "EduTech",
            cost = 800.00,
            duration = "3 days",
            justification = "Skill development for Q2 campaigns"
        }
    }
}

-- Process each workflow request
for i, request in ipairs(workflow_requests) do
    local workflow = create_workflow(request.workflow_type, request.user_id, request.data)
    local user_info = user_directory[request.user_id]
    
    print(string.format("   %d. 📝 %s requested %s", i, user_info.name, request.workflow_type))
    
    -- Add workflow-specific processing steps
    if request.workflow_type == "expense_report" then
        workflow.steps = {"validation", "approval", "reimbursement", "accounting"}
        
        Event.publish("workflow.user.expense_submitted", {
            workflow_id = workflow.id,
            user_id = request.user_id,
            amount = request.data.amount,
            category = request.data.category,
            requires_manager_approval = request.data.amount > 100,
            requires_admin_approval = request.data.amount > 1000
        })
        
    elseif request.workflow_type == "vacation_request" then
        workflow.steps = {"validation", "manager_approval", "hr_notification", "calendar_update"}
        
        Event.publish("workflow.user.vacation_submitted", {
            workflow_id = workflow.id,
            user_id = request.user_id,
            start_date = request.data.start_date,
            end_date = request.data.end_date,
            days_requested = request.data.days,
            coverage_arranged = request.data.coverage_plan ~= nil
        })
        
    elseif request.workflow_type == "procurement_request" then
        workflow.steps = {"validation", "budget_check", "admin_approval", "vendor_contact", "purchase"}
        
        Event.publish("workflow.user.procurement_submitted", {
            workflow_id = workflow.id,
            user_id = request.user_id,
            estimated_cost = request.data.estimated_cost,
            vendor = request.data.vendor,
            requires_budget_approval = request.data.estimated_cost > 500
        })
        
    elseif request.workflow_type == "training_request" then
        workflow.steps = {"validation", "manager_approval", "budget_check", "registration"}
        
        Event.publish("workflow.user.training_submitted", {
            workflow_id = workflow.id,
            user_id = request.user_id,
            course = request.data.course,
            cost = request.data.cost,
            training_days = request.data.duration
        })
    end
    
    workflow_system.metrics.user_interactions = workflow_system.metrics.user_interactions + 1
    
    -- Simulate processing delay
    os.execute("sleep 1")
end

print()
print("4. Simulating approval processes:")

print("   ✅ Processing approval workflows:")

-- Simulate approval decisions for pending requests
local approval_decisions = {
    {approver_id = "user003", decision = "approved", comment = "Approved - reasonable travel expenses"},
    {approver_id = "user004", decision = "approved", comment = "Vacation approved - coverage confirmed"},
    {approver_id = "user003", decision = "approved", comment = "Equipment purchase approved"},
    {approver_id = "admin001", decision = "approved", comment = "High-value expense approved"},
    {approver_id = "admin001", decision = "approved", comment = "Procurement request approved"},
    {approver_id = "user004", decision = "approved", comment = "Training investment approved"}
}

-- Process approval decisions
for i, decision in ipairs(approval_decisions) do
    local approver_info = user_directory[decision.approver_id]
    
    -- Find pending approvals for this approver
    if workflow_system.approval_queues[decision.approver_id] then
        for j, approval_request in ipairs(workflow_system.approval_queues[decision.approver_id]) do
            if approval_request.status == "pending" then
                -- Process the approval
                approval_request.status = decision.decision
                approval_request.decision_time = os.time()
                approval_request.comment = decision.comment
                approval_request.decision_duration = approval_request.decision_time - approval_request.request_time
                
                log_workflow_event("INFO", approval_request.workflow_id, decision.approver_id, 
                                  string.format("Approval %s: %s", decision.decision, decision.comment))
                
                Event.publish("workflow.approval.decision", {
                    workflow_id = approval_request.workflow_id,
                    approver_id = decision.approver_id,
                    decision = decision.decision,
                    comment = decision.comment,
                    decision_time = approval_request.decision_time,
                    approval_level = approval_request.approval_level,
                    decision_duration = approval_request.decision_duration
                })
                
                print(string.format("   %d. ✅ %s %s workflow %s", 
                      i, approver_info.name, decision.decision, approval_request.workflow_id))
                
                -- Update workflow status if this was the final approval
                if workflow_system.active_workflows[approval_request.workflow_id] then
                    local workflow = workflow_system.active_workflows[approval_request.workflow_id]
                    
                    if not workflow.approvals then
                        workflow.approvals = {}
                    end
                    
                    workflow.approvals[decision.approver_id] = approval_request
                    
                    -- Check if all required approvals are complete
                    local all_approved = true
                    local routing_data = workflow.data and workflow.data.routing_decision
                    
                    if routing_data and routing_data.approval_chain then
                        for _, required_approver in ipairs(routing_data.approval_chain) do
                            if not workflow.approvals[required_approver] or 
                               workflow.approvals[required_approver].status ~= "approved" then
                                all_approved = false
                                break
                            end
                        end
                        
                        if all_approved then
                            workflow.status = "approved"
                            workflow.approved_at = os.time()
                            
                            Event.publish("workflow.automation.fully_approved", {
                                workflow_id = workflow.id,
                                workflow_type = workflow.type,
                                initiator = workflow.initiator,
                                approved_at = workflow.approved_at,
                                total_approval_time = workflow.approved_at - workflow.created_at
                            })
                            
                            print(string.format("   🎉 Workflow %s fully approved - proceeding to execution", workflow.id))
                        end
                    end
                end
                
                break -- Process only one approval per approver per iteration
            end
        end
    end
end

print()
print("5. Automated workflow execution:")

print("   🤖 Executing approved workflows:")

-- Execute approved workflows automatically
local executed_workflows = 0

for workflow_id, workflow in pairs(workflow_system.active_workflows) do
    if workflow.status == "approved" then
        log_workflow_event("INFO", workflow_id, workflow.initiator, "Starting automated execution")
        
        -- Execute workflow steps
        for step_index, step_name in ipairs(workflow.steps) do
            local step_start = os.time()
            
            Event.publish("workflow.automation.step_started", {
                workflow_id = workflow_id,
                step_name = step_name,
                step_index = step_index,
                total_steps = #workflow.steps,
                start_time = step_start
            })
            
            print(string.format("   🔄 %s: Step %d/%d - %s", workflow_id, step_index, #workflow.steps, step_name))
            
            -- Simulate step execution time
            local step_duration = math.random(1, 3)
            os.execute("sleep " .. step_duration)
            
            -- Simulate step success (95% success rate)
            local step_success = math.random() > 0.05
            
            if step_success then
                Event.publish("workflow.automation.step_completed", {
                    workflow_id = workflow_id,
                    step_name = step_name,
                    step_index = step_index,
                    duration = step_duration,
                    success = true
                })
                
                print(string.format("   ✅ %s: %s completed (%ds)", workflow_id, step_name, step_duration))
            else
                -- Step failed
                workflow.status = "failed"
                workflow.failed_at = os.time()
                workflow.failure_reason = step_name .. "_execution_failed"
                
                Event.publish("workflow.automation.step_failed", {
                    workflow_id = workflow_id,
                    step_name = step_name,
                    step_index = step_index,
                    error_type = "execution_failure"
                })
                
                log_workflow_event("ERROR", workflow_id, workflow.initiator, 
                                  string.format("Workflow failed at step: %s", step_name))
                
                print(string.format("   ❌ %s: %s failed", workflow_id, step_name))
                break
            end
        end
        
        -- Complete workflow if all steps succeeded
        if workflow.status ~= "failed" then
            workflow.status = "completed"
            workflow.completed_at = os.time()
            workflow.total_duration = workflow.completed_at - workflow.created_at
            
            workflow_system.completed_workflows[workflow_id] = workflow
            workflow_system.active_workflows[workflow_id] = nil
            workflow_system.metrics.workflows_completed = workflow_system.metrics.workflows_completed + 1
            
            Event.publish("workflow.automation.completed", {
                workflow_id = workflow_id,
                workflow_type = workflow.type,
                initiator = workflow.initiator,
                completed_at = workflow.completed_at,
                total_duration = workflow.total_duration,
                steps_completed = #workflow.steps
            })
            
            log_workflow_event("SUCCESS", workflow_id, workflow.initiator, 
                              string.format("Workflow completed in %ds", workflow.total_duration))
            
            print(string.format("   🎉 %s: Workflow completed successfully", workflow_id))
            executed_workflows = executed_workflows + 1
        end
    end
end

print(string.format("   📊 Executed %d workflows automatically", executed_workflows))

print()
print("6. User interaction monitoring:")

print("   📱 Monitoring user interactions and feedback:")

-- Monitor workflow-related events from user perspective
local user_interaction_events = {}

for pattern_name, sub_id in pairs(subscriptions) do
    local events_received = 0
    
    print(string.format("   🔍 Monitoring %s:", pattern_name))
    
    for attempt = 1, 3 do
        local received = Event.receive(sub_id, 200)
        if received then
            events_received = events_received + 1
            
            local event_type = received.event_type or "unknown"
            local workflow_id = received.data and received.data.workflow_id
            local user_id = received.data and received.data.user_id or received.data and received.data.initiator
            
            print(string.format("     %d. %s", events_received, event_type))
            
            if workflow_id then
                print(string.format("        Workflow: %s", workflow_id))
            end
            
            if user_id and user_directory[user_id] then
                print(string.format("        User: %s", user_directory[user_id].name))
            end
            
            -- Track specific interaction types
            if received.data then
                if received.data.decision then
                    print(string.format("        Decision: %s", received.data.decision))
                end
                if received.data.amount then
                    print(string.format("        Amount: $%.2f", received.data.amount))
                end
                if received.data.notification_type then
                    print(string.format("        Notification: %s", received.data.notification_type))
                end
            end
        else
            break
        end
    end
    
    user_interaction_events[pattern_name] = events_received
    
    if events_received > 0 then
        print(string.format("   📊 %s: %d events", pattern_name, events_received))
    else
        print(string.format("   ⏰ %s: no events", pattern_name))
    end
end

print()
print("7. Workflow analytics and insights:")

print("   📈 Workflow Analytics Dashboard:")

-- Calculate comprehensive analytics
local total_events = 0
for _, count in pairs(user_interaction_events) do
    total_events = total_events + count
end

local completion_rate = workflow_system.metrics.workflows_created > 0 and
                       (workflow_system.metrics.workflows_completed / workflow_system.metrics.workflows_created) * 100 or 0

local avg_approval_time = 0
local approval_count = 0

-- Calculate average approval time
for _, queues in pairs(workflow_system.approval_queues) do
    for _, approval in ipairs(queues) do
        if approval.decision_duration then
            avg_approval_time = avg_approval_time + approval.decision_duration
            approval_count = approval_count + 1
        end
    end
end

avg_approval_time = approval_count > 0 and avg_approval_time / approval_count or 0

print(string.format("   • Workflows created: %d", workflow_system.metrics.workflows_created))
print(string.format("   • Workflows completed: %d", workflow_system.metrics.workflows_completed))
print(string.format("   • Completion rate: %.1f%%", completion_rate))
print(string.format("   • User interactions: %d", workflow_system.metrics.user_interactions))
print(string.format("   • Approvals processed: %d", workflow_system.metrics.approvals_processed))
print(string.format("   • Automated decisions: %d", workflow_system.metrics.automated_decisions))
print(string.format("   • Average approval time: %ds", math.floor(avg_approval_time)))
print(string.format("   • Total events captured: %d", total_events))

-- User engagement analysis
print("   👥 User Engagement Analysis:")
local user_activity = {}

for user_id, user_info in pairs(user_directory) do
    user_activity[user_id] = {
        name = user_info.name,
        role = user_info.role,
        workflows_initiated = 0,
        approvals_given = 0,
        notifications_received = 0
    }
end

-- Count user activities from completed workflows
for _, workflow in pairs(workflow_system.completed_workflows) do
    if user_activity[workflow.initiator] then
        user_activity[workflow.initiator].workflows_initiated = 
            user_activity[workflow.initiator].workflows_initiated + 1
    end
    
    if workflow.approvals then
        for approver_id, _ in pairs(workflow.approvals) do
            if user_activity[approver_id] then
                user_activity[approver_id].approvals_given = 
                    user_activity[approver_id].approvals_given + 1
            end
        end
    end
end

-- Display top active users
local active_users = {}
for user_id, activity in pairs(user_activity) do
    if activity.workflows_initiated > 0 or activity.approvals_given > 0 then
        table.insert(active_users, {
            user_id = user_id,
            name = activity.name,
            role = activity.role,
            total_activity = activity.workflows_initiated + activity.approvals_given
        })
    end
end

table.sort(active_users, function(a, b) return a.total_activity > b.total_activity end)

for i = 1, math.min(5, #active_users) do
    local user = active_users[i]
    local activity = user_activity[user.user_id]
    print(string.format("   %d. %s (%s): %d workflows, %d approvals", 
          i, user.name, user.role, activity.workflows_initiated, activity.approvals_given))
end

print()
print("8. Business process optimization:")

print("   💡 Business Process Optimization Insights:")

-- Analyze workflow patterns and provide recommendations
local optimization_insights = {}

-- Approval bottleneck analysis
if avg_approval_time > 120 then -- More than 2 minutes
    table.insert(optimization_insights, {
        area = "Approval Speed",
        insight = "Average approval time is high - consider implementing approval delegation or auto-approval rules",
        impact = "Medium",
        estimated_improvement = "30-50% faster approvals"
    })
end

-- Workflow type analysis
local workflow_types = {}
for _, workflow in pairs(workflow_system.completed_workflows) do
    workflow_types[workflow.type] = (workflow_types[workflow.type] or 0) + 1
end

local most_common_type = nil
local max_count = 0
for workflow_type, count in pairs(workflow_types) do
    if count > max_count then
        max_count = count
        most_common_type = workflow_type
    end
end

if most_common_type then
    table.insert(optimization_insights, {
        area = "Process Automation",
        insight = string.format("'%s' is the most common workflow type - consider full automation", most_common_type),
        impact = "High",
        estimated_improvement = "80% reduction in manual processing"
    })
end

-- User role optimization
local manager_approval_load = 0
for user_id, activity in pairs(user_activity) do
    local user_info = user_directory[user_id]
    if user_info and user_info.role == "manager" then
        manager_approval_load = manager_approval_load + activity.approvals_given  
    end
end

if manager_approval_load > 5 then
    table.insert(optimization_insights, {
        area = "Manager Workload",
        insight = "Managers have high approval load - consider approval thresholds or delegation",
        impact = "Medium",
        estimated_improvement = "40% reduction in manager overhead"
    })
end

-- Display insights
for i, insight in ipairs(optimization_insights) do
    print(string.format("   %d. [%s Impact] %s", i, insight.impact, insight.area))
    print(string.format("      %s", insight.insight))
    print(string.format("      Expected: %s", insight.estimated_improvement))
    
    Event.publish("workflow.analytics.optimization", {
        insight_id = "opt_" .. i,
        area = insight.area,
        insight = insight.insight,
        impact = insight.impact,
        estimated_improvement = insight.estimated_improvement,
        timestamp = os.time()
    })
end

if #optimization_insights == 0 then
    print("   ✅ Workflow processes are well-optimized")
end

print()
print("9. Integration best practices demonstrated:")

print("   💡 Integration Best Practices Demonstrated:")
print("   • User-centric workflow design with role-based routing")
print("   • Intelligent approval chain construction based on organizational hierarchy")
print("   • Automated escalation and timeout handling")
print("   • Real-time notification system for stakeholder updates")
print("   • Comprehensive audit trail for compliance and debugging")
print("   • Event-driven architecture for loose coupling and scalability")
print("   • Business process analytics and optimization insights")
print("   • Multi-user coordination with approval queues and status tracking")
print("   • Integration between user actions, system automation, and business rules")
print("   • Configurable workflow parameters and business rules")

print()
print("10. Cleanup and final statistics:")

-- Final statistics summary
local final_stats = {
    workflows_processed = workflow_system.metrics.workflows_created,
    workflows_completed = workflow_system.metrics.workflows_completed,
    user_interactions = workflow_system.metrics.user_interactions,
    approvals_processed = workflow_system.metrics.approvals_processed,
    automated_decisions = workflow_system.metrics.automated_decisions,
    active_users = #active_users,
    optimization_insights = #optimization_insights,
    hooks_registered = 0,
    subscriptions_created = 0
}

-- Count infrastructure components
for _ in pairs(hook_handles) do
    final_stats.hooks_registered = final_stats.hooks_registered + 1
end

for _ in pairs(subscriptions) do
    final_stats.subscriptions_created = final_stats.subscriptions_created + 1
end

print("   📊 Final Workflow Statistics:")
print(string.format("   • Workflows processed: %d", final_stats.workflows_processed))
print(string.format("   • Workflows completed: %d", final_stats.workflows_completed))
print(string.format("   • Completion rate: %.1f%%", completion_rate))
print(string.format("   • User interactions: %d", final_stats.user_interactions))
print(string.format("   • Approvals processed: %d", final_stats.approvals_processed))
print(string.format("   • Automated decisions: %d", final_stats.automated_decisions))
print(string.format("   • Active users: %d", final_stats.active_users))
print(string.format("   • Optimization insights: %d", final_stats.optimization_insights))
print(string.format("   • Infrastructure components: %d hooks, %d subscriptions", 
      final_stats.hooks_registered, final_stats.subscriptions_created))

-- Cleanup
print("   🧹 Cleaning up workflow infrastructure:")
local hooks_cleaned = 0
for hook_name, handle in pairs(hook_handles) do
    if handle and handle:id() then
        Hook.unregister(handle)
        hooks_cleaned = hooks_cleaned + 1
        print(string.format("   • Unregistered hook: %s", hook_name))
    end
end

local subs_cleaned = 0
for pattern_name, sub_id in pairs(subscriptions) do
    local unsubscribed = Event.unsubscribe(sub_id)
    if unsubscribed then
        subs_cleaned = subs_cleaned + 1
        print(string.format("   • Unsubscribed from: %s", pattern_name))
    end
end

local final_hooks = #Hook.list()
local final_subs = Event.list_subscriptions()

print(string.format("   ✅ Cleaned up %d hooks and %d subscriptions", hooks_cleaned, subs_cleaned))
print(string.format("   ✅ Final state: %d hooks, %d subscriptions", final_hooks, #final_subs))

print()
print("✨ User Workflow Automation Integration Example Complete!")
print("   Real-world business process automation demonstrated:")
print("   • Multi-user workflow coordination with role-based routing")
print("   • Intelligent approval chains based on organizational hierarchy")  
print("   • Automated escalation and timeout handling")
print("   • Real-time notifications and status updates")
print("   • Comprehensive business process analytics")
print("   • User engagement tracking and optimization insights")
print("   • Integration of business rules with technical automation")
print("   • Scalable event-driven architecture for enterprise workflows")
print("   • Audit trail and compliance tracking")
print("   • Production-ready patterns for business process management")