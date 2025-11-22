-- Recommended profile: rag-dev
-- Run with: llmspell -p rag-dev run rag-session.lua
-- RAG features with debug logging

-- ============================================================
-- LLMSPELL COOKBOOK SHOWCASE
-- ============================================================
-- Pattern ID: RAG-02 - Session-Based RAG v0.8.0
-- Complexity Level: INTERMEDIATE
-- Real-World Use Case: Conversational AI with context-aware memory
-- Pattern Category: RAG & Session Management
--
-- Purpose: Implement session-specific RAG collections for conversational memory,
--          temporary knowledge bases, and context-aware interactions. Perfect for
--          chatbots, research assistants, and interactive learning systems where
--          context needs to persist within a session but not across sessions.
-- Architecture: Session-scoped vector collections with automatic cleanup
-- Crates Showcased: llmspell-rag, llmspell-sessions, llmspell-bridge
-- Key Features:
--   • Session-specific RAG collections
--   • Conversational memory with vector storage
--   • Dynamic context building during conversations
--   • Session replay with RAG context
--   • Automatic cleanup on session end
--   • Session artifacts integration
--   • Context summarization and compression
--
-- Prerequisites:
--   • LLMSpell installed and built
--   • API key: OPENAI_API_KEY environment variable (for embeddings)
--   • Network connectivity for API calls
--
-- HOW TO RUN:
-- ./target/debug/llmspell -p sessions \
--   run examples/script-users/cookbook/rag-session.lua
--
-- EXPECTED OUTPUT:
-- Session created with unique ID
-- Conversation history stored in session RAG
-- Context-aware responses using session memory
-- Session artifacts saved for replay
-- Cleanup demonstration
--
-- Time to Complete: <15 seconds
-- Production Notes: Set appropriate session TTLs, implement session persistence
--                   for crash recovery, monitor session memory usage, compress
--                   old conversations, implement privacy-compliant data retention.
-- ============================================================

print("=== Session-Based RAG System ===")
print("Pattern RAG-02: INTERMEDIATE - Conversational memory with RAG\n")

-- ============================================================
-- Pattern 1: Session Manager with RAG
-- ============================================================

print("1. Session Manager with RAG Integration")
print("-" .. string.rep("-", 40))

local SessionRAG = {}
SessionRAG.__index = SessionRAG

function SessionRAG:new(options)
    options = options or {}
    return setmetatable({
        session_id = nil,  -- Will be set by Session.create()
        collection_name = nil,
        conversation_history = {},
        context_window = options.context_window or 10,  -- Last N messages for context
        max_memory_size = options.max_memory_size or 1000,  -- Max documents in memory
        summary_threshold = options.summary_threshold or 20,  -- Summarize after N turns
        created_at = os.time(),
        metadata = options.metadata or {}
    }, self)
end

function SessionRAG:initialize()
    -- First create the actual session in the session system
    if Session then
        -- Session.create() returns a session ID string
        local session_id = Session.create({
            name = "RAG Conversation Session",
            description = "Session with integrated RAG memory",
            tags = {"rag", "conversation", "memory"}
        })
        
        if session_id then
            self.session_id = session_id
            print(string.format("   ✓ Session created: %s", self.session_id))
        else
            print("   ✗ Failed to create session")
            return false
        end
    else
        print("   ✗ Session API not available")
        return false
    end
    
    print(string.format("   Initializing RAG for session: %s", self.session_id))
    
    -- Create session-specific RAG collection
    self.collection_name = "session_" .. self.session_id
    local result = RAG.create_session_collection(self.session_id, 3600)
    
    if result and result.created then
        print(string.format("   ✓ RAG collection created: %s", result.namespace or self.collection_name))
        
        -- Update collection name from result
        if result.namespace then
            self.collection_name = result.namespace
        end
        
        return true
    else
        print(string.format("   ✗ Failed to create RAG collection: %s", 
            result and result.error or "Unknown"))
        return false
    end
end

function SessionRAG:add_to_memory(role, content, metadata)
    -- Add conversation turn to history
    local turn = {
        role = role,
        content = content,
        timestamp = os.time(),
        turn_number = #self.conversation_history + 1,
        metadata = metadata or {}
    }
    
    table.insert(self.conversation_history, turn)
    
    -- Ingest into RAG for semantic search
    local doc = {
        content = string.format("[%s]: %s", role, content),
        metadata = {
            role = role,
            turn_number = turn.turn_number,
            timestamp = turn.timestamp,
            session_id = self.session_id
        }
    }
    
    for k, v in pairs(metadata or {}) do
        doc.metadata[k] = v
    end
    
    -- Ingest into session scope
    local result = RAG.ingest(doc, {
        scope = "session",
        scope_id = self.session_id
    })
    
    if result and result.success then
        print(string.format("   ✓ Added to memory: Turn %d (%s)", 
            turn.turn_number, role))
    else
        print(string.format("   ⚠️ Failed to add to RAG memory: %s",
            result and result.error or "Unknown"))
    end
    
    -- Check if we need to summarize
    if #self.conversation_history % self.summary_threshold == 0 then
        self:summarize_conversation()
    end
    
    -- Check memory size limits
    if #self.conversation_history > self.max_memory_size then
        self:compress_memory()
    end
    
    return turn
end

function SessionRAG:get_relevant_context(query, limit)
    limit = limit or 5
    
    print(string.format("   Searching session memory for: '%s'", query))
    
    -- Search session-specific collection
    local result = RAG.search(query, {
        limit = limit,
        collection = self.collection_name
    })
    
    if result and result.success and result.results then
        local context_parts = {}
        
        for _, doc in ipairs(result.results) do
            table.insert(context_parts, {
                content = doc.content,
                score = doc.score,
                metadata = doc.metadata
            })
        end
        
        return context_parts
    end
    
    return {}
end

function SessionRAG:get_recent_context(n)
    n = n or self.context_window
    
    local recent = {}
    local start_idx = math.max(1, #self.conversation_history - n + 1)
    
    for i = start_idx, #self.conversation_history do
        table.insert(recent, self.conversation_history[i])
    end
    
    return recent
end

function SessionRAG:summarize_conversation()
    print(string.format("   📝 Summarizing conversation at turn %d", 
        #self.conversation_history))
    
    -- Get last N turns for summarization
    local to_summarize = self:get_recent_context(self.summary_threshold)
    
    -- Build summary content
    local summary_parts = {}
    for _, turn in ipairs(to_summarize) do
        table.insert(summary_parts, 
            string.format("[%s]: %s", turn.role, turn.content))
    end
    
    local summary_content = "CONVERSATION SUMMARY:\n" .. 
                           table.concat(summary_parts, "\n")
    
    -- Store summary as special document in session scope
    RAG.ingest({
        content = summary_content,
        metadata = {
            type = "summary",
            session_id = self.session_id,
            turns_summarized = #to_summarize,
            timestamp = os.time()
        }
    }, {
        scope = "session",
        scope_id = self.session_id
    })
    
    print("   ✓ Summary stored in session memory")
end

function SessionRAG:compress_memory()
    print("   🗜️ Compressing memory to stay within limits")
    
    -- Keep only recent history in RAM
    local keep_recent = math.floor(self.max_memory_size * 0.5)
    local to_remove = #self.conversation_history - keep_recent
    
    if to_remove > 0 then
        -- Archive old turns before removing
        for i = 1, to_remove do
            local turn = self.conversation_history[i]
            -- Could store to disk or external storage here
        end
        
        -- Remove old turns from memory
        for i = 1, to_remove do
            table.remove(self.conversation_history, 1)
        end
        
        print(string.format("   Compressed: Removed %d old turns", to_remove))
    end
end

function SessionRAG:save_artifact(name, content, artifact_type)
    if not Artifact then
        print("   ⚠️ Artifact API not available")
        return nil
    end
    
    -- Store as session artifact
    -- Artifact.store(session_id, type, name, content, metadata) returns artifact ID table
    local success, artifact_id = pcall(function()
        return Artifact.store(
            self.session_id,
            artifact_type or "text",
            name,
            content,
            {
                conversation_turns = #self.conversation_history,
                timestamp = os.time()
            }
        )
    end)
    
    if success and artifact_id then
        -- artifact_id is a table with content_hash, session_id, sequence
        print(string.format("   ✓ Saved artifact: %s (hash: %s)", 
            name, artifact_id.content_hash or "unknown"))
        return artifact_id
    else
        print(string.format("   ✗ Failed to save artifact: %s",
            artifact_id or "Unknown error"))
        return nil
    end
end

function SessionRAG:cleanup()
    print(string.format("\n   Cleaning up session: %s", self.session_id))
    
    -- Clean up RAG collection (using session scope)
    if self.session_id then
        local deleted_count = RAG.cleanup_scope("session", self.session_id)
        
        if deleted_count then
            print(string.format("   ✓ Session RAG collection cleaned: %d vectors removed", deleted_count))
        end
    end
    
    -- Clear memory
    self.conversation_history = {}
    
    print("   ✓ Session cleanup complete")
end

-- Initialize session
local session = SessionRAG:new({
    metadata = {
        user = "demo_user",
        purpose = "cookbook_example"
    }
})

if not session:initialize() then
    print("Failed to initialize session RAG")
    return {success = false}
end

print()

-- ============================================================
-- Pattern 2: Conversational AI with Memory
-- ============================================================

print("2. Conversational AI with Session Memory")
print("-" .. string.rep("-", 40))

-- Simulate a conversation
local conversation_turns = {
    {role = "user", content = "Hi! I'm interested in learning about machine learning."},
    {role = "assistant", content = "Hello! I'd be happy to help you learn about machine learning. It's a fascinating field that enables computers to learn from data without being explicitly programmed. What aspect would you like to explore first?"},
    {role = "user", content = "Can you explain what neural networks are?"},
    {role = "assistant", content = "Neural networks are computing systems inspired by biological neural networks in animal brains. They consist of interconnected nodes (neurons) organized in layers: input layer, hidden layers, and output layer. Information flows through the network, with each connection having a weight that gets adjusted during training."},
    {role = "user", content = "What's the difference between supervised and unsupervised learning?"},
    {role = "assistant", content = "Great question! Supervised learning uses labeled data where the model learns from input-output pairs (like email spam classification). Unsupervised learning finds patterns in unlabeled data without predefined categories (like customer segmentation). There's also reinforcement learning where agents learn through trial and error with rewards."},
    {role = "user", content = "I'm working on a project to classify images. What approach should I use?"},
    {role = "assistant", content = "For image classification, I recommend using Convolutional Neural Networks (CNNs). They're specifically designed for image data. You could start with transfer learning using pre-trained models like ResNet or EfficientNet. Since we discussed neural networks earlier, CNNs are a specialized type that use convolutional layers to detect features like edges, shapes, and patterns."},
    {role = "user", content = "Tell me more about transfer learning."},
    {role = "assistant", content = "Transfer learning leverages knowledge from a pre-trained model (usually trained on large datasets like ImageNet) and applies it to your specific task. You can freeze the early layers that detect general features and fine-tune the later layers for your specific classes. This is particularly useful for image classification projects like yours, as it requires less data and training time."}
}

print("\n   Starting conversation simulation...")
for i, turn in ipairs(conversation_turns) do
    session:add_to_memory(turn.role, turn.content, {
        topic = i <= 4 and "ml_basics" or "project_specific"
    })
    
    -- Simulate processing delay
    -- In real usage, you would actually call an agent here
end

print()

-- ============================================================
-- Pattern 3: Context-Aware Responses
-- ============================================================

print("3. Context-Aware Question Answering")
print("-" .. string.rep("-", 40))

-- Function to answer questions using session context
function answer_with_context(session, question)
    print(string.format("\n   Question: %s", question))
    
    -- Get relevant context from session memory
    local relevant_context = session:get_relevant_context(question, 3)
    
    if #relevant_context > 0 then
        print("   📚 Retrieved context from conversation:")
        for i, ctx in ipairs(relevant_context) do
            print(string.format("      %d. [Score: %.3f] %s",
                i,
                ctx.score,
                string.sub(ctx.content, 1, 80) .. "..."
            ))
        end
    end
    
    -- Get recent context for continuity
    local recent_context = session:get_recent_context(3)
    print(string.format("   📝 Recent conversation context: %d turns", #recent_context))
    
    -- Build augmented prompt (would send to agent in production)
    local prompt_parts = {
        "Based on our conversation history:",
        ""
    }
    
    -- Add relevant context
    if #relevant_context > 0 then
        table.insert(prompt_parts, "Relevant context:")
        for _, ctx in ipairs(relevant_context) do
            table.insert(prompt_parts, "- " .. ctx.content)
        end
        table.insert(prompt_parts, "")
    end
    
    -- Add recent turns
    table.insert(prompt_parts, "Recent discussion:")
    for _, turn in ipairs(recent_context) do
        table.insert(prompt_parts, string.format("[%s]: %s", 
            turn.role, string.sub(turn.content, 1, 100)))
    end
    table.insert(prompt_parts, "")
    table.insert(prompt_parts, "Question: " .. question)
    
    local augmented_prompt = table.concat(prompt_parts, "\n")
    
    -- Simulate response generation
    local response = "Based on our discussion about CNNs and your image classification project, " ..
                    "I recommend starting with a pre-trained ResNet50 model. Since you're new to " ..
                    "neural networks but understand the basics we covered, transfer learning will " ..
                    "give you the best results with minimal complexity."
    
    -- Add response to memory
    session:add_to_memory("user", question)
    session:add_to_memory("assistant", response)
    
    print(string.format("\n   🤖 Context-aware response:\n   %s", response))
    
    return response
end

-- Test context-aware responses
answer_with_context(session, "What model should I use for my project?")
answer_with_context(session, "How many layers should my network have?")

print()

-- ============================================================
-- Pattern 4: Session Artifacts
-- ============================================================

print("4. Saving Session Artifacts")
print("-" .. string.rep("-", 40))

-- Save conversation summary as artifact
local summary = {}
table.insert(summary, "# Conversation Summary")
table.insert(summary, string.format("Session ID: %s", session.session_id))
table.insert(summary, string.format("Total turns: %d", #session.conversation_history))
table.insert(summary, "\n## Topics Discussed:")
table.insert(summary, "- Machine Learning basics")
table.insert(summary, "- Neural Networks")
table.insert(summary, "- Supervised vs Unsupervised Learning")
table.insert(summary, "- Image Classification")
table.insert(summary, "- CNNs and Transfer Learning")
table.insert(summary, "\n## Key Recommendations:")
table.insert(summary, "- Use CNNs for image classification")
table.insert(summary, "- Start with transfer learning (ResNet50)")
table.insert(summary, "- Fine-tune pre-trained models")

local summary_content = table.concat(summary, "\n")
session:save_artifact("conversation_summary.md", summary_content, "markdown")

-- Save conversation history as artifact
local history_json = JSON and JSON.stringify(session.conversation_history)
if history_json then
    session:save_artifact("conversation_history.json", history_json, "json")
end

print()

-- ============================================================
-- Pattern 5: Session Analytics
-- ============================================================

print("5. Session Analytics")
print("-" .. string.rep("-", 40))

function analyze_session(session)
    print(string.format("\n   📊 Session Analytics for: %s", session.session_id))
    
    -- Basic metrics
    print(string.format("   Duration: %d seconds", 
        os.time() - session.created_at))
    print(string.format("   Total turns: %d", 
        #session.conversation_history))
    
    -- Turn distribution
    local user_turns = 0
    local assistant_turns = 0
    local total_user_chars = 0
    local total_assistant_chars = 0
    
    for _, turn in ipairs(session.conversation_history) do
        if turn.role == "user" then
            user_turns = user_turns + 1
            total_user_chars = total_user_chars + string.len(turn.content)
        else
            assistant_turns = assistant_turns + 1
            total_assistant_chars = total_assistant_chars + string.len(turn.content)
        end
    end
    
    print(string.format("   User turns: %d (avg length: %d chars)",
        user_turns,
        user_turns > 0 and math.floor(total_user_chars / user_turns) or 0
    ))
    print(string.format("   Assistant turns: %d (avg length: %d chars)",
        assistant_turns,
        assistant_turns > 0 and math.floor(total_assistant_chars / assistant_turns) or 0
    ))
    
    -- Topic analysis (simplified)
    local topics = {}
    for _, turn in ipairs(session.conversation_history) do
        if turn.metadata and turn.metadata.topic then
            topics[turn.metadata.topic] = (topics[turn.metadata.topic] or 0) + 1
        end
    end
    
    if next(topics) then
        print("   Topics discussed:")
        for topic, count in pairs(topics) do
            print(string.format("     - %s: %d turns", topic, count))
        end
    end
    
    -- Memory usage
    local stats = RAG.get_stats("session", session.session_id)
    if stats then
        print(string.format("   RAG memory usage:"))
        print(string.format("     Vectors: %d", stats.total_vectors or 0))
        print(string.format("     Storage: %d bytes", stats.total_storage_bytes or 0))
    end
end

analyze_session(session)

print()

-- ============================================================
-- Pattern 6: Session Replay
-- ============================================================

print("6. Session Replay Capability")
print("-" .. string.rep("-", 40))

function replay_session(session, start_turn, end_turn)
    start_turn = start_turn or 1
    end_turn = end_turn or #session.conversation_history
    
    print(string.format("\n   🔄 Replaying session turns %d-%d", start_turn, end_turn))
    print("   " .. string.rep("-", 50))
    
    for i = start_turn, end_turn do
        local turn = session.conversation_history[i]
        if turn then
            print(string.format("\n   Turn %d [%s]:", i, turn.role))
            -- Truncate long messages for display
            local content = turn.content
            if string.len(content) > 150 then
                content = string.sub(content, 1, 147) .. "..."
            end
            print("   " .. content)
        end
    end
    
    print("\n   " .. string.rep("-", 50))
    print("   Replay complete")
end

-- Replay portion of conversation
replay_session(session, 1, 4)

print()

-- ============================================================
-- Pattern 7: Cleanup
-- ============================================================

print("7. Session Cleanup")
print("-" .. string.rep("-", 40))

-- Demonstrate cleanup
session:cleanup()

-- Verify cleanup
print("\n   Verifying cleanup:")
print(string.format("   Conversation history: %d turns", #session.conversation_history))

local search_after_cleanup = RAG.search("neural networks", {
    collection = session.collection_name
})

if search_after_cleanup and search_after_cleanup.results then
    print(string.format("   Documents in collection: %d", #search_after_cleanup.results))
else
    print("   ✓ Collection cleaned or inaccessible")
end

print()
print("🎯 Key Takeaways:")
print("   • Session-specific RAG collections for isolated context")
print("   • Conversational memory with semantic search")
print("   • Context-aware responses using session history")
print("   • Automatic summarization and compression")
print("   • Session artifacts for persistence")
print("   • Clean separation between sessions")
print()
print("💡 Best Practices:")
print("   • Set appropriate TTLs for session collections")
print("   • Implement compression for long conversations")
print("   • Save important artifacts before cleanup")
print("   • Monitor memory usage per session")
print("   • Use summaries to maintain context in long sessions")
print("   • Implement privacy-compliant data retention")

-- Return success
return {
    success = true,
    session_id = session.session_id,
    total_turns = #conversation_turns,
    artifacts_saved = 2
}