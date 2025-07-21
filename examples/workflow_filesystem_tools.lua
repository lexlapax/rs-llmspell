-- ABOUTME: File system tools integration with workflows
-- ABOUTME: Demonstrates all 8 file system tools in workflow patterns

print("=== File System Tools Workflow Examples ===\n")

-- Example 1: File Processing Pipeline
print("--- Example 1: File Processing Pipeline ---")

local file_pipeline = Workflow.sequential({
    name = "file_processing_pipeline",
    description = "Process files through multiple stages",
    error_strategy = "fail_fast",
    steps = {
        -- Step 1: Search for files
        {
            name = "find_files",
            type = "tool",
            tool = "file_search",
            input = {
                pattern = "*.txt",
                directory = "./examples",
                recursive = true
            }
        },
        -- Step 2: Read file contents
        {
            name = "read_file",
            type = "tool",
            tool = "file_operations",
            input = {
                operation = "read",
                path = "./examples/sample.txt"
            }
        },
        -- Step 3: Convert file format
        {
            name = "convert_format",
            type = "tool",
            tool = "file_converter",
            input = {
                input_path = "./examples/sample.txt",
                output_format = "md",
                output_path = "./examples/sample.md"
            }
        },
        -- Step 4: Create archive
        {
            name = "create_archive",
            type = "tool",
            tool = "archive_handler",
            input = {
                operation = "create",
                archive_path = "./examples/output.zip",
                files = ["./examples/sample.txt", "./examples/sample.md"]
            }
        }
    }
})

-- Example 2: Parallel File Operations
print("\n--- Example 2: Parallel File Operations ---")

local parallel_file_ops = Workflow.parallel({
    name = "parallel_file_operations",
    description = "Perform multiple file operations concurrently",
    max_concurrency = 4,
    branches = {
        {
            name = "branch_copy",
            steps = {{
                name = "copy_files",
                type = "tool",
                tool = "file_operations",
                input = {
                    operation = "copy",
                    source = "./examples/source.txt",
                    destination = "./examples/backup/source.txt"
                }
            }}
        },
        {
            name = "branch_search",
            steps = {{
                name = "search_logs",
                type = "tool",
                tool = "file_search",
                input = {
                    pattern = "*.log",
                    directory = "./logs",
                    content_pattern = "ERROR"
                }
            }}
        },
        {
            name = "branch_watch",
            steps = {{
                name = "watch_directory",
                type = "tool",
                tool = "file_watcher",
                input = {
                    path = "./watched",
                    events = ["create", "modify", "delete"],
                    duration = 5
                }
            }}
        },
        {
            name = "branch_archive",
            steps = {{
                name = "extract_archive",
                type = "tool",
                tool = "archive_handler",
                input = {
                    operation = "extract",
                    archive_path = "./examples/data.zip",
                    destination = "./examples/extracted"
                }
            }}
        }
    }
})

-- Example 3: Conditional File Processing
print("\n--- Example 3: Conditional File Processing ---")

-- Set up conditions
State.set("file_size", "large")

local conditional_file = Workflow.conditional({
    name = "conditional_file_processor",
    description = "Process files based on size",
    branches = {
        {
            name = "large_file_branch",
            condition = {
                type = "shared_data_equals",
                key = "file_size",
                expected = "large"
            },
            steps = {
                {
                    name = "split_file",
                    type = "tool",
                    tool = "file_operations",
                    input = {
                        operation = "split",
                        path = "./examples/large_file.dat",
                        chunk_size = 1048576 -- 1MB chunks
                    }
                },
                {
                    name = "archive_chunks",
                    type = "tool",
                    tool = "archive_handler",
                    input = {
                        operation = "create",
                        archive_path = "./examples/chunks.tar.gz",
                        compression = "gzip"
                    }
                }
            }
        },
        {
            name = "small_file_branch",
            condition = {
                type = "shared_data_equals",
                key = "file_size",
                expected = "small"
            },
            steps = {
                {
                    name = "process_directly",
                    type = "tool",
                    tool = "file_converter",
                    input = {
                        input_path = "./examples/small_file.txt",
                        output_format = "pdf"
                    }
                }
            }
        }
    },
    default_branch = {
        name = "medium_file_branch",
        steps = {
            {
                name = "standard_processing",
                type = "tool",
                tool = "file_operations",
                input = {
                    operation = "read",
                    path = "./examples/medium_file.txt"
                }
            }
        }
    }
})

-- Example 4: File Monitoring Loop
print("\n--- Example 4: File Monitoring Loop ---")

local file_monitor_loop = Workflow.loop({
    name = "file_monitor_loop",
    description = "Monitor directory for changes",
    iterator = "range",
    start = 1,
    ["end"] = 5,
    step = 1,
    body = {
        {
            name = "check_directory",
            type = "tool",
            tool = "file_watcher",
            input = {
                path = "./monitored",
                events = ["create", "modify"],
                duration = 2
            }
        },
        {
            name = "process_changes",
            type = "tool",
            tool = "file_search",
            input = {
                pattern = "*.new",
                directory = "./monitored",
                created_after = "{{loop:iteration_start}}"
            }
        }
    },
    break_conditions = {
        {
            type = "shared_data_equals",
            key = "stop_monitoring",
            expected = true
        }
    }
})

-- Example 5: Advanced File Pipeline with Error Handling
print("\n--- Example 5: Advanced File Pipeline ---")

local advanced_pipeline = Workflow.sequential({
    name = "advanced_file_pipeline",
    description = "Complex file processing with error handling",
    error_strategy = "continue",
    steps = {
        -- Create test directory structure
        {
            name = "setup_directories",
            type = "tool",
            tool = "file_operations",
            input = {
                operation = "mkdir",
                path = "./examples/test_pipeline",
                recursive = true
            }
        },
        -- Write test file
        {
            name = "create_test_file",
            type = "tool",
            tool = "file_operations",
            input = {
                operation = "write",
                path = "./examples/test_pipeline/input.txt",
                content = "Test data for pipeline processing"
            }
        },
        -- Search and list files
        {
            name = "list_files",
            type = "tool",
            tool = "file_search",
            input = {
                pattern = "*",
                directory = "./examples/test_pipeline"
            }
        },
        -- Convert to multiple formats in parallel
        {
            name = "convert_formats",
            type = "workflow",
            workflow = Workflow.parallel({
                name = "format_conversions",
                branches = {
                    {
                        name = "to_markdown",
                        steps = {{
                            name = "convert_md",
                            type = "tool",
                            tool = "file_converter",
                            input = {
                                input_path = "./examples/test_pipeline/input.txt",
                                output_format = "md"
                            }
                        }}
                    },
                    {
                        name = "to_html",
                        steps = {{
                            name = "convert_html",
                            type = "tool",
                            tool = "file_converter",
                            input = {
                                input_path = "./examples/test_pipeline/input.txt",
                                output_format = "html"
                            }
                        }}
                    }
                }
            })
        },
        -- Archive all outputs
        {
            name = "create_final_archive",
            type = "tool",
            tool = "archive_handler",
            input = {
                operation = "create",
                archive_path = "./examples/test_pipeline/output.zip",
                directory = "./examples/test_pipeline",
                exclude_patterns = ["*.zip"]
            }
        },
        -- Verify archive
        {
            name = "verify_archive",
            type = "tool",
            tool = "archive_handler",
            input = {
                operation = "list",
                archive_path = "./examples/test_pipeline/output.zip"
            }
        }
    }
})

-- Example 6: File System Cleanup Workflow
print("\n--- Example 6: File System Cleanup ---")

local cleanup_workflow = Workflow.sequential({
    name = "filesystem_cleanup",
    description = "Clean up temporary files and old backups",
    steps = {
        -- Find old files
        {
            name = "find_old_files",
            type = "tool",
            tool = "file_search",
            input = {
                pattern = "*.tmp",
                directory = "/tmp",
                modified_before = "7d" -- Files older than 7 days
            }
        },
        -- Delete old files
        {
            name = "delete_old_files",
            type = "tool",
            tool = "file_operations",
            input = {
                operation = "delete",
                paths = "{{step:find_old_files}}"
            }
        },
        -- Compress logs
        {
            name = "compress_logs",
            type = "tool",
            tool = "archive_handler",
            input = {
                operation = "create",
                archive_path = "./backups/logs_{{date}}.tar.gz",
                directory = "./logs",
                compression = "gzip"
            }
        }
    }
})

-- Hook integration for file operations
advanced_pipeline:onBeforeExecute(function(context)
    print("[Hook] Starting file pipeline: " .. context.workflow_id)
    State.set("pipeline_start_time", os.time())
end)

advanced_pipeline:onAfterExecute(function(context)
    local duration = os.time() - State.get("pipeline_start_time")
    print("[Hook] Pipeline completed in " .. duration .. " seconds")
    
    -- Emit event with results
    advanced_pipeline:emit("file_pipeline_complete", {
        duration = duration,
        files_processed = State.get("files_processed") or 0
    })
end)

advanced_pipeline:onError(function(error)
    print("[Hook] Error in file pipeline: " .. tostring(error))
    -- Could trigger cleanup or recovery workflow here
end)

print("\n=== File System Tools Examples Complete ===")