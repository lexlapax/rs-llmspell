## TODOS Handoff to Phase 2 
**checks for architecture consistency, implementation consistency, gaps between phases and late breaking requirements**
Each Section of this document comes with it's own **Instructions** read that for a section at a time. Ask me after you're read the entirety of this document.

### Section 1: Check Architecture for and late breaking requirement
**Instructions** clear all your internal todos first. we're ready to transition to `Phase 2`
now I'm going to ask you a series of questions about the architecture documented in `/docs/technical/rs-llmspell-complete-architecture.md` . for each question, document the question in Section 1 of this document. for each question, document the question under the Task/question Section 1 heading. for that question review the architecture document and find the answer and document the answer under - [ ] Answer: . if you do not find the answer, document it as such. if you figure out what needs to be done, document in - [ ] Todo: subtask.Once that's done, prompt me for another answer. To understand this read this TODO.md document and make sure you understand the section **Task 1** and what I'm saying here.  think hard and reason. do what I'm asking in this prompt and come back and tell me that you understand.
- [ ] **Task/question 1.1** convenience model names in provider calls
    - Does the architecture allow scripts to call llms with short naming syntax `providername/modelname` and base_url overrides? e.g. 
            ```Lua
            local agent = Agent.new({
                name = "research_assistant",
                model = "openai/llama3.5"
                base_url = "http://localhost:11434"
            })``` where provider=openai and model=llama3.5, and base_url="http://localhost:11434" 
            or 
            ```Lua
            local agent = Agent.new({
                name = "collater",
                model = "openrouter/deepseek/deepseek-r1-0528"
                base_url = "http://localhost:11434"
            })``` where provider=openrouter and model=deepseek/deepseek-r1-0528, and base_url="http://localhost:11434" 
        where the provider is automatically derived from the model name? if not what needs to happen to change that behaviour? should it be done at the llm or the provider abstraction layer so that it propagates all the way back out?
    - Do the subsequent layers above? core, agent, tools, workflow, and above that engine, cli, repl allow this?
    - do you need to do external research/web research for this question?
    - How does the `/docs/technical/rs-llmspell-complete-architecture.md` need to change? 
    - does the change require changes to what was accomplished in `/doc/in-progress/PHASE*-DONE.md`, what and how so?
    - does the change require changes to subsequent phases as documented in `/doc/in-progress/implementation-phases.md`, what and how so?
    - [ ] Answer: 
        - 
        - 
    - [ ] Todo: 
        - [ ] 
        - [ ] 
- [ ] **Task/question 1.2** Question 2
    - Does the architecture allow llms to accomodate streaming requirements for clients?
    - Do the subsequent layers above? core, agent, tools, workflow, and above that engine, cli, repl allow for streaming?
    - do you need to do external research/web research for this question?
    - How does the `/docs/technical/rs-llmspell-complete-architecture.md` need to change? 
    - does the change require changes to what was accomplished in `/doc/in-progress/PHASE00-DONE.md`, what and how so?
    - does the change require changes to subsequent phases as documented in `/doc/in-progress/implementation-phases.md`, what and how so?
    - [ ] Answer: 
        - 
        - 
    - [ ] Todo: 
        - [ ] 
        - [ ] 

### Section 2: Changes to architecture document and implementation plan
**Instructions** Read Section 1 questions, answers and todos and propose a detailed change to the the documents in the task sections outlined below: for each question below, first answer the first task `Changes to document`, then pause and ask if you should do the second part `Applied changes to document`, one at a time.
- [ ] **Task/question 2.1** changes to overall architecture in `/docs/technical/rs-llmspell-final-architecture.md` keeping in mind we've already accomplished last phase changes as documented in `/docs/in-progress/PHASE01-DONE.md`. Ensure the entire document makes holistic architecture sense, do not just add new sections, the updates may need to be interspersed across sections.
    - [ ] **Task 2.1.1** Changes to document
        - 
        - 
    - [ ] **Task 2.1.2** Applied changes to document <date>
- [ ] **Task/question 2.2** changes to implementation plan in `/docs/in-progress/implementation-phases.md` keeping in mind we've already accomplished last phase changes as documented in `/docs/in-progress/PHASE01-DONE.md`. Ensure the entire document makes sense from a dependency, scoping complexity perspective for each subsequent/yet to be done phase. Take into account the changes above in Task/question 2.1. do not just add new sections, the updates may need to be interspersed across sections for each phase.
    - [ ] **Task 2.2.1** Changes to document
        - 
        - 
    - [ ] **Task 2.2.2** Applied changes to document <date>


### Section 3: Transition to next phase
**Instructions** Read Section 1 and section 2 above and complete the following tasks/questions: 
- [ ] **Task/question 3.1** Based on the `/docs/technical/rs-llmspell-final-architecture.md`, and the `/docs/in-progress/implemenation-phases.md`, are you ready to create a detailed design doc for phase 02 `/docs/in-progress/phase-02-design-doc.md` in the style of `/docs/in-progress/phase-00-design-doc.md` and a todo list `/docs/in-progress/PHASE02-TODO.md` in the style of `/docs/in-progress/PHASE00-TODO.md`? if not what needs to be done? you can also look at output of previous phase `/docs/in-progress/PHASE00-TODO-DONE.md`, `/docs/in-progress/PHASE01_HANDOFF_PACKAGE.md` and `/docs/in-progress/PHASE01_KNOWLEDGE_TRANSFER.md` to inform your answers and todos.
    - [ ] Answer: 
        - 
        - 
    - [ ] Todo: 
        - [ ] Todo 1
        - [ ] Todo 2
        - [ ] Create `/docs/in-progress/phase-02-design-doc.md` - COMPLETED <date>
        - [ ] Create `/docs/in-progress/PHASE02-TODO.md` - COMPLETED <date>
