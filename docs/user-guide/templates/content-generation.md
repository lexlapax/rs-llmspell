# Content Generation Template

**Category**: Content
**Version**: 0.1.0
**Status**: Production Ready

## Overview

The Content Generation Template provides quality-driven content creation through a 4-agent pipeline with iterative improvement. The template creates content by planning, writing, reviewing, and formatting, using quality scoring to determine when content meets your standards.

**Key Features:**
- **4-Agent Pipeline**: Planner → Writer → Editor → Formatter for comprehensive content creation
- **Quality-Driven Iteration**: Editor scores content (0.0-1.0), iterates until quality threshold met
- **6 Content Types**: blog, documentation, marketing, technical, creative, general
- **5 Tone Options**: professional, casual, technical, persuasive, friendly
- **Target Length Control**: Specify word count (50-10,000 words)
- **Iteration Safety**: Max iterations (1-10) prevents infinite loops
- **Multiple Output Formats**: Markdown, HTML, JSON, plain text
- **Artifact Saving**: Content plan and final content saved automatically

## Quick Start

### Basic Blog Post Generation

Generate a blog post with default quality threshold (0.8):

```bash
llmspell template exec content-generation \
  --param topic="Getting Started with Rust Async Programming" \
  --param content_type=blog
```

### Technical Documentation with High Quality

Create technical documentation with strict quality requirements:

```bash
llmspell template exec content-generation \
  --param topic="REST API Implementation Guide" \
  --param content_type=documentation \
  --param tone=technical \
  --param quality_threshold=0.9 \
  --param target_length=2000
```

### Marketing Copy with Iteration Control

Generate marketing content with limited iterations:

```bash
llmspell template exec content-generation \
  --param topic="Product Launch Campaign" \
  --param content_type=marketing \
  --param tone=persuasive \
  --param max_iterations=5 \
  --param output_format=html
```

## Parameters

### Required Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `topic` | String | Content topic or title (min 3 chars) |

### Optional Parameters

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `content_type` | String | `"general"` | Content type: `blog`, `documentation`, `marketing`, `technical`, `creative`, `general` |
| `target_length` | Integer | - | Word count target (50-10,000) |
| `tone` | String | `"professional"` | Tone: `professional`, `casual`, `technical`, `persuasive`, `friendly` |
| `style_guide` | String | - | Custom style guidelines for content |
| `quality_threshold` | Float | `0.8` | Quality score threshold (0.0-1.0) for iteration termination |
| `max_iterations` | Integer | `3` | Maximum editing iterations (1-10) |
| `output_format` | String | `"markdown"` | Output format: `markdown`, `html`, `text`, `json` |
| `include_outline` | Boolean | `false` | Include planning outline in output |
| `model` | String | `"ollama/llama3.2:3b"` | LLM model override |

## Content Types

### 1. Blog (`blog`)

Optimized for engaging blog posts with reader retention.

**Planning Guidance:**
- Engaging introduction with hook
- Clear section structure with subheadings
- Practical examples and takeaways
- Strong call-to-action conclusion

**Best For:**
- Personal blogs
- Company blog posts
- Thought leadership articles
- Tutorial-style content

**Example:**
```bash
llmspell template exec content-generation \
  --param topic="10 Rust Performance Tips" \
  --param content_type=blog \
  --param tone=casual \
  --param target_length=1200
```

### 2. Documentation (`documentation`)

Structured for technical documentation with code examples.

**Planning Guidance:**
- Clear overview and prerequisites
- Step-by-step instructions
- Code examples and usage scenarios
- Troubleshooting section
- References and related resources

**Best For:**
- API documentation
- User guides
- Installation instructions
- Technical specifications

**Example:**
```bash
llmspell template exec content-generation \
  --param topic="Async/Await Error Handling Patterns" \
  --param content_type=documentation \
  --param tone=technical \
  --param output_format=markdown
```

### 3. Marketing (`marketing`)

Designed for conversion-focused marketing content.

**Planning Guidance:**
- Attention-grabbing headline
- Benefit-focused structure
- Social proof and testimonials
- Clear value proposition
- Strong call-to-action

**Best For:**
- Landing pages
- Email campaigns
- Sales copy
- Product descriptions

**Example:**
```bash
llmspell template exec content-generation \
  --param topic="Revolutionary CI/CD Platform Launch" \
  --param content_type=marketing \
  --param tone=persuasive \
  --param quality_threshold=0.85 \
  --param output_format=html
```

### 4. Technical (`technical`)

Structured for in-depth technical analysis and research.

**Planning Guidance:**
- Precise technical introduction
- Detailed methodology or approach
- Technical specifications and data
- Analysis and findings
- Conclusions and recommendations

**Best For:**
- Whitepapers
- Technical reports
- Research papers
- Architecture documents

**Example:**
```bash
llmspell template exec content-generation \
  --param topic="Microservices vs Monoliths: Performance Analysis" \
  --param content_type=technical \
  --param tone=technical \
  --param target_length=3000
```

### 5. Creative (`creative`)

Optimized for narrative and storytelling content.

**Planning Guidance:**
- Compelling narrative structure
- Character or theme development
- Descriptive and engaging language
- Emotional resonance
- Satisfying resolution

**Best For:**
- Short stories
- Brand narratives
- Case studies with storytelling
- Creative marketing

**Example:**
```bash
llmspell template exec content-generation \
  --param topic="Journey of a Solo Developer" \
  --param content_type=creative \
  --param tone=friendly \
  --param target_length=1500
```

### 6. General (`general`)

Balanced approach for general-purpose content.

**Planning Guidance:**
- Clear introduction
- Well-organized main points
- Supporting details and examples
- Logical flow and transitions
- Effective conclusion

**Best For:**
- General articles
- Explanatory content
- News-style writing
- Informational posts

**Example:**
```bash
llmspell template exec content-generation \
  --param topic="Understanding Cloud Computing Costs" \
  --param content_type=general
```

## Quality-Driven Iteration

The template uses a quality-driven iteration loop to ensure content meets your standards.

### How It Works

```
1. Plan Content → Outline, key points, audience
2. Write Draft → Initial content generation
3. Review Loop:
   ┌─────────────────────────────┐
   │ Editor Reviews Content      │
   │ Scores Quality (0.0-1.0)   │
   │ Provides Feedback          │
   └─────────────────────────────┘
           │
           ├─ Quality ≥ Threshold? → Exit Loop
           │
           ├─ Quality < Threshold?
           │   └─ Iterations < Max?
           │       ├─ Yes → Improve Content → Review Again
           │       └─ No  → Exit Loop
4. Format Output → Final formatting
```

### Quality Threshold Tuning

| Threshold | Description | Use Case |
|-----------|-------------|----------|
| **0.6-0.7** | Permissive | Quick drafts, brainstorming, high iteration count |
| **0.75-0.8** | Balanced (default) | General content, blog posts, documentation |
| **0.85-0.9** | Strict | Marketing copy, technical papers, public-facing content |
| **0.9-1.0** | Very Strict | Legal documents, critical communications (may never meet threshold) |

**Recommendation**: Start with default (0.8), adjust based on results.

### Iteration Budget

The `max_iterations` parameter controls the maximum number of review/improve cycles:

- **1-2 iterations**: Fast generation, lower quality ceiling
- **3 iterations (default)**: Good balance of quality and speed
- **5-7 iterations**: Higher quality, longer execution time
- **8-10 iterations**: Maximum quality focus, potentially slow

**Example with Iteration Control:**
```bash
llmspell template exec content-generation \
  --param topic="Critical Product Announcement" \
  --param quality_threshold=0.9 \
  --param max_iterations=7
```

## Output Formats

### Markdown (Default)

Human-readable markdown with optional outline:

```markdown
# How to Build a REST API in Rust

## Content Plan

1. Introduction to REST APIs
2. Setting up Rust environment
...

---

## Introduction

REST APIs are the backbone of modern web applications...

---

*Generated by LLMSpell Content Generation Template*
*Quality Score: 0.87 | Iterations: 2*
```

### HTML

Styled HTML document ready for web publishing:

```html
<!DOCTYPE html>
<html>
<head>
  <meta charset="utf-8">
  <title>How to Build a REST API in Rust</title>
</head>
<body>
  <details>
    <summary>Content Plan</summary>
    <pre>1. Introduction...</pre>
  </details>

  <h1>Introduction</h1>
  <p>REST APIs are the backbone...</p>
</body>
</html>
```

### JSON

Structured data with quality metrics:

```json
{
  "topic": "How to Build a REST API in Rust",
  "plan": {
    "outline": "1. Introduction...",
    "key_points": ["REST principles", "Rust advantages"],
    "target_audience": "Developers"
  },
  "content": "REST APIs are the backbone...",
  "quality_metrics": {
    "final_quality_score": 0.87,
    "iterations": 2,
    "word_count": 1543
  },
  "reviews": [
    {
      "quality_score": 0.75,
      "feedback": "Needs more examples",
      "strengths": ["Clear introduction"],
      "improvements": ["Add code snippets"]
    },
    {
      "quality_score": 0.87,
      "feedback": "Much improved",
      "strengths": ["Good examples", "Clear flow"],
      "improvements": []
    }
  ]
}
```

### Plain Text

Simple text output without formatting:

```
How to Build a REST API in Rust

REST APIs are the backbone of modern web applications...
```

## Use Cases

### 1. Blog Post Automation

Generate multiple blog posts from topic list:

```bash
# topics.txt contains one topic per line
while IFS= read -r topic; do
  llmspell template exec content-generation \
    --param topic="$topic" \
    --param content_type=blog \
    --param tone=casual \
    --param target_length=1000 \
    --param output_format=markdown > "blog-$(echo "$topic" | tr ' ' '-').md"
done < topics.txt
```

### 2. API Documentation Generation

Create technical documentation with high quality:

```bash
llmspell template exec content-generation \
  --param topic="REST API v2.0 Reference" \
  --param content_type=documentation \
  --param tone=technical \
  --param quality_threshold=0.85 \
  --param include_outline=true \
  --param output_format=markdown > api-docs.md
```

### 3. Product Launch Content

Generate marketing materials with iteration focus:

```bash
# Email copy
llmspell template exec content-generation \
  --param topic="Product Launch Announcement Email" \
  --param content_type=marketing \
  --param tone=persuasive \
  --param target_length=400 \
  --param quality_threshold=0.9 \
  --param max_iterations=5 \
  --param output_format=html > launch-email.html

# Landing page
llmspell template exec content-generation \
  --param topic="Product Landing Page Copy" \
  --param content_type=marketing \
  --param tone=persuasive \
  --param target_length=800 \
  --param quality_threshold=0.9 \
  --param max_iterations=5 \
  --param output_format=html > landing-page.html
```

### 4. Technical Report Generation

Create in-depth technical analysis:

```bash
llmspell template exec content-generation \
  --param topic="Cloud Migration Cost-Benefit Analysis" \
  --param content_type=technical \
  --param tone=technical \
  --param target_length=5000 \
  --param quality_threshold=0.85 \
  --param max_iterations=7 \
  --param output_format=markdown > technical-report.md
```

### 5. Content Series Creation

Generate a series of related articles:

```bash
# Define series topics
TOPICS=(
  "Introduction to Microservices"
  "Microservices Communication Patterns"
  "Microservices Data Management"
  "Microservices Deployment Strategies"
)

# Generate each article
for i in "${!TOPICS[@]}"; do
  llmspell template exec content-generation \
    --param topic="${TOPICS[$i]}" \
    --param content_type=blog \
    --param tone=professional \
    --param target_length=1500 \
    --param output_format=markdown > "article-$((i+1)).md"
done
```

## Lua Script Integration

### Basic Content Generation

```lua
local Template = require("llmspell.template")

-- Generate blog post
local result = Template.execute("content-generation", {
    topic = "Rust Error Handling Best Practices",
    content_type = "blog",
    tone = "professional",
    target_length = 1200
})

print("Generated Content:")
print(result.output)
print(string.format("Quality: %.2f", result.metadata.quality_score))
print(string.format("Iterations: %d", result.metadata.iterations))
```

### Quality Monitoring Workflow

```lua
local Template = require("llmspell.template")

local topics = {
    "Introduction to Async Rust",
    "Error Handling in Rust",
    "Rust Memory Safety"
}

local quality_threshold = 0.85
local low_quality_topics = {}

for _, topic in ipairs(topics) do
    print(string.format("Generating: %s", topic))

    local result = Template.execute("content-generation", {
        topic = topic,
        content_type = "blog",
        quality_threshold = quality_threshold,
        max_iterations = 5,
        output_format = "json"
    })

    local data = json.decode(result.output)
    local final_quality = data.quality_metrics.final_quality_score

    print(string.format("  Quality: %.2f", final_quality))

    if final_quality < quality_threshold then
        table.insert(low_quality_topics, {
            topic = topic,
            quality = final_quality
        })
    end

    -- Save content
    local filename = string.format("%s.md", topic:gsub(" ", "-"))
    local f = io.open(filename, "w")
    f:write(data.content)
    f:close()
end

-- Report low quality topics
if #low_quality_topics > 0 then
    print("\nTopics requiring manual review:")
    for _, item in ipairs(low_quality_topics) do
        print(string.format("  - %s (quality: %.2f)", item.topic, item.quality))
    end
end
```

### Batch Generation with Custom Style

```lua
local Template = require("llmspell.template")

local style_guide = [[
Writing Style Guide:
- Use active voice
- Short paragraphs (3-4 sentences)
- Include code examples with explanations
- Avoid jargon without definitions
- End with actionable takeaways
]]

local topics = {
    "Understanding Rust Lifetimes",
    "Async Rust Patterns",
    "Building CLI Tools in Rust"
}

for _, topic in ipairs(topics) do
    local result = Template.execute("content-generation", {
        topic = topic,
        content_type = "documentation",
        tone = "technical",
        style_guide = style_guide,
        quality_threshold = 0.8,
        max_iterations = 4,
        include_outline = true
    })

    -- Save to file
    local filename = string.format("docs/%s.md", topic:gsub(" ", "-"):lower())
    local f = io.open(filename, "w")
    f:write(result.output)
    f:close()

    print(string.format("Generated: %s (quality: %.2f)", filename, result.metadata.quality_score))
end
```

### Progressive Quality Improvement

```lua
local Template = require("llmspell.template")

-- Try with progressively higher quality thresholds
local thresholds = {0.7, 0.8, 0.85, 0.9}
local topic = "Advanced Rust Concurrency"

for _, threshold in ipairs(thresholds) do
    print(string.format("\nAttempting quality threshold: %.2f", threshold))

    local result = Template.execute("content-generation", {
        topic = topic,
        content_type = "technical",
        quality_threshold = threshold,
        max_iterations = 7,
        output_format = "json"
    })

    local data = json.decode(result.output)
    local final_quality = data.quality_metrics.final_quality_score
    local iterations = data.quality_metrics.iterations

    print(string.format("  Achieved: %.2f after %d iterations", final_quality, iterations))

    -- Save best result
    if final_quality >= threshold then
        local f = io.open(string.format("content-q%.2f.md", threshold), "w")
        f:write(data.content)
        f:close()
        print("  ✓ Threshold met, saved content")
    else
        print("  ✗ Threshold not met")
    end
end
```

## Performance Considerations

### Execution Time

Content generation time scales with iterations and target length:

| Config | Estimated Time* |
|--------|-----------------|
| 500 words, 1-2 iterations | ~30-45 seconds |
| 1000 words, 3 iterations (default) | ~1-2 minutes |
| 2000 words, 5 iterations | ~2-4 minutes |
| 5000 words, 7 iterations | ~5-8 minutes |

*Times vary based on LLM model speed and hardware

### Optimization Tips

1. **Start with Lower Threshold**
   ```bash
   --param quality_threshold=0.75  # Faster, good for drafts
   ```

2. **Limit Iterations for Speed**
   ```bash
   --param max_iterations=2  # 2x faster than default
   ```

3. **Use Local Models for Iteration**
   ```bash
   --param model=ollama/llama3.2:3b  # Faster than API calls
   ```

4. **Batch Similar Content**
   ```bash
   # Generate multiple articles in parallel
   for topic in "${topics[@]}"; do
       llmspell template exec content-generation --param topic="$topic" &
   done
   wait
   ```

5. **Adjust Target Length Realistically**
   ```bash
   --param target_length=800  # Smaller target = faster generation
   ```

### Resource Usage

**Token Consumption Estimates:**
- Base (plan + write + format): ~2,600 tokens
- Per iteration (review + improve): ~2,000 tokens each
- Default (3 iterations): ~7,400 tokens
- Maximum (10 iterations): ~21,400 tokens

**Memory Usage:**
- Minimal: Template state ~100 KB
- Content artifacts: Varies with target_length (1KB-50KB)

## Troubleshooting

### Common Issues

**Issue**: Content quality never reaches threshold
```
Warning: Quality score 0.78, threshold 0.9, max iterations reached
```
**Solution**:
- Lower quality_threshold to realistic level (0.75-0.85)
- Increase max_iterations to 5-7
- Provide more specific topic description
- Add custom style_guide with clear requirements

---

**Issue**: Content is too short/long
```
Generated 2500 words, expected 1000
```
**Solution**:
- LLMs may overshoot/undershoot target_length
- Adjust target_length with ~20% buffer
- Use custom style_guide to emphasize word count:
  ```bash
  --param style_guide="Content must be exactly 1000 words"
  ```

---

**Issue**: Iterations always max out
```
All content reached max_iterations without meeting threshold
```
**Solution**:
- Quality threshold too high for content_type
- Try different model with --param model=...
- Simplify topic or make more specific
- Review quality_threshold expectations (0.8 is balanced default)

---

**Issue**: Slow execution with local models
```
Generation took 8 minutes for 1500 words
```
**Solution**:
- Use faster model: `ollama/llama3.2:3b` (faster) vs `ollama/llama3.1:70b` (slower but higher quality)
- Reduce max_iterations from default 3 to 2
- Lower quality_threshold slightly to reduce iteration count
- Consider cloud models (OpenAI, Anthropic) for production

---

**Issue**: Output format not as expected
```
Expected HTML but got plain text
```
**Solution**:
- Verify output_format parameter: `--param output_format=html`
- Check valid formats: markdown, html, text, json
- Ensure no shell redirection interfering with output

---

**Issue**: Content doesn't match tone
```
Expected casual tone but output is very formal
```
**Solution**:
- Tone is a suggestion, not guarantee
- Add explicit tone guidance in style_guide:
  ```bash
  --param style_guide="Use casual, conversational language. Avoid formal phrases."
  ```
- Try different models (some better at tone control)
- Increase quality_threshold to iterate more on tone

### Getting Help

- **Documentation**: `/docs/user-guide/templates/` for detailed guides
- **Issues**: Report bugs at https://github.com/lexlapax/rs-llmspell/issues
- **Examples**: Check `examples/templates/content-generation/` for sample scripts

## Best Practices

1. **Start Conservative**: Use default quality_threshold (0.8) and max_iterations (3), adjust based on results
2. **Provide Context**: More specific topics produce better content ("Rust Async Error Handling Patterns" vs "Rust Errors")
3. **Use Style Guides**: Custom style_guide parameter significantly improves output consistency
4. **Monitor Quality**: Use JSON output to track quality_score trends across multiple generations
5. **Iterate Parameters**: Experiment with different content_type, tone, and threshold combinations
6. **Batch Wisely**: Generate multiple pieces in parallel but monitor system resources
7. **Save Artifacts**: Artifact files (content-plan.md, generated-content.md) are saved to output directory
8. **Choose Right Format**: Use markdown for readability, JSON for programmatic processing, HTML for web publishing

## Advanced Configuration

### Environment-Specific Quality Thresholds

```lua
local env = os.getenv("ENV") or "development"

local quality_map = {
    development = 0.7,   -- Fast iteration
    staging = 0.8,       -- Balanced
    production = 0.9     -- High quality
}

Template.execute("content-generation", {
    topic = "Product Documentation",
    quality_threshold = quality_map[env] or 0.8
})
```

### Custom Model Per Content Type

```lua
local content_configs = {
    blog = { model = "ollama/llama3.2:3b", quality_threshold = 0.75 },
    documentation = { model = "openai/gpt-4", quality_threshold = 0.85 },
    marketing = { model = "anthropic/claude-3-sonnet", quality_threshold = 0.9 }
}

for content_type, config in pairs(content_configs) do
    Template.execute("content-generation", {
        topic = "Example Topic",
        content_type = content_type,
        model = config.model,
        quality_threshold = config.quality_threshold
    })
end
```

### Conditional Iteration Based on Quality

```lua
local topic = "Critical Announcement"
local result = Template.execute("content-generation", {
    topic = topic,
    content_type = "marketing",
    quality_threshold = 0.85,
    max_iterations = 3,
    output_format = "json"
})

local data = json.decode(result.output)

-- If quality not met, regenerate with more iterations
if data.quality_metrics.final_quality_score < 0.85 then
    print("Quality not met, regenerating with higher iteration budget...")
    result = Template.execute("content-generation", {
        topic = topic,
        content_type = "marketing",
        quality_threshold = 0.85,
        max_iterations = 7  -- Increased from 3
    })
end
```

## Future Enhancements

Planned features for upcoming versions:

- [ ] Multi-section generation (chapters, series)
- [ ] SEO optimization mode (keyword density, meta descriptions)
- [ ] Plagiarism checking integration
- [ ] Readability score tracking (Flesch-Kincaid)
- [ ] Multi-language content generation
- [ ] A/B testing support (generate variants)
- [ ] Citation and reference management
- [ ] Image/diagram suggestion integration

## See Also

- [Code Generator Template](code-generator.md) - Generate code from specifications
- [Document Processor Template](document-processor.md) - Process and transform documents
- [Workflow Orchestrator Template](workflow-orchestrator.md) - Custom content generation workflows
- [Research Assistant Template](research-assistant.md) - Research topics for content ideas
