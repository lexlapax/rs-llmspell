-- ABOUTME: Backup validation and integrity checking automation
-- ABOUTME: Provides comprehensive backup health monitoring and validation procedures

-- Backup Validation and Monitoring Script
-- This script provides automated validation of backup integrity and operational readiness

local function log(level, message)
    local timestamp = os.date("%Y-%m-%d %H:%M:%S")
    print(string.format("[%s] %s: %s", timestamp, level, message))
end

local function log_info(message)
    log("INFO", message)
end

local function log_error(message)
    log("ERROR", message)
end

local function log_success(message)
    log("SUCCESS", message)
end

local function log_warning(message)
    log("WARNING", message)
end

-- Backup Validation Configuration
local validation_config = {
    max_backup_age_hours = 24,
    min_backup_count = 3,
    max_validation_time_seconds = 300,
    required_scopes = {"global", "agents", "sessions"},
    integrity_check_sample_size = 10
}

-- Backup Information Structure
local function create_backup_info(id, created_at, size_bytes, entry_count, is_incremental)
    return {
        id = id,
        created_at = created_at,
        size_bytes = size_bytes,
        entry_count = entry_count,
        is_incremental = is_incremental,
        validation_status = "pending",
        validation_errors = {},
        validation_warnings = {}
    }
end

-- Simulate backup listing (in real implementation, this would call backup system)
local function get_available_backups()
    log_info("Retrieving available backups...")
    
    local current_time = os.time()
    local backups = {}
    
    -- Simulate several backups of different ages
    for i = 1, 5 do
        local backup_age_hours = i * 4
        local backup_time = current_time - (backup_age_hours * 3600)
        local backup_id = string.format("backup_%d_%s", backup_time, string.format("%x", math.random(1000000, 9999999)))
        
        local backup = create_backup_info(
            backup_id,
            backup_time,
            math.random(1000000, 10000000), -- Random size
            math.random(50, 500),          -- Random entry count
            i > 1                          -- First backup is full, rest are incremental
        )
        
        table.insert(backups, backup)
    end
    
    log_success(string.format("Found %d available backups", #backups))
    return backups
end

-- Validate backup age and freshness
local function validate_backup_freshness(backups)
    log_info("=== VALIDATING BACKUP FRESHNESS ===")
    
    local current_time = os.time()
    local fresh_backups = 0
    local stale_backups = 0
    
    for _, backup in ipairs(backups) do
        local backup_age_hours = (current_time - backup.created_at) / 3600
        
        if backup_age_hours <= validation_config.max_backup_age_hours then
            fresh_backups = fresh_backups + 1
            log_success(string.format("✓ Backup %s is fresh (%.1f hours old)", 
                                    string.sub(backup.id, 1, 12) .. "...", backup_age_hours))
        else
            stale_backups = stale_backups + 1
            table.insert(backup.validation_warnings, 
                        string.format("Backup is stale (%.1f hours old)", backup_age_hours))
            log_warning(string.format("⚠ Backup %s is stale (%.1f hours old)", 
                                    string.sub(backup.id, 1, 12) .. "...", backup_age_hours))
        end
    end
    
    -- Check minimum backup count
    if #backups < validation_config.min_backup_count then
        log_error(string.format("✗ Insufficient backups: %d (minimum required: %d)", 
                               #backups, validation_config.min_backup_count))
        return false
    end
    
    -- Check fresh backup availability
    if fresh_backups == 0 then
        log_error("✗ No fresh backups available within the required timeframe")
        return false
    end
    
    log_success(string.format("✓ Backup freshness validation passed (%d fresh, %d stale)", 
                             fresh_backups, stale_backups))
    return true
end

-- Validate backup size and growth patterns
local function validate_backup_sizes(backups)
    log_info("=== VALIDATING BACKUP SIZES ===")
    
    -- Sort backups by creation time
    table.sort(backups, function(a, b) return a.created_at < b.created_at end)
    
    local size_anomalies = 0
    local total_size = 0
    
    for i, backup in ipairs(backups) do
        total_size = total_size + backup.size_bytes
        local size_mb = backup.size_bytes / (1024 * 1024)
        
        -- Check for size anomalies
        if i > 1 then
            local prev_backup = backups[i-1]
            local prev_size_mb = prev_backup.size_bytes / (1024 * 1024)
            local size_change_percent = ((size_mb - prev_size_mb) / prev_size_mb) * 100
            
            -- Flag significant size changes (> 50% growth or > 30% shrinkage)
            if size_change_percent > 50 then
                size_anomalies = size_anomalies + 1
                table.insert(backup.validation_warnings, 
                           string.format("Large size increase: %.1f%% vs previous backup", size_change_percent))
                log_warning(string.format("⚠ Backup %s: Large size increase (%.1fMB, +%.1f%%)", 
                                        string.sub(backup.id, 1, 12) .. "...", size_mb, size_change_percent))
            elseif size_change_percent < -30 then
                size_anomalies = size_anomalies + 1
                table.insert(backup.validation_warnings, 
                           string.format("Large size decrease: %.1f%% vs previous backup", size_change_percent))
                log_warning(string.format("⚠ Backup %s: Large size decrease (%.1fMB, %.1f%%)", 
                                        string.sub(backup.id, 1, 12) .. "...", size_mb, size_change_percent))
            end
        end
        
        log_info(string.format("Backup %s: %.1fMB, %d entries", 
                              string.sub(backup.id, 1, 12) .. "...", size_mb, backup.entry_count))
    end
    
    local total_size_mb = total_size / (1024 * 1024)
    log_info(string.format("Total backup storage: %.1fMB", total_size_mb))
    
    if size_anomalies > 0 then
        log_warning(string.format("⚠ %d size anomalies detected (investigate but not critical)", size_anomalies))
    else
        log_success("✓ Backup sizes are consistent")
    end
    
    return true
end

-- Validate incremental backup chain integrity
local function validate_incremental_chains(backups)
    log_info("=== VALIDATING INCREMENTAL BACKUP CHAINS ===")
    
    local full_backups = {}
    local incremental_backups = {}
    
    -- Separate full and incremental backups
    for _, backup in ipairs(backups) do
        if backup.is_incremental then
            table.insert(incremental_backups, backup)
        else
            table.insert(full_backups, backup)
        end
    end
    
    log_info(string.format("Found %d full backups and %d incremental backups", 
                          #full_backups, #incremental_backups))
    
    -- Validate we have at least one full backup
    if #full_backups == 0 then
        log_error("✗ No full backups found - incremental chain is broken")
        return false
    end
    
    -- Check incremental backup relationships
    local orphaned_incrementals = 0
    for _, inc_backup in ipairs(incremental_backups) do
        local has_parent = false
        
        -- In a real implementation, this would check actual parent relationships
        -- For simulation, we assume incrementals have valid parents if created after a full backup
        for _, full_backup in ipairs(full_backups) do
            if inc_backup.created_at > full_backup.created_at then
                has_parent = true
                break
            end
        end
        
        if not has_parent then
            orphaned_incrementals = orphaned_incrementals + 1
            table.insert(inc_backup.validation_errors, "No valid parent backup found")
            log_error(string.format("✗ Incremental backup %s has no valid parent", 
                                   string.sub(inc_backup.id, 1, 12) .. "..."))
        end
    end
    
    if orphaned_incrementals > 0 then
        log_error(string.format("✗ %d orphaned incremental backups found", orphaned_incrementals))
        return false
    end
    
    log_success("✓ Incremental backup chain integrity validated")
    return true
end

-- Simulate integrity checking for a backup
local function check_backup_integrity(backup)
    log_info(string.format("Checking integrity of backup: %s", string.sub(backup.id, 1, 12) .. "..."))
    
    local checks = {
        "Checksum verification",
        "Data structure validation",
        "Schema compatibility check",
        "Compression integrity",
        "Metadata consistency"
    }
    
    local passed_checks = 0
    local failed_checks = 0
    
    for _, check in ipairs(checks) do
        -- Simulate check execution time
        os.execute("sleep 0.1")
        
        -- Simulate occasional check failures (5% chance)
        local success = math.random() > 0.05
        
        if success then
            passed_checks = passed_checks + 1
            log_success(string.format("  ✓ %s", check))
        else
            failed_checks = failed_checks + 1
            table.insert(backup.validation_errors, string.format("%s failed", check))
            log_error(string.format("  ✗ %s", check))
        end
    end
    
    local integrity_valid = failed_checks == 0
    backup.validation_status = integrity_valid and "valid" or "invalid"
    
    return integrity_valid, passed_checks, failed_checks
end

-- Perform data sampling validation
local function validate_backup_data_samples(backup)
    log_info(string.format("Validating data samples from backup: %s", string.sub(backup.id, 1, 12) .. "..."))
    
    local sample_validations = {}
    
    -- Simulate sampling different data types
    local sample_types = {
        "Global configuration data",
        "Agent state data", 
        "User session data",
        "System metadata",
        "Feature flags"
    }
    
    for _, sample_type in ipairs(sample_types) do
        -- Simulate sample validation
        os.execute("sleep 0.05")
        
        local valid = math.random() > 0.02 -- 98% success rate
        table.insert(sample_validations, {type = sample_type, valid = valid})
        
        if valid then
            log_success(string.format("  ✓ %s sample valid", sample_type))
        else
            log_error(string.format("  ✗ %s sample invalid", sample_type))
            table.insert(backup.validation_errors, string.format("%s sample validation failed", sample_type))
        end
    end
    
    local valid_samples = 0
    for _, validation in ipairs(sample_validations) do
        if validation.valid then
            valid_samples = valid_samples + 1
        end
    end
    
    local sample_success_rate = (valid_samples / #sample_validations) * 100
    log_info(string.format("Sample validation success rate: %.1f%%", sample_success_rate))
    
    return sample_success_rate >= 95 -- Require 95% sample validation success
end

-- Comprehensive backup validation
local function validate_individual_backup(backup)
    log_info(string.format("=== VALIDATING BACKUP: %s ===", string.sub(backup.id, 1, 12) .. "..."))
    
    local validation_start_time = os.time()
    
    -- Perform integrity checks
    local integrity_valid, passed_checks, failed_checks = check_backup_integrity(backup)
    
    -- Perform data sample validation
    local samples_valid = validate_backup_data_samples(backup)
    
    local validation_duration = os.time() - validation_start_time
    
    -- Overall validation result
    local overall_valid = integrity_valid and samples_valid
    backup.validation_status = overall_valid and "valid" or "invalid"
    
    if overall_valid then
        log_success(string.format("✓ Backup validation PASSED (%d seconds)", validation_duration))
    else
        log_error(string.format("✗ Backup validation FAILED (%d seconds)", validation_duration))
        log_error(string.format("  Integrity checks: %d passed, %d failed", passed_checks, failed_checks))
        log_error(string.format("  Sample validation: %s", samples_valid and "passed" or "failed"))
    end
    
    return overall_valid
end

-- Generate validation report
local function generate_validation_report(backups, validation_results)
    log_info("=== GENERATING VALIDATION REPORT ===")
    
    local total_backups = #backups
    local valid_backups = 0
    local invalid_backups = 0
    local total_errors = 0
    local total_warnings = 0
    
    for _, backup in ipairs(backups) do
        if backup.validation_status == "valid" then
            valid_backups = valid_backups + 1
        else
            invalid_backups = invalid_backups + 1
        end
        
        total_errors = total_errors + #backup.validation_errors
        total_warnings = total_warnings + #backup.validation_warnings
    end
    
    -- Calculate overall health score
    local health_score = (valid_backups / total_backups) * 100
    
    log_info("======================================")
    log_info("BACKUP VALIDATION REPORT")
    log_info("======================================")
    log_info(string.format("Total backups validated: %d", total_backups))
    log_info(string.format("Valid backups: %d", valid_backups))
    log_info(string.format("Invalid backups: %d", invalid_backups))
    log_info(string.format("Total errors: %d", total_errors))
    log_info(string.format("Total warnings: %d", total_warnings))
    log_info(string.format("Backup health score: %.1f%%", health_score))
    
    -- Determine overall status
    local overall_status = "HEALTHY"
    if health_score < 80 then
        overall_status = "CRITICAL"
    elseif health_score < 95 then
        overall_status = "WARNING"
    end
    
    if overall_status == "HEALTHY" then
        log_success(string.format("Overall backup status: %s", overall_status))
    elseif overall_status == "WARNING" then
        log_warning(string.format("Overall backup status: %s", overall_status))
    else
        log_error(string.format("Overall backup status: %s", overall_status))
    end
    
    -- Recommendations
    log_info("======================================")
    log_info("RECOMMENDATIONS")
    log_info("======================================")
    
    if invalid_backups > 0 then
        log_info("• Investigate and resolve backup integrity issues")
        log_info("• Consider creating new full backups to replace invalid ones")
    end
    
    if total_warnings > 0 then
        log_info("• Review backup size trends and storage capacity")
        log_info("• Monitor backup creation schedule and timing")
    end
    
    if health_score < 100 then
        log_info("• Run backup validation more frequently")
        log_info("• Consider automated backup health monitoring")
    end
    
    log_info("• Test disaster recovery procedures regularly")
    log_info("• Verify backup retention policies are appropriate")
    log_info("======================================")
    
    return {
        total_backups = total_backups,
        valid_backups = valid_backups,
        invalid_backups = invalid_backups,
        health_score = health_score,
        overall_status = overall_status,
        total_errors = total_errors,
        total_warnings = total_warnings
    }
end

-- Main backup validation procedure
local function main()
    log_info("========================================")
    log_info("BACKUP VALIDATION PROCEDURE INITIATED")
    log_info("========================================")
    
    local validation_start_time = os.time()
    
    -- Get available backups
    local backups = get_available_backups()
    
    if #backups == 0 then
        log_error("No backups found - backup system may not be configured")
        return
    end
    
    -- Perform validation checks
    local freshness_ok = validate_backup_freshness(backups)
    local sizes_ok = validate_backup_sizes(backups) 
    local chains_ok = validate_incremental_chains(backups)
    
    -- Validate individual backups
    local individual_validation_results = {}
    for _, backup in ipairs(backups) do
        local backup_valid = validate_individual_backup(backup)
        table.insert(individual_validation_results, backup_valid)
    end
    
    -- Generate comprehensive report
    local validation_results = {
        freshness_validation = freshness_ok,
        size_validation = sizes_ok,
        chain_validation = chains_ok,
        individual_validations = individual_validation_results
    }
    
    local report = generate_validation_report(backups, validation_results)
    
    local total_validation_time = os.time() - validation_start_time
    
    log_info("========================================")
    if report.overall_status == "HEALTHY" then
        log_success("BACKUP VALIDATION COMPLETED SUCCESSFULLY")
    elseif report.overall_status == "WARNING" then
        log_warning("BACKUP VALIDATION COMPLETED WITH WARNINGS")
    else
        log_error("BACKUP VALIDATION IDENTIFIED CRITICAL ISSUES")
    end
    log_info(string.format("Total validation time: %d seconds", total_validation_time))
    log_info("========================================")
    
    -- Store validation results for monitoring
    State.save("global", "last_backup_validation", {
        timestamp = os.time(),
        results = report,
        validation_duration = total_validation_time
    })
end

-- Execute the backup validation procedure
main()