## TODOS Change of Plans for Phase 2 
**checks for architecture consistency, implementation consistency, gaps between phases and late breaking requirements**
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
    - [ ] Answer: 
        - 
        - 
    - [ ] Todo: 
        - [ ] 
        - [ ] 

- [ ] **Task/question 1.2** rethink Task 2.5.3: SemanticSearchTool and Task 2.5.2: CodeSearchTool
    - Task 2.5.3: SemanticSearchTool in `/TODO.md` for Phase 2 is attempting to use Retrieval Augmented Generation (RAG) and vector databases to achieve semantic search. However, architecturally, do these need to be part of the llmspell library itself rather than tools? 
        - e.g. should vector database be part of a distinct type of storage in llmspell-storage?
        - should RAG be part of a llmspell-tuiles or similar library?
        - what about indexing and searching? 
        - and looking at `/docs/technical/rs-llmspell-complete-architecture.md` all these are building blocks for agent memory.. how should that be accomplished?
        - then these tools just use the underlying library that is implemented - how does that change the implementation plan for SemanticSearchTool, CodeSearchTool and later agent/workflow/hook based memory implementations
    - How does the `/docs/technical/rs-llmspell-complete-architecture.md` need to change? 
    - does the change require changes to what was accomplished in `/doc/in-progress/PHASE*-DONE.md` what and how so?
    - does the change require changes to subsequent phases as documented in `/doc/in-progress/implementation-phases.md`, what and how so?
    - [ ] Answer: 
        - 
        - 
    - [ ] Todo: 
        - [ ] 
        - [ ] 

### Section 2: Changes to architecture document and implementation plan
**Instructions** Read Section 1 questions, answers and todos and propose a detailed change to the the documents in the task sections outlined below: for each question below, first answer the first task `Changes to document`, then pause and ask if you should do the second part `Applied changes to document`, one at a time.
- [ ] **Task/question 2.1** changes to overall architecture in `/docs/technical/rs-llmspell-final-architecture.md` keeping in mind we've already accomplished last phase changes as documented in `/docs/in-progress/PHASE*-DONE.md` and `/TODO.md` which captures current Phase 2 done tasks.. Ensure the entire document makes holistic architecture sense, do not just add new sections, the updates may need to be interspersed across sections.
    - [ ] **Task 2.1.1** Changes to document
    - [ ] **Task 2.1.2** Applied changes to document 
- [ ] **Task/question 2.2** changes to implementation plan in `/docs/in-progress/implementation-phases.md` keeping in mind we've already accomplished last phase changes as documented in `/docs/in-progress/PHASE*-DONE.md` and `/TODO.md` which captures current Phase 2 done tasks. so retroactive changes to previous phases have to be rolled into future phases. Some changes may need to be done in the current phase which is Phase 2. Ensure the entire document makes sense from a dependency, scoping complexity perspective for each subsequent/yet to be done phase. Take into account the changes above in Task/question 2.1. do not just add new sections, the updates may need to be interspersed across sections for each phase.
    - [ ] **Task 2.2.1** Changes to document
    - [ ] **Task 2.2.2** Applied changes to document
- [ ] **Task/question 2.3** changes to current phase implementation plan in `/TODO.md` keeping in mind we've already accomplished a portion of the tasks. s Ensure the entire document makes sense from a dependency, scoping complexity perspective for each subsequent/yet to be done phase. Take into account the changes above in Task/question 2.1 and 2.2. do not just add new sections, the updates may need to be interspersed across sections for each phase.
    - [ ] **Task 2.2.1** Changes to document
    - [ ] **Task 2.2.2** Applied changes to document

