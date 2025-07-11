# Phase 2 Tools Parameter Consistency Analysis

## Summary of Findings

### 1. Operation Parameter Usage
- **18 tools use "operation" parameter**: archive_handler, audio_processor, base64_encoder, calculator, csv_analyzer, date_time_handler, environment_reader, file_converter, file_operations, file_watcher, graphql_query, hash_calculator, image_processor, json_processor, system_monitor, text_manipulator, uuid_generator, video_processor
- **7 tools DO NOT use "operation" parameter**: data_validation, diff_calculator, file_search, http_request, process_executor, service_checker, template_engine, web_search

### 2. File Path Parameter Inconsistencies

**Multiple naming conventions for file paths:**
- `file_path`: audio_processor, image_processor, video_processor
- `input_path` + `output_path`: audio_processor, file_converter, image_processor
- `path`: file_operations, file_search
- `archive_path`: archive_handler
- `paths`: file_watcher

**Issue**: Media processors use both `file_path` AND `input_path`/`output_path`, creating confusion.

### 3. Input/Data/Content/Text Parameter Inconsistencies

**Different parameter names for similar data:**
- `text`: text_manipulator
- `content`: csv_analyzer, file_operations, json_processor
- `input`: base64_encoder, date_time_handler, json_processor
- `data`: data_validation, hash_calculator

**Issue**: json_processor uses BOTH `input` AND `content` parameters, which is confusing.

### 4. Diff Calculator Special Case
- Uses `old_text`/`new_text` for text diffs
- Uses `old_json`/`new_json` for JSON diffs
- Uses `type` parameter instead of `operation`

### 5. Category-Specific Inconsistencies

#### Data Processing Tools
- json_processor: uses `input` (object) and `content` (string)
- csv_analyzer: uses `content`
- data_validation: uses `data`
- **Inconsistent**: Same category, different parameter names

#### File System Tools
- file_operations: uses `path` and `content`
- file_search: uses `path` and `pattern`
- file_converter: uses `input_path` and `output_path`
- archive_handler: uses `archive_path` and `files`
- file_watcher: uses `paths` (plural)
- **Inconsistent**: Different naming for file paths

#### Media Processing Tools
- All use `file_path` for single operations
- All use `input_path`/`output_path` for conversion operations
- **Partially consistent** within category

#### Utility Tools
- base64_encoder: uses `input` and `input_file`/`output_file`
- hash_calculator: uses `data` and `file`
- text_manipulator: uses `text`
- template_engine: uses `template` and `context`
- diff_calculator: uses `old_text`/`new_text` or `old_json`/`new_json`
- **Very inconsistent**: Each tool has its own naming convention

#### API Tools
- http_request: uses `url`, `method`, `body`, `headers`
- graphql_query: uses `endpoint`, `query`, `variables`
- **Consistent** with their domain-specific needs

#### System Tools
- process_executor: uses `executable`, `arguments`
- environment_reader: uses `variable_name`, `value`
- service_checker: uses `target`, `check_type`
- system_monitor: only uses `operation`
- **Consistent** with their specific purposes

## Recommendations for Consistency

### 1. Standardize File Path Parameters
- Use `path` for single file/directory operations
- Use `source_path` and `target_path` for operations involving two paths
- Use `paths` (plural) for operations on multiple files

### 2. Standardize Input Data Parameters
- Use `input` for the primary data being processed
- Use `content` only for file content operations
- Use `data` only for structured data (JSON, CSV)
- Use `text` only for text-specific operations

### 3. Standardize Operation Parameters
- All tools that have multiple operations should use `operation`
- Tools with single purpose (like web_search, http_request) don't need `operation`

### 4. Category-Specific Standards

#### Data Processing
- Primary input: `data`
- Operation type: `operation`
- Format/schema: `format` or `schema`

#### File System
- File/directory path: `path`
- Multiple paths: `paths`
- Source/target: `source_path`, `target_path`

#### Media Processing
- Input file: `source_path`
- Output file: `target_path`
- Operation: `operation`

#### Utility Tools
- Primary input: `input`
- Operation: `operation` (if multiple operations)
- Options: `options`

### 5. Special Cases That Should Remain
- diff_calculator: `old_text`/`new_text` is clear and specific
- template_engine: `template` and `context` are domain-specific
- http_request: HTTP-specific parameters are appropriate
- graphql_query: GraphQL-specific parameters are appropriate

## Critical Issues to Fix

1. **json_processor** having both `input` and `content` parameters
2. **Media processors** having both `file_path` and `input_path` parameters
3. **Data processing tools** using different names (`data`, `content`, `input`) for the same concept
4. **File system tools** using inconsistent path parameter names

## Impact Assessment

- **High Impact**: Data processing and file system tools - used frequently, inconsistency causes confusion
- **Medium Impact**: Media processing tools - less frequently used but still confusing
- **Low Impact**: System and API tools - their domain-specific naming is actually helpful