-- Cookbook: Consensus Patterns - Agreement Mechanisms for Multi-Agent Systems
-- Purpose: Implement consensus patterns for distributed decision making among agents
-- Prerequisites: OPENAI_API_KEY or ANTHROPIC_API_KEY environment variable, cookbook config
-- Expected Output: Demonstration of consensus patterns for agent coordination
-- Version: 0.7.0
-- Tags: cookbook, consensus, multi-agent, distributed-systems, agreement

print("=== Consensus Patterns ===\n")

-- ============================================================
-- Pattern 1: Majority Voting Consensus
-- ============================================================

print("1. Majority Voting Consensus")
print("-" .. string.rep("-", 40))

local MajorityVoting = {}
MajorityVoting.__index = MajorityVoting

function MajorityVoting:new(required_majority)
    return setmetatable({
        participants = {},
        required_majority = required_majority or 0.51, -- 51%
        votes = {},
        proposals = {}
    }, self)
end

function MajorityVoting:add_participant(agent_name, weight)
    self.participants[agent_name] = {
        name = agent_name,
        weight = weight or 1,
        active = true
    }
    
    print(string.format("   👤 Added voting participant: %s (weight: %d)", 
        agent_name, weight or 1))
end

function MajorityVoting:create_proposal(proposal_id, description, options)
    self.proposals[proposal_id] = {
        id = proposal_id,
        description = description,
        options = options,
        votes = {},
        status = "open",
        created_at = os.time()
    }
    
    print(string.format("   📋 Created proposal: %s", description))
    print(string.format("     Options: %s", table.concat(options, ", ")))
    
    return proposal_id
end

function MajorityVoting:cast_vote(proposal_id, agent_name, vote_option)
    local proposal = self.proposals[proposal_id]
    local participant = self.participants[agent_name]
    
    if not proposal then
        return false, "Proposal not found"
    end
    
    if proposal.status ~= "open" then
        return false, "Proposal is closed"
    end
    
    if not participant or not participant.active then
        return false, "Participant not eligible"
    end
    
    -- Check if option is valid
    local valid_option = false
    for _, option in ipairs(proposal.options) do
        if option == vote_option then
            valid_option = true
            break
        end
    end
    
    if not valid_option then
        return false, "Invalid vote option"
    end
    
    proposal.votes[agent_name] = {
        option = vote_option,
        weight = participant.weight,
        timestamp = os.time()
    }
    
    print(string.format("   🗳️  %s voted: %s", agent_name, vote_option))
    
    -- Check if we can reach consensus
    local result = self:check_consensus(proposal_id)
    if result.reached then
        proposal.status = "decided"
        print(string.format("   ✅ Consensus reached: %s", result.decision))
    end
    
    return true, nil
end

function MajorityVoting:check_consensus(proposal_id)
    local proposal = self.proposals[proposal_id]
    if not proposal then
        return {reached = false, error = "Proposal not found"}
    end
    
    -- Calculate total possible weight
    local total_weight = 0
    for _, participant in pairs(self.participants) do
        if participant.active then
            total_weight = total_weight + participant.weight
        end
    end
    
    -- Count votes by option
    local vote_counts = {}
    local total_votes_weight = 0
    
    for agent_name, vote in pairs(proposal.votes) do
        local option = vote.option
        vote_counts[option] = (vote_counts[option] or 0) + vote.weight
        total_votes_weight = total_votes_weight + vote.weight
    end
    
    -- Find the option with most votes
    local winning_option = nil
    local winning_weight = 0
    
    for option, weight in pairs(vote_counts) do
        if weight > winning_weight then
            winning_weight = weight
            winning_option = option
        end
    end
    
    -- Check if majority requirement is met
    if winning_option and winning_weight >= (total_weight * self.required_majority) then
        return {
            reached = true,
            decision = winning_option,
            vote_percentage = (winning_weight / total_weight) * 100,
            total_votes = total_votes_weight,
            total_possible = total_weight
        }
    end
    
    return {
        reached = false,
        current_leader = winning_option,
        leader_percentage = winning_option and (winning_weight / total_weight) * 100 or 0,
        required_percentage = self.required_majority * 100
    }
end

function MajorityVoting:get_proposal_status(proposal_id)
    local proposal = self.proposals[proposal_id]
    if not proposal then
        return nil
    end
    
    local consensus = self:check_consensus(proposal_id)
    
    return {
        proposal = proposal,
        consensus = consensus,
        votes_cast = 0,
        vote_breakdown = {}
    }
end

-- Test majority voting
local voting = MajorityVoting:new(0.6) -- 60% majority required

-- Add participants with different weights
voting:add_participant("lead_agent", 3)
voting:add_participant("senior_agent_1", 2)
voting:add_participant("senior_agent_2", 2)
voting:add_participant("junior_agent_1", 1)
voting:add_participant("junior_agent_2", 1)

-- Create a proposal
local proposal_id = voting:create_proposal("deployment_strategy", 
    "Choose deployment strategy for new feature", 
    {"blue_green", "canary", "rolling_update"})

print("   Simulating voting process:")

-- Simulate votes
local votes = {
    {"lead_agent", "blue_green"},
    {"senior_agent_1", "canary"},
    {"senior_agent_2", "blue_green"},
    {"junior_agent_1", "blue_green"},
    {"junior_agent_2", "rolling_update"}
}

for _, vote_data in ipairs(votes) do
    voting:cast_vote(proposal_id, vote_data[1], vote_data[2])
end

local status = voting:get_proposal_status(proposal_id)
print(string.format("   Final result: %s", 
    status.consensus.reached and "Consensus reached" or "No consensus"))

print()

-- ============================================================
-- Pattern 2: Byzantine Fault Tolerant Consensus
-- ============================================================

print("2. Byzantine Fault Tolerant Consensus")
print("-" .. string.rep("-", 40))

local ByzantineConsensus = {}
ByzantineConsensus.__index = ByzantineConsensus

function ByzantineConsensus:new(fault_tolerance)
    return setmetatable({
        nodes = {},
        fault_tolerance = fault_tolerance or 1, -- Can tolerate 1 faulty node
        rounds = {},
        current_round = 0
    }, self)
end

function ByzantineConsensus:add_node(node_name, is_faulty)
    self.nodes[node_name] = {
        name = node_name,
        is_faulty = is_faulty or false,
        value = nil,
        votes = {},
        active = true
    }
    
    print(string.format("   🖥️  Added node: %s %s", node_name, 
        is_faulty and "(faulty)" or "(honest)"))
end

function ByzantineConsensus:start_consensus(initial_values)
    self.current_round = 1
    
    -- Set initial values for nodes
    for node_name, value in pairs(initial_values) do
        if self.nodes[node_name] then
            self.nodes[node_name].value = value
        end
    end
    
    print("   Starting Byzantine consensus...")
    
    -- Simulate multiple rounds until consensus
    local max_rounds = 5
    for round = 1, max_rounds do
        print(string.format("\n   Round %d:", round))
        
        local consensus_reached = self:execute_round(round)
        if consensus_reached then
            print(string.format("   ✅ Consensus reached in round %d", round))
            break
        end
    end
    
    return self:get_consensus_result()
end

function ByzantineConsensus:execute_round(round_number)
    self.rounds[round_number] = {
        messages = {},
        votes = {},
        faulty_behavior = {}
    }
    
    local round_data = self.rounds[round_number]
    
    -- Phase 1: Broadcast values
    for node_name, node in pairs(self.nodes) do
        if node.active then
            local broadcast_value = node.value
            
            -- Simulate faulty behavior
            if node.is_faulty then
                -- Faulty nodes might send different values to different nodes
                broadcast_value = "faulty_value_" .. round_number
                table.insert(round_data.faulty_behavior, {
                    node = node_name,
                    behavior = "inconsistent_broadcast"
                })
            end
            
            round_data.messages[node_name] = broadcast_value
            print(string.format("     %s broadcasts: %s", node_name, broadcast_value))
        end
    end
    
    -- Phase 2: Collect votes based on received messages
    local vote_counts = {}
    
    for node_name, node in pairs(self.nodes) do
        if node.active and not node.is_faulty then
            -- Honest nodes vote for the majority value they received
            local received_values = {}
            for sender, value in pairs(round_data.messages) do
                if sender ~= node_name then
                    table.insert(received_values, value)
                end
            end
            
            -- Count occurrences
            local value_counts = {}
            for _, value in ipairs(received_values) do
                value_counts[value] = (value_counts[value] or 0) + 1
            end
            
            -- Vote for most common value
            local best_value = nil
            local best_count = 0
            for value, count in pairs(value_counts) do
                if count > best_count then
                    best_count = count
                    best_value = value
                end
            end
            
            if best_value then
                round_data.votes[node_name] = best_value
                vote_counts[best_value] = (vote_counts[best_value] or 0) + 1
                print(string.format("     %s votes for: %s", node_name, best_value))
            end
        end
    end
    
    -- Check for consensus (need > 2/3 of honest nodes to agree)
    local honest_nodes = 0
    for _, node in pairs(self.nodes) do
        if node.active and not node.is_faulty then
            honest_nodes = honest_nodes + 1
        end
    end
    
    local required_votes = math.ceil((2 * honest_nodes) / 3)
    
    for value, count in pairs(vote_counts) do
        if count >= required_votes then
            -- Update node values for next round
            for node_name, node in pairs(self.nodes) do
                if not node.is_faulty then
                    node.value = value
                end
            end
            return true
        end
    end
    
    return false
end

function ByzantineConsensus:get_consensus_result()
    -- Get the consensus value from honest nodes
    local honest_values = {}
    for _, node in pairs(self.nodes) do
        if not node.is_faulty then
            table.insert(honest_values, node.value)
        end
    end
    
    -- All honest nodes should have the same value if consensus was reached
    local consensus_value = honest_values[1]
    local all_agree = true
    
    for _, value in ipairs(honest_values) do
        if value ~= consensus_value then
            all_agree = false
            break
        end
    end
    
    return {
        consensus_reached = all_agree,
        consensus_value = all_agree and consensus_value or nil,
        honest_node_values = honest_values,
        total_rounds = self.current_round
    }
end

-- Test Byzantine consensus
local byzantine = ByzantineConsensus:new(1)

-- Add nodes (some faulty)
byzantine:add_node("node_1", false) -- Honest
byzantine:add_node("node_2", false) -- Honest
byzantine:add_node("node_3", false) -- Honest
byzantine:add_node("node_4", true)  -- Faulty

-- Start consensus with different initial values
local initial_values = {
    node_1 = "value_A",
    node_2 = "value_A", 
    node_3 = "value_B",
    node_4 = "value_C"
}

local result = byzantine:start_consensus(initial_values)

print(string.format("\n   Byzantine consensus result:"))
print(string.format("     Consensus reached: %s", result.consensus_reached))
if result.consensus_reached then
    print(string.format("     Agreed value: %s", result.consensus_value))
end
print(string.format("     Total rounds: %d", result.total_rounds))

print()

-- ============================================================
-- Pattern 3: Weighted Consensus with Reputation
-- ============================================================

print("3. Weighted Consensus with Reputation")
print("-" .. string.rep("-", 40))

local ReputationConsensus = {}
ReputationConsensus.__index = ReputationConsensus

function ReputationConsensus:new()
    return setmetatable({
        agents = {},
        reputation_history = {},
        decisions = {},
        min_reputation = 0.1,
        max_reputation = 2.0
    }, self)
end

function ReputationConsensus:add_agent(agent_name, initial_reputation)
    self.agents[agent_name] = {
        name = agent_name,
        reputation = initial_reputation or 1.0,
        decisions_made = 0,
        correct_decisions = 0,
        last_active = os.time()
    }
    
    print(string.format("   🎯 Added agent: %s (reputation: %.2f)", 
        agent_name, initial_reputation or 1.0))
end

function ReputationConsensus:propose_decision(decision_id, description, options, timeout)
    local decision = {
        id = decision_id,
        description = description,
        options = options,
        proposals = {},
        final_decision = nil,
        created_at = os.time(),
        timeout = timeout or 30,
        status = "open"
    }
    
    self.decisions[decision_id] = decision
    
    print(string.format("   📋 Decision proposed: %s", description))
    print(string.format("     Options: %s", table.concat(options, ", ")))
    
    return decision_id
end

function ReputationConsensus:submit_proposal(decision_id, agent_name, proposal)
    local decision = self.decisions[decision_id]
    local agent = self.agents[agent_name]
    
    if not decision or decision.status ~= "open" then
        return false, "Decision not available for proposals"
    end
    
    if not agent then
        return false, "Agent not found"
    end
    
    decision.proposals[agent_name] = {
        proposal = proposal,
        agent_reputation = agent.reputation,
        timestamp = os.time(),
        reasoning = proposal.reasoning or ""
    }
    
    print(string.format("   💡 %s proposes: %s (reputation weight: %.2f)", 
        agent_name, proposal.option, agent.reputation))
    
    return true, nil
end

function ReputationConsensus:calculate_weighted_consensus(decision_id)
    local decision = self.decisions[decision_id]
    if not decision then
        return nil
    end
    
    local option_weights = {}
    local total_weight = 0
    
    for agent_name, proposal in pairs(decision.proposals) do
        local weight = proposal.agent_reputation
        local option = proposal.proposal.option
        
        option_weights[option] = (option_weights[option] or 0) + weight
        total_weight = total_weight + weight
    end
    
    -- Find the option with highest weighted support
    local best_option = nil
    local best_weight = 0
    
    for option, weight in pairs(option_weights) do
        if weight > best_weight then
            best_weight = weight
            best_option = option
        end
    end
    
    local confidence = total_weight > 0 and (best_weight / total_weight) or 0
    
    return {
        decision = best_option,
        confidence = confidence,
        total_weight = total_weight,
        option_weights = option_weights
    }
end

function ReputationConsensus:finalize_decision(decision_id, actual_outcome)
    local decision = self.decisions[decision_id]
    if not decision then
        return false, "Decision not found"
    end
    
    local consensus = self:calculate_weighted_consensus(decision_id)
    decision.final_decision = consensus.decision
    decision.actual_outcome = actual_outcome
    decision.status = "finalized"
    
    print(string.format("   ⚖️  Decision finalized: %s", consensus.decision))
    print(string.format("     Confidence: %.1f%%, Actual outcome: %s", 
        consensus.confidence * 100, actual_outcome))
    
    -- Update agent reputations based on accuracy
    for agent_name, proposal in pairs(decision.proposals) do
        local agent = self.agents[agent_name]
        agent.decisions_made = agent.decisions_made + 1
        
        local was_correct = (proposal.proposal.option == actual_outcome)
        if was_correct then
            agent.correct_decisions = agent.correct_decisions + 1
        end
        
        -- Update reputation based on correctness and confidence
        local accuracy = agent.correct_decisions / agent.decisions_made
        local reputation_change = was_correct and 0.1 or -0.05
        
        agent.reputation = math.max(self.min_reputation, 
            math.min(self.max_reputation, agent.reputation + reputation_change))
        
        print(string.format("     %s: %s (new reputation: %.2f)", 
            agent_name, was_correct and "correct" or "incorrect", agent.reputation))
    end
    
    return consensus
end

function ReputationConsensus:get_agent_rankings()
    local rankings = {}
    
    for _, agent in pairs(self.agents) do
        table.insert(rankings, {
            name = agent.name,
            reputation = agent.reputation,
            accuracy = agent.decisions_made > 0 and 
                (agent.correct_decisions / agent.decisions_made) or 0,
            decisions_made = agent.decisions_made
        })
    end
    
    -- Sort by reputation
    table.sort(rankings, function(a, b) return a.reputation > b.reputation end)
    
    return rankings
end

-- Test reputation-based consensus
local reputation_consensus = ReputationConsensus:new()

-- Add agents with different initial reputations
reputation_consensus:add_agent("expert_agent", 1.8)
reputation_consensus:add_agent("experienced_agent", 1.2)
reputation_consensus:add_agent("novice_agent", 0.8)
reputation_consensus:add_agent("specialist_agent", 1.5)

-- Test multiple decisions to see reputation changes
local test_scenarios = {
    {
        id = "market_direction",
        description = "Predict market direction next quarter",
        options = {"bullish", "bearish", "sideways"},
        proposals = {
            expert_agent = {option = "bearish", reasoning = "Economic indicators suggest downturn"},
            experienced_agent = {option = "sideways", reasoning = "Mixed signals in the market"},
            novice_agent = {option = "bullish", reasoning = "Optimistic sentiment"},
            specialist_agent = {option = "bearish", reasoning = "Historical patterns indicate correction"}
        },
        actual_outcome = "bearish"
    },
    {
        id = "technology_adoption",
        description = "Rate of AI adoption in enterprise",
        options = {"rapid", "moderate", "slow"},
        proposals = {
            expert_agent = {option = "rapid", reasoning = "Strong enterprise demand"},
            experienced_agent = {option = "moderate", reasoning = "Implementation challenges"},
            novice_agent = {option = "rapid", reasoning = "Hype cycle suggests rapid adoption"},
            specialist_agent = {option = "moderate", reasoning = "Regulatory constraints"}
        },
        actual_outcome = "moderate"
    }
}

print("   Testing reputation-based consensus:")

for _, scenario in ipairs(test_scenarios) do
    print(string.format("\n   Scenario: %s", scenario.description))
    
    local decision_id = reputation_consensus:propose_decision(
        scenario.id, scenario.description, scenario.options)
    
    -- Submit proposals
    for agent_name, proposal in pairs(scenario.proposals) do
        reputation_consensus:submit_proposal(decision_id, agent_name, proposal)
    end
    
    -- Calculate and finalize consensus
    local result = reputation_consensus:finalize_decision(decision_id, scenario.actual_outcome)
end

-- Show final agent rankings
print("\n   Final Agent Rankings:")
local rankings = reputation_consensus:get_agent_rankings()
for i, agent in ipairs(rankings) do
    print(string.format("   %d. %s: %.2f reputation (%.1f%% accuracy, %d decisions)", 
        i, agent.name, agent.reputation, agent.accuracy * 100, agent.decisions_made))
end

print()
print("🎯 Key Takeaways:")
print("   • Majority voting works well for simple binary decisions")
print("   • Byzantine fault tolerance handles adversarial participants")
print("   • Reputation systems improve decision quality over time")
print("   • Weight votes based on participant expertise")
print("   • Track decision accuracy to adjust influence")
print("   • Implement timeouts to prevent blocking decisions")