# Document Processor Template

**Version:** 0.1.0
**Category:** Document
**Status:** Placeholder Implementation (Phase 12.4.4)

## Overview

The Document Processor template automates document extraction, transformation, and processing workflows. It handles PDF, OCR, document parsing, and intelligent transformation of content.

### What It Does

- **Multi-Format Extraction**: PDF, DOCX, images (via OCR), HTML, Markdown
- **Content Transformation**: Summarization, translation, format conversion
- **Intelligent Parsing**: Extract structured data from unstructured documents
- **Batch Processing**: Process multiple documents in parallel
- **Output Formatting**: Generate transformed documents in various formats

### Use Cases

- Invoice and receipt processing
- Legal document analysis
- Research paper extraction
- Form data extraction
- Document translation and summarization

---

## Quick Start

### CLI - Basic Usage

```bash
llmspell template exec document-processor \
  --param documents='["invoice.pdf"]' \
  --param transformation="extract"
```

### Lua - Basic Usage

```lua
local result = Template.execute("document-processor", {
    documents = {"contract.pdf", "terms.pdf"},
    transformation = "summarize"
})

print(result.result)
```

---

## Parameters

### Required Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `documents` | Array | List of document paths or URLs |
| `transformation` | Enum | Transformation: `extract`, `summarize`, `translate`, `convert` |

### Optional Parameters

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `output_format` | Enum | `"markdown"` | Output format: `markdown`, `json`, `html`, `pdf` |
| `language` | String | `"en"` | Target language for translation |
| `parallel` | Boolean | `true` | Process documents in parallel |
| `model` | String | `"ollama/llama3.2:3b"` | LLM for transformation |

**Inspect Full Schema:**
```bash
llmspell template schema document-processor
```

---

## Implementation Status

⚠️ **Note**: This template is a **placeholder implementation** as of Phase 12.4.4.

**Implemented:**
- ✅ Template metadata and parameter schema
- ✅ Parameter validation
- ✅ Cost estimation
- ✅ 12 comprehensive unit tests

**Placeholder/Pending:**
- ⏳ PDF extraction
- ⏳ OCR integration
- ⏳ Document parsing
- ⏳ Transformation logic
- ⏳ Multi-format output generation

**Expected**: Full implementation in Phase 14 (Advanced Templates)

---

## Output Format

### Extract Transformation
```json
{
  "result_type": "structured",
  "result": {
    "documents": [
      {
        "filename": "invoice.pdf",
        "extracted": {
          "invoice_number": "INV-2024-001",
          "total": 1250.00,
          "items": [...]
        }
      }
    ]
  },
  "artifacts": [
    {
      "filename": "invoice_data.json",
      "mime_type": "application/json"
    }
  ]
}
```

### Summarize Transformation
```json
{
  "result_type": "text",
  "result": "Summary:\nDocument 1: Contract terms for software licensing...\nDocument 2: Privacy policy outlining data handling...",
  "artifacts": [
    {
      "filename": "summary.md",
      "mime_type": "text/markdown"
    }
  ]
}
```

---

## Examples

### CLI Examples

#### Extract Data from PDF
```bash
llmspell template exec document-processor \
  --param documents='["invoices/march_2024.pdf"]' \
  --param transformation="extract" \
  --param output_format="json" \
  --output-dir ./extracted_data
```

#### Batch Summarization
```bash
llmspell template exec document-processor \
  --param documents='["doc1.pdf","doc2.pdf","doc3.pdf"]' \
  --param transformation="summarize" \
  --param parallel=true \
  --param output_format="markdown"
```

#### Document Translation
```bash
llmspell template exec document-processor \
  --param documents='["contract_en.pdf"]' \
  --param transformation="translate" \
  --param language="es" \
  --param output_format="pdf"
```

### Lua Examples

```lua
-- Process multiple documents
local result = Template.execute("document-processor", {
    documents = {
        "reports/q1_2024.pdf",
        "reports/q2_2024.pdf",
        "reports/q3_2024.pdf"
    },
    transformation = "summarize",
    output_format = "markdown",
    parallel = true
})

-- Save results
if result.artifacts then
    for i, artifact in ipairs(result.artifacts) do
        local filename = "summary_" .. i .. ".md"
        local file = io.open(filename, "w")
        file:write(artifact.content)
        file:close()
    end
end

print("Processed " .. #result.result.documents .. " documents")
print("Duration: " .. result.metrics.duration_ms .. "ms")
```

---

## Cost Estimation

```bash
llmspell template info document-processor --show-schema
```

### Estimated Costs (per document)

| Transformation | Pages | Tokens | Duration | Cost (USD) |
|---------------|-------|--------|----------|------------|
| Extract | 5 | ~2,500 | ~8s | $0.00025 |
| Summarize | 5 | ~3,500 | ~10s | $0.00035 |
| Translate | 5 | ~4,000 | ~12s | $0.00040 |
| Convert | 5 | ~1,500 | ~5s | $0.00015 |

**Parallel Processing**: ~2x faster for 3+ documents

---

## Troubleshooting

### Error: "Unsupported transformation: ocr"

**Cause**: Transformation type not in supported list

**Solution**: Use supported transformation:
```bash
--param transformation="extract"    # Supported
--param transformation="summarize"  # Supported
--param transformation="translate"  # Supported
--param transformation="convert"    # Supported
```

### Using Placeholder Implementation

**Current Behavior**: The template validates parameters but generates placeholder document processing results.

**Workaround**: For production document processing:
1. Use PyPDF2/pdfplumber for PDF extraction
2. Use Tesseract for OCR
3. Wait for Phase 14 full implementation

---

## Related Documentation

- [Template System Overview](../templates/README.md)
- [Research Assistant Template](./research-assistant.md) (production example)
- [Tool Integration](../../tools/README.md)

---

## Roadmap

### Phase 14 (Planned)
- PDF extraction with PyPDF2/pdfplumber
- OCR integration with Tesseract
- Document parsing and structuring
- AI-powered transformations
- Multi-format output generation
- Batch processing optimization

### Phase 15 (Future)
- Real-time document streaming
- Collaborative document processing
- Advanced OCR with layout detection
- Form auto-filling
- Document comparison

---

**Last Updated**: Phase 12.4.4 (Placeholder Implementation)
**Next Review**: Phase 14 (Advanced Templates)
