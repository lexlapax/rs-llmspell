## TODOS Change of Plans for Phase 2 
**checks for architecture consistency, implementation consistency, gaps between phases  and during a phase and late breaking requirements**
Each Section of this document comes with it's own **Instructions** read that for a section at a time. Ask me after you're read the entirety of this document.

### Section 1: Check Architecture for and late breaking requirement
**Instructions** clear all your internal todos first. we need to accomodate some late breaking changes to the `Phase 2` Plans
now I'm going to ask you a series of questions about the architecture documented in `/docs/technical/rs-llmspell-complete-architecture.md` . for each question, document the question in Section 1 of this document. for each question, document the question under the Task/question Section 1 heading. for that question review the architecture document and find the answer and document the answer under - [ ] Answer: . if you do not find the answer, document it as such. if you figure out what needs to be done, document in - [ ] Todo: subtask.Once that's done, prompt me for another answer. To understand this read this TODO.md document and make sure you understand the section **Task 1** and what I'm saying here.  think hard and reason. do what I'm asking in this prompt and come back and tell me that you understand.
- [x] **Task/question 1.1** Tool categories and addition of tools
    - The plan does not account for the following tools: 
        - media conversion and manipulation tools for audio files, video files and images, does this need a new tool category called media? 
        - calculator tool to do math calculations.. should this be in a category of it's own called math ?
        - datetime tool to do date time conversions, localtime, timezone conversations, date calculations.. that can probably be in the utils library?
    - if we are to create a plan to implement the above, does this affect the overall architecture? 
    - do you need to do external research/web research for this question? for additional crates and libraries?
    - [x] Answer: 
        - **Current Tool Categories (8 total):**
          1. File System Operations (8 tools)
          2. Web and Network Operations (7 tools)
          3. Data Processing and Analysis (9 tools)
          4. AI and Machine Learning (6 tools)
          5. System Integration (4 tools)
          6. Utilities and Helpers (8 tools) - **ALREADY includes calculator and date_time_handler**
          7. Communication and External APIs (5 tools)
          8. Specialized Domain Tools (3 tools)
        
        - **Existing Tools Found:**
          - `calculator` - Already exists in "Utilities and Helpers" category (mathematical operations, basic math, scientific functions)
          - `date_time_handler` - Already exists in "Utilities and Helpers" category (date/time operations, parsing, formatting, timezones)
          - `image_processor` - Already exists in "Specialized Domain Tools" category (image manipulation, resize, format conversion, metadata)
        
        - **Media Tools Analysis:**
          - The architecture already has `image_processor` in "Specialized Domain Tools" category
          - Adding audio/video conversion tools would fit naturally in the existing "Specialized Domain Tools" category alongside `image_processor`
          - No new category needed - just expand "Specialized Domain Tools" with audio_processor and video_processor tools
        
        - **Architecture Impact:** 
          - Minimal - the category system is already comprehensive and flexible
          - The "Utilities and Helpers" category already contains the calculator and datetime tools requested
          - The "Specialized Domain Tools" category can accommodate media conversion tools
          - No changes needed to core architecture, just add new tools to existing categories
        
        - **External Research Needed:**
          - For audio processing: `symphonia` (pure Rust audio decoding), `rodio` (audio playback/processing)
          - For video processing: `ffmpeg-next` (Rust FFmpeg bindings), `gstreamer` (multimedia framework)
          - These would require careful sandboxing due to potential security risks with media file processing
        
    - [x] Todo: 
        - [x] Add `audio_processor` tool to "Specialized Domain Tools" category
        - [x] Add `video_processor` tool to "Specialized Domain Tools" category
        - [x] Update tool count in architecture from "40+ tools" to "42+ tools" (adding 2 media tools)
        - [x] No new categories needed - use existing structure
        - [x] Document security considerations for media processing in sandboxing section 

- [x] **Task/question 1.2** rethink Task 2.5.3: SemanticSearchTool and Task 2.5.2: CodeSearchTool
    - Task 2.5.3: SemanticSearchTool in `/TODO.md` for Phase 2 is attempting to use Retrieval Augmented Generation (RAG) and vector databases to achieve semantic search. However, architecturally, do these need to be part of the llmspell library itself rather than tools? 
        - e.g. should vector database be part of a distinct type of storage in llmspell-storage?
        - should RAG be part of a llmspell-tuiles or similar library?
        - what about indexing and searching? 
        - and looking at `/docs/technical/rs-llmspell-complete-architecture.md` all these are building blocks for agent memory.. how should that be accomplished?
        - then these tools just use the underlying library that is implemented - how does that change the implementation plan for SemanticSearchTool, CodeSearchTool and later agent/workflow/hook based memory implementations
    - How does the `/docs/technical/rs-llmspell-complete-architecture.md` need to change? 
    - does the change require changes to what was accomplished in `/doc/in-progress/PHASE*-DONE.md` what and how so?
    - does the change require changes to subsequent phases as documented in `/doc/in-progress/implementation-phases.md`, what and how so?
    - [x] Answer: 
        - **Current Architecture Analysis:**
          - `llmspell-storage` crate already exists (implemented in Phase 0) with `StorageBackend` trait
          - Storage is currently **key-value only** - no vector operations or search indexing support
          - Architecture mentions `Memory` trait and agent memory but no implementation details
          - SemanticSearchTool and CodeSearchTool are planned as complex tools in Phase 2
          - No dedicated RAG infrastructure, vector store abstraction, or search indexing exists
          - No code parsing infrastructure (tree-sitter, AST, symbol extraction) exists
        
        - **CodeSearchTool Analysis:**
          - Requires tree-sitter integration for code parsing
          - Needs symbol extraction and AST analysis
          - Requires search indexing (full-text search with ranking)
          - Git integration for repository analysis
          - **Decision: Keep self-contained** - too specialized for shared infrastructure
          - Common utilities can be extracted to llmspell-utils later if patterns emerge
        
        - **SemanticSearchTool Analysis:**
          - Requires embedding generation and vector operations
          - Needs similarity search and kNN queries
          - Foundation for agent memory and RAG patterns
          - **Decision: Add vector infrastructure** to llmspell-storage
        
        - **Architectural Recommendations:**
          - **Vector storage SHOULD be part of llmspell-storage** as a new storage type:
            - Create `VectorStorageBackend` trait with vector-specific operations
            - Implement in-memory, disk-based, and external DB adapters
            - Provides foundation for agent memory and semantic search
          - **RAG patterns SHOULD be in a new crate** (`llmspell-rag`):
            - Higher-level abstractions over vector storage and LLM integration
            - Document chunking, embedding strategies, retrieval patterns
            - Reusable by tools, agents, and workflows
          - **Code parsing remains tool-specific**:
            - CodeSearchTool includes its own tree-sitter integration
            - Symbol extraction and indexing specific to code search needs
            - Avoids premature abstraction of specialized functionality
          - **DEFER both tools to Phase 3.5**:
            - Both are complex and need proper infrastructure
            - Phase 2 should focus on simpler, immediately useful tools
            - Allows time to design vector storage and indexing properly
        
        - **Agent Memory Architecture:**
          - Short-term memory: existing key-value storage (already available)
          - Long-term memory: vector storage for semantic retrieval (Phase 3.5)
          - Episodic memory: combination with time-based indexing (future phase)
          - Working memory: in-process caching (already available)
        
        - **Changes Required:**
          - **Architecture document**: 
            - Add vector storage section to llmspell-storage chapter
            - Document agent memory architecture using storage layers
            - Add llmspell-rag crate to architecture overview
          - **Phase 0-1 (completed)**: No changes needed
          - **Phase 2 (current)**: 
            - REMOVE SemanticSearchTool and CodeSearchTool tasks
            - Focus on simpler tools that don't need new infrastructure
          - **New Phase 3.5: Vector Storage and Search Infrastructure**:
            - Implement VectorStorageBackend in llmspell-storage
            - Create llmspell-rag crate with RAG patterns
            - Implement SemanticSearchTool using vector storage
            - Implement CodeSearchTool with self-contained parsing
          - **Future phases**: Agent memory can build on vector storage
        
    - [x] Todo: 
        - [x] Remove SemanticSearchTool and CodeSearchTool from Phase 2 TODO.md
        - [x] Add "Phase 3.5: Vector Storage and Search Infrastructure" after Phase 3
        - [x] Update architecture to include VectorStorageBackend in llmspell-storage
        - [x] Add llmspell-rag crate specification to architecture
        - [x] Document agent memory architecture using storage layers
        - [x] Keep CodeSearchTool self-contained with its own tree-sitter integration
        - [x] Design SemanticSearchTool to use vector storage infrastructure 

### Section 2: Changes to architecture document and implementation plan
**Instructions** Read Section 1 questions, answers and todos and propose a detailed change to the the documents in the task sections outlined below: for each question below, first answer the first task `Changes to document`, then pause and ask if you should do the second part `Applied changes to document`, one at a time.
- [ ] **Task/question 2.1** changes to overall architecture in `/docs/technical/rs-llmspell-final-architecture.md` keeping in mind we've already accomplished last phase changes as documented in `/docs/in-progress/PHASE*-DONE.md` and `/TODO.md` which captures current Phase 2 done tasks.. Ensure the entire document makes holistic architecture sense, do not just add new sections, the updates may need to be interspersed across sections.
    - [x] **Task 2.1.1** Changes to document
      
      **1. Tool Catalog Updates (Section: Complete Built-in Tools Catalog)**
      - Update tool count from "40+ tools" to "42+ tools"
      - Add to "Specialized Domain Tools" category:
        - `audio_processor` - Audio file manipulation (format conversion, metadata extraction)
        - `video_processor` - Video file operations (format conversion, frame extraction)
      - Move `SemanticSearchTool` and `CodeSearchTool` to a new "Future Tools" note
      
      **2. Storage Abstraction Enhancement (Section: llmspell-storage crate)**
      - Add new subsection "Vector Storage Backend" after key-value storage:
        ```rust
        #[async_trait]
        pub trait VectorStorageBackend: Send + Sync {
            async fn insert_vector(&self, id: &str, vector: &[f32], metadata: Option<Value>) -> Result<()>;
            async fn search_similar(&self, query: &[f32], k: usize, threshold: Option<f32>) -> Result<Vec<(String, f32, Option<Value>)>>;
            async fn update_vector(&self, id: &str, vector: &[f32]) -> Result<()>;
            async fn delete_vector(&self, id: &str) -> Result<()>;
            async fn get_vector(&self, id: &str) -> Result<Option<(Vec<f32>, Option<Value>)>>;
        }
        ```
      - Note that vector storage will be implemented in Phase 3.5
      
      **3. New Crate Addition (Section: Crate Organization)**
      - Add `llmspell-rag` to the crate list (to be implemented in Phase 3.5):
        - Purpose: "RAG patterns and retrieval strategies"
        - Dependencies: llmspell-core, llmspell-storage, llmspell-providers
      
      **4. Agent Memory Architecture (New subsection under Agent trait section)**
      - Add "Agent Memory System" subsection explaining:
        - Short-term memory: Key-value storage (available now)
        - Long-term memory: Vector storage for semantic retrieval (Phase 3.5)
        - Episodic memory: Time-indexed combination (future)
        - Working memory: In-process state (available now)
      
      **5. Phase Overview Update (Section: Implementation Roadmap)**
      - Insert "Phase 3.5: Vector Storage and Search Infrastructure" after Phase 3
      - Update phase count and timeline estimates
      
      **6. Security Considerations (Section: Security Sandboxing)**
      - Add media file processing security notes:
        - Audio/video file validation requirements
        - Sandboxing for FFmpeg/media library calls
        - Resource limits for media processing
      
      **7. Feature Flags Update (Section: Cargo Features)**
      - Add planned feature flags:
        - `vector-storage` - Vector storage backends (Phase 3.5)
        - `media-tools` - Audio/video processing tools
        - `rag-patterns` - RAG functionality (Phase 3.5)
      
    - [x] **Task 2.1.2** Applied changes to document
      - ✅ Updated tool count from 40+ to 42+
      - ✅ Added audio_processor and video_processor to Specialized Domain Tools
      - ✅ Added VectorStorageBackend trait to storage abstraction section
      - ✅ Added llmspell-rag crate to crate organization
      - ✅ Added Agent Memory System section with multi-tiered architecture
      - ✅ Added Phase 3.5 to implementation roadmap
      - ✅ Added media processing security considerations
      - ✅ Added new feature flags (vector-storage, media-tools, rag-patterns)
      - ✅ Added note about deferred search tools 
- [ ] **Task/question 2.2** changes to implementation plan in `/docs/in-progress/implementation-phases.md` keeping in mind we've already accomplished last phase changes as documented in `/docs/in-progress/PHASE*-DONE.md` and `/TODO.md` which captures current Phase 2 done tasks. so retroactive changes to previous phases have to be rolled into future phases. Some changes may need to be done in the current phase which is Phase 2. Ensure the entire document makes sense from a dependency, scoping complexity perspective for each subsequent/yet to be done phase. Take into account the changes above in Task/question 2.1. do not just add new sections, the updates may need to be interspersed across sections for each phase.
    - [x] **Task 2.2.1** Changes to document
      
      **MAJOR RESTRUCTURING**: Based on comprehensive gap analysis, the phase organization needs complete overhaul to address the 39+ missing tools and proper dependency management.
      
      **1. Phase 2 Expansion (Current Phase - Weeks 5-6 → 5-8)**
      - **Focus**: All self-contained tools without external dependencies
      - **Categories to Complete**:
        - **Utilities & Helpers** (8 missing): calculator, text_manipulator, date_time_handler, uuid_generator, hash_calculator, base64_encoder, diff_calculator
        - **File System** (3 missing): file_watcher, file_converter, file_search  
        - **System Integration** (4 missing): environment_reader, process_executor, service_checker, system_monitor
        - **Simple Media** (3 missing): image_processor, audio_processor, video_processor (basic operations only)
      - **Update tool count**: From "12+ tools" to "26+ tools" (18 additional self-contained tools)
      - **Remove**: WebSearchTool, CodeSearchTool, SemanticSearchTool (move to later phases)
      - **Extend timeline**: 2 weeks → 4 weeks to accommodate 18 additional tools
      
      **2. New Phase 2.5 Addition (Weeks 9-10)**
      - **Focus**: Non-AI dependent tools with moderate complexity
      - **Web & Network** (5 missing): web_scraper, url_analyzer, api_tester, webhook_caller, webpage_monitor, sitemap_crawler
      - **Communication & APIs** (4 missing): email_sender, slack_integration, github_integration, database_connector
      - **Data Processing** (6 missing): xml_processor, yaml_processor, data_transformer, statistical_analyzer, text_analyzer, data_visualizer
      - **Web Search** (1 existing): WebSearchTool (move from Phase 2)
      - **Tool count**: 16+ tools
      - **Dependencies**: Phase 2 complete (uses basic utilities)
      
      **3. Phase 3: Workflow Orchestration (Weeks 11-12)**
      - **Unchanged scope** but timeline shifts
      - Dependencies: Phase 2.5 complete (workflows can use all basic tools)
      
      **4. Phase 3.5: Vector Storage and Search Infrastructure (Weeks 13-14)**
      - **Unchanged scope** from previous plan
      - Components: VectorStorageBackend, llmspell-rag, SemanticSearchTool, CodeSearchTool
      - Dependencies: Phase 3 complete (workflows needed for RAG patterns)
      
      **5. Phase Renumbering and Reorganization**
      - **Phase 4**: Production Readiness (Weeks 15-16) - shift from 11-12
      - **Phases 5-10**: Maintain existing scope but shift timeline by 4 weeks
      - **Phase 12**: JavaScript Engine Support (Weeks 23-24) - **MOVED from Phase 5**
      - **Phase 5.5 → Phase 11**: AI/ML Complex Tools (Weeks 25-26) - **RENAMED and EXPANDED**
        - **AI/ML Tools** (6 missing): text_summarizer, sentiment_analyzer, language_detector, text_classifier, named_entity_recognizer, embedding_generator
        - **Advanced Multimodal** (8 missing): image_analyzer, ocr_extractor, audio_transcriber, image_generator, media_converter, face_detector, scene_analyzer
        - **Dependencies**: Vector storage (Phase 3.5) + Production readiness (Phase 4)
      
      **6. Rationale for Reorganization**
      - **JavaScript moved later**: MCP integration (Phase 10) is more critical for tool ecosystem
      - **AI/ML tools grouped**: All require similar ML dependencies and model infrastructure
      - **Self-contained first**: Phase 2 becomes immediately useful with 26+ tools
      - **Dependency order**: Simple → Moderate → Complex → AI/ML
      
      **7. MVP Definition Update**
      - **MVP**: Phases 0-3 (now includes Phase 2.5)
      - **Rationale**: 42+ tools available after Phase 2.5 makes it genuinely useful
      - **Search capabilities**: Available in Phase 3.5 (post-MVP)
      
      **8. Timeline Impact**
      - **Total extension**: +4 weeks due to comprehensive tool implementation
      - **Phase 2**: 4 weeks (was 2) - 18 additional tools
      - **All phases 4+**: Shift by 4 weeks
      - **JavaScript delayed**: Reasonable since MCP/infrastructure more important
      
      **9. Feature Flag Updates**
      - **Phase 2**: `utilities-tools`, `system-tools`, `simple-media-tools`
      - **Phase 2.5**: `web-tools`, `communication-tools`, `data-analysis-tools`
      - **Phase 3.5**: `vector-storage`, `rag-patterns` (unchanged)
      - **Phase 11**: `ai-ml-tools`, `advanced-multimodal-tools`
      
    - [x] **Task 2.2.2** Applied changes to document
- [ ] **Task/question 2.3** changes to current phase implementation plan in `/TODO.md` and the design for it captured in `/docs/in-progress/phase-02-design-doc.md` keeping in mind we've already accomplished a portion of the tasks.  Ensure the entire document makes sense from a dependency, scoping complexity perspective, the detailed design and definition of done for each tasks and the definition of done for the entire Phase. Take into account the changes above in Task/question 2.1 and 2.2. and look at `/docs/in-progress/implementation-phases.md` to change/add new tasks. do not just add new sections, the updates may need to be interspersed across sections for each phase. make sure that DRY principle is applied, common library utilitilities should be put in the llmspell-utils crate and used for tool specific usage in the llmspell-tools crate. The designs that need to be taken out of phase-02-design-doc.md can be put into phase-02.5-design-doc.md (e.g the codesearchtool etc). focus on what changes we need to make to the `/docs/in-progress/phase-02-design-doc.md`
    - [x] **Task 2.3.1** Changes to phase-02-design-doc.md
      
      **MAJOR DESIGN DOC RESTRUCTURING**: Based on gap analysis, Phase 2 design doc needs updates to reflect 26+ self-contained tools focus.
      
      **1. Overview Section Updates**
      - **Timeline**: "Weeks 5-6 (10 working days)" → "Weeks 5-8 (14 working days)"
      - **Goal**: "12+ essential tools" → "26+ self-contained tools across all categories"
      - **Success Criteria**: Update from 12+ to 26+ tools
      - **Add Principle**: "Self-Contained First - No external dependencies in Phase 2"
      
      **2. Extract External/Complex Tool Designs to phase-02.5-design-doc.md**
      - **WebSearchTool**: Full specification (Section 1.3.1, 2.3, 3.1, 4 references)
      - **CodeSearchTool**: Full specification with tree-sitter integration
      - **SemanticSearchTool**: Full specification with vector storage
      - **Rationale**: These have external dependencies or complex infrastructure needs
      
      **3. Tool Catalog Updates (Section 1.3)**
      - **Remove**: WebSearchTool, CodeSearchTool, SemanticSearchTool from Search category
      - **Add New Categories**:
        - **Utilities & Helpers** (7 tools): TextManipulatorTool, UuidGeneratorTool, HashCalculatorTool, Base64EncoderTool, DiffCalculatorTool, DateTimeHandlerTool, CalculatorTool
        - **File System Extended** (3 tools): FileWatcherTool, FileConverterTool, FileSearchTool
        - **System Integration** (4 tools): EnvironmentReaderTool, ProcessExecutorTool, ServiceCheckerTool, SystemMonitorTool
        - **Simple Media** (3 tools): AudioProcessorTool, VideoProcessorTool, ImageProcessorTool
      
      **4. Add llmspell-utils Enhancement Section (New Section 1.4)**
      - **Common Utilities Design**:
        - Text processing functions (manipulation, regex, formatting)
        - Hash and encoding utilities (SHA, MD5, Base64, UUID generation)
        - File monitoring utilities (watchers, change detection)
        - System query utilities (env vars, process info, resource monitoring)
        - Date/time utilities (parsing, formatting, timezone handling)
      - **DRY Principle**: Tools use utils crate, maintain only tool-specific logic
      
      **5. Timeline Adjustments (Section 4)**
      - **Phase 2.1-2.2**: Days 1-3 (unchanged - provider enhancement, infrastructure)
      - **Phase 2.3**: Days 4-5 → "Utilities & Helpers Tools"
      - **Phase 2.4**: Days 6-7 → "Data Processing & File System Tools"
      - **Phase 2.5**: Day 8 → "System Integration Tools"
      - **Phase 2.6**: Day 9 → "Simple Media Tools"
      - **Phase 2.7**: Day 10 → "Common Utilities Enhancement"
      - **Phase 2.8**: Days 11-14 → "Integration, Testing, Documentation"
      
      **6. Implementation Specifications Updates (Section 2)**
      - **Add specifications for all 18 new tools**
      - **Include Tool trait implementations showing llmspell-utils usage**
      - **Security sandbox requirements for system tools**
      - **Resource limits for media processing tools**
      
      **7. Testing Strategy Updates (Section 3)**
      - **Expand test coverage for 26+ tools**
      - **Add system integration test scenarios**
      - **Media file processing test cases**
      - **Security boundary testing for system tools**
      
      **8. Risk Mitigation Updates (Section 5)**
      - **Remove "External API Dependencies" risk (moved to Phase 2.5)**
      - **Add "System Tool Security" risk and mitigation**
      - **Add "Media Processing Performance" risk**
      - **Update schedule risk to reflect 14-day timeline**
      
      **9. Dependencies Updates (Section 6)**
      - **Add crates**: `notify` (file watching), `encoding_rs` (encoding detection), `sysinfo` (system monitoring)
      - **Remove crates moved to Phase 2.5**: Complex ML/embedding dependencies
      - **Add internal dependency**: Heavy reliance on enhanced llmspell-utils
    - [x] **Task 2.3.2** Applied changes to document
      - ✅ Created phase-02.5-design-doc.md with extracted external tools
      - ✅ Updated phase-02-design-doc.md to 26+ self-contained tools
      - ✅ Updated TODO.md with new task structure (Phase 2.5-2.10)
      - ✅ Applied DRY principle with llmspell-utils enhancement

