# Data Analysis Template

**Version:** 0.1.0
**Category:** Analysis
**Status:** Placeholder Implementation (Phase 12.4.2)

## Overview

The Data Analysis template automates statistical analysis, data visualization, and insight generation from structured datasets. It combines data processing tools, statistical analysis, and AI-powered interpretation.

### What It Does

- **Data Loading**: Support for CSV, JSON, Parquet, and database sources
- **Statistical Analysis**: Descriptive statistics, correlations, distributions
- **Visualization**: Charts, graphs, and interactive plots
- **AI Insights**: Natural language interpretation of findings
- **Report Generation**: Automated analysis reports

### Use Cases

- Business intelligence dashboards
- Research data analysis
- A/B test analysis
- Performance metrics analysis
- Financial data analysis

---

## Quick Start

### CLI - Basic Usage

```bash
llmspell template exec data-analysis \
  --param dataset="path/to/data.csv" \
  --param analysis_type="descriptive"
```

### Lua - Basic Usage

```lua
local result = Template.execute("data-analysis", {
    dataset = "sales_data.csv",
    analysis_type = "descriptive"
})

print(result.result)
```

---

## Parameters

### Required Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `dataset` | String | Path to dataset or data source URL |
| `analysis_type` | Enum | Type of analysis: `descriptive`, `comparative`, `predictive`, `exploratory` |

### Optional Parameters

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `visualization` | Boolean | `true` | Generate charts and graphs |
| `output_format` | Enum | `"markdown"` | Format: `markdown`, `html`, `pdf`, `json` |
| `confidence_level` | Float | `0.95` | Statistical confidence level (0.0-1.0) |
| `model` | String | `"ollama/llama3.2:3b"` | LLM for insight generation |

**Inspect Full Schema:**
```bash
llmspell template schema data-analysis
```

---

## Implementation Status

⚠️ **Note**: This template is a **placeholder implementation** as of Phase 12.4.2.

**Implemented:**
- ✅ Template metadata and parameter schema
- ✅ Parameter validation
- ✅ Cost estimation

**Placeholder/Pending:**
- ⏳ Data loading from multiple sources
- ⏳ Statistical analysis engine
- ⏳ Visualization generation
- ⏳ AI-powered insights
- ⏳ Report generation

**Expected**: Full implementation in Phase 14 (Advanced Templates)

---

## Output Format

```json
{
  "result_type": "structured",
  "result": {
    "summary": {
      "rows": 1000,
      "columns": 15,
      "missing_values": 23
    },
    "descriptive_stats": {
      "mean": 42.5,
      "median": 40.0,
      "std_dev": 12.3
    },
    "insights": [
      "Strong positive correlation between X and Y (r=0.85)",
      "Significant outliers detected in column Z"
    ]
  },
  "artifacts": [
    {
      "filename": "correlation_matrix.png",
      "mime_type": "image/png"
    },
    {
      "filename": "analysis_report.md",
      "mime_type": "text/markdown"
    }
  ]
}
```

---

## Examples

### CLI Examples

#### Descriptive Analysis
```bash
llmspell template exec data-analysis \
  --param dataset="sales_q4.csv" \
  --param analysis_type="descriptive" \
  --param visualization=true \
  --output-dir ./analysis_results
```

#### Comparative Analysis
```bash
llmspell template exec data-analysis \
  --param dataset="experiment_results.csv" \
  --param analysis_type="comparative" \
  --param confidence_level=0.99 \
  --param output_format="pdf"
```

### Lua Examples

```lua
local result = Template.execute("data-analysis", {
    dataset = "performance_metrics.csv",
    analysis_type = "exploratory",
    visualization = true,
    output_format = "html"
})

if result.artifacts then
    for _, artifact in ipairs(result.artifacts) do
        print("Generated: " .. artifact.filename)
    end
end
```

---

## Troubleshooting

### Using Placeholder Implementation

**Current Behavior**: The template validates parameters but generates placeholder analysis results.

**Workaround**: For production data analysis, consider:
1. Using pandas/numpy directly in Python
2. Using specialized analytics tools
3. Waiting for Phase 14 full implementation

---

## Related Documentation

- [Template System Overview](../templates/README.md)
- [Research Assistant Template](./research-assistant.md) (production example)
- [Tool Integration](../../tools/README.md)

---

## Roadmap

### Phase 14 (Planned)
- Complete data loading from multiple sources
- Statistical analysis engine
- Visualization generation (matplotlib/plotly)
- AI-powered insight generation
- Multi-format report generation

### Phase 15 (Future)
- Real-time data streaming analysis
- Machine learning integration
- Interactive dashboards
- Collaborative analysis

---

**Last Updated**: Phase 12.4.2 (Placeholder Implementation)
**Next Review**: Phase 14 (Advanced Templates)
