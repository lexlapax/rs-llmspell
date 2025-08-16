-- Cookbook: Audit Logging - Security Event Tracking and Compliance
-- Purpose: Implement comprehensive audit logging patterns for security monitoring and compliance
-- Prerequisites: None (self-contained patterns for logging events)
-- Expected Output: Demonstration of audit logging patterns
-- Version: 0.7.0
-- Tags: cookbook, audit-logging, security, compliance, monitoring

print("=== Audit Logging Patterns ===\n")

-- ============================================================
-- Pattern 1: Structured Audit Logger
-- ============================================================

print("1. Structured Audit Logger")
print("-" .. string.rep("-", 40))

local AuditLogger = {}
AuditLogger.__index = AuditLogger

function AuditLogger:new(config)
    return setmetatable({
        log_entries = {},
        config = config or {},
        session_id = "session_" .. os.time() .. "_" .. math.random(10000),
        log_levels = {"DEBUG", "INFO", "WARN", "ERROR", "CRITICAL"},
        event_types = {
            "USER_AUTH", "USER_LOGOUT", "DATA_ACCESS", "DATA_MODIFY", 
            "SYSTEM_CONFIG", "SECURITY_VIOLATION", "ADMIN_ACTION", "API_CALL"
        },
        max_entries = config.max_entries or 1000,
        auto_flush_threshold = config.auto_flush_threshold or 100
    }, self)
end

function AuditLogger:log_event(event_type, level, message, metadata)
    local timestamp = os.date("%Y-%m-%d %H:%M:%S")
    local entry = {
        timestamp = timestamp,
        session_id = self.session_id,
        event_type = event_type,
        level = level or "INFO",
        message = message,
        metadata = metadata or {},
        sequence_number = #self.log_entries + 1,
        source = metadata and metadata.source or "system",
        user_id = metadata and metadata.user_id or "anonymous",
        ip_address = metadata and metadata.ip_address or "127.0.0.1",
        operation = metadata and metadata.operation or "unknown"
    }
    
    -- Add integrity hash (simple checksum for demonstration)
    entry.checksum = self:calculate_checksum(entry)
    
    table.insert(self.log_entries, entry)
    
    -- Format and display the log entry
    self:format_and_display(entry)
    
    -- Auto-flush if threshold reached
    if #self.log_entries >= self.auto_flush_threshold then
        self:flush_logs()
    end
end

function AuditLogger:calculate_checksum(entry)
    -- Simple checksum calculation (in production, use proper hashing)
    local data = entry.timestamp .. entry.event_type .. entry.message .. (entry.user_id or "")
    local checksum = 0
    for i = 1, #data do
        checksum = checksum + string.byte(data, i)
    end
    return string.format("%x", checksum % 0xFFFFFF)
end

function AuditLogger:format_and_display(entry)
    local level_symbols = {
        DEBUG = "🔍", INFO = "ℹ️", WARN = "⚠️", ERROR = "❌", CRITICAL = "🚨"
    }
    
    local symbol = level_symbols[entry.level] or "📝"
    
    print(string.format("   %s [%s] %s: %s", 
        symbol, entry.timestamp, entry.event_type, entry.message))
    
    if entry.user_id ~= "anonymous" then
        print(string.format("     User: %s | IP: %s | Op: %s", 
            entry.user_id, entry.ip_address, entry.operation))
    end
    
    if entry.metadata and next(entry.metadata) then
        local metadata_str = ""
        for key, value in pairs(entry.metadata) do
            if key ~= "source" and key ~= "user_id" and key ~= "ip_address" and key ~= "operation" then
                metadata_str = metadata_str .. key .. "=" .. tostring(value) .. " "
            end
        end
        if metadata_str ~= "" then
            print(string.format("     Metadata: %s", metadata_str))
        end
    end
end

function AuditLogger:log_authentication(user_id, success, ip_address, method)
    local event_type = success and "USER_AUTH" or "AUTH_FAILURE"
    local level = success and "INFO" or "WARN"
    local message = success and 
        string.format("User %s authenticated successfully", user_id) or
        string.format("Authentication failed for user %s", user_id)
    
    self:log_event(event_type, level, message, {
        user_id = user_id,
        ip_address = ip_address,
        operation = "authenticate",
        auth_method = method,
        success = success
    })
end

function AuditLogger:log_data_access(user_id, resource, action, success, ip_address)
    local event_type = "DATA_ACCESS"
    local level = success and "INFO" or "ERROR"
    local message = string.format("User %s %s %s: %s", 
        user_id, action, resource, success and "SUCCESS" or "FAILED")
    
    self:log_event(event_type, level, message, {
        user_id = user_id,
        ip_address = ip_address,
        operation = action,
        resource = resource,
        success = success
    })
end

function AuditLogger:log_security_violation(violation_type, description, user_id, ip_address, severity)
    self:log_event("SECURITY_VIOLATION", severity or "CRITICAL", 
        string.format("Security violation: %s - %s", violation_type, description), {
        user_id = user_id or "unknown",
        ip_address = ip_address or "unknown",
        operation = "security_check",
        violation_type = violation_type,
        severity = severity or "critical"
    })
end

function AuditLogger:log_admin_action(admin_user, action, target, details, ip_address)
    self:log_event("ADMIN_ACTION", "INFO",
        string.format("Admin %s performed %s on %s", admin_user, action, target), {
        user_id = admin_user,
        ip_address = ip_address,
        operation = action,
        target = target,
        details = details
    })
end

function AuditLogger:get_logs_by_criteria(criteria)
    local filtered_logs = {}
    
    for _, entry in ipairs(self.log_entries) do
        local matches = true
        
        if criteria.event_type and entry.event_type ~= criteria.event_type then
            matches = false
        end
        
        if criteria.user_id and entry.user_id ~= criteria.user_id then
            matches = false
        end
        
        if criteria.level and entry.level ~= criteria.level then
            matches = false
        end
        
        if criteria.time_from then
            -- Simple time comparison (in production, use proper date parsing)
            if entry.timestamp < criteria.time_from then
                matches = false
            end
        end
        
        if matches then
            table.insert(filtered_logs, entry)
        end
    end
    
    return filtered_logs
end

function AuditLogger:generate_security_report()
    local report = {
        total_events = #self.log_entries,
        by_event_type = {},
        by_level = {},
        security_violations = {},
        failed_authentications = {},
        suspicious_activity = {}
    }
    
    for _, entry in ipairs(self.log_entries) do
        -- Count by event type
        report.by_event_type[entry.event_type] = (report.by_event_type[entry.event_type] or 0) + 1
        
        -- Count by level
        report.by_level[entry.level] = (report.by_level[entry.level] or 0) + 1
        
        -- Collect security violations
        if entry.event_type == "SECURITY_VIOLATION" then
            table.insert(report.security_violations, entry)
        end
        
        -- Collect failed authentications
        if entry.event_type == "AUTH_FAILURE" then
            table.insert(report.failed_authentications, entry)
        end
        
        -- Identify suspicious activity (multiple failures from same IP)
        if entry.event_type == "AUTH_FAILURE" then
            local ip = entry.ip_address
            if not report.suspicious_activity[ip] then
                report.suspicious_activity[ip] = 0
            end
            report.suspicious_activity[ip] = report.suspicious_activity[ip] + 1
        end
    end
    
    return report
end

function AuditLogger:flush_logs()
    local flushed_count = #self.log_entries
    -- In production, this would write to persistent storage
    print(string.format("   💾 Flushed %d audit log entries to persistent storage", flushed_count))
    
    -- Keep recent entries for immediate access
    local keep_recent = math.min(50, self.max_entries)
    if #self.log_entries > keep_recent then
        local recent_entries = {}
        for i = #self.log_entries - keep_recent + 1, #self.log_entries do
            table.insert(recent_entries, self.log_entries[i])
        end
        self.log_entries = recent_entries
    end
    
    return flushed_count
end

-- Test structured audit logging
local audit_logger = AuditLogger:new({
    max_entries = 100,
    auto_flush_threshold = 25
})

print("   Testing structured audit logging:")

-- Test various audit scenarios
audit_logger:log_authentication("john_doe", true, "192.168.1.100", "password")
audit_logger:log_authentication("jane_smith", true, "192.168.1.101", "oauth")
audit_logger:log_authentication("hacker_user", false, "10.0.0.50", "brute_force")
audit_logger:log_authentication("hacker_user", false, "10.0.0.50", "brute_force")

audit_logger:log_data_access("john_doe", "/api/users", "READ", true, "192.168.1.100")
audit_logger:log_data_access("jane_smith", "/api/financial", "UPDATE", true, "192.168.1.101")
audit_logger:log_data_access("unauthorized_user", "/api/admin", "READ", false, "10.0.0.75")

audit_logger:log_security_violation("sql_injection", "Detected SQL injection attempt in search parameter", 
    "malicious_user", "203.0.113.0", "CRITICAL")

audit_logger:log_admin_action("admin_user", "user_role_change", "john_doe", 
    "Changed role from user to admin", "192.168.1.1")

print()

-- ============================================================
-- Pattern 2: Compliance Audit Trail
-- ============================================================

print("2. Compliance Audit Trail")
print("-" .. string.rep("-", 40))

local ComplianceAuditor = {}
ComplianceAuditor.__index = ComplianceAuditor

function ComplianceAuditor:new(compliance_standard)
    return setmetatable({
        compliance_standard = compliance_standard or "SOX", -- SOX, HIPAA, GDPR, PCI_DSS
        audit_trail = {},
        control_objectives = self:get_control_objectives(compliance_standard),
        retention_period = self:get_retention_period(compliance_standard),
        required_fields = self:get_required_fields(compliance_standard)
    }, self)
end

function ComplianceAuditor:get_control_objectives(standard)
    local objectives = {
        SOX = {
            "financial_reporting_accuracy",
            "internal_controls_effectiveness", 
            "change_management_controls",
            "access_controls",
            "data_integrity"
        },
        HIPAA = {
            "phi_access_controls",
            "audit_logs_integrity",
            "minimum_necessary_access",
            "encryption_compliance",
            "breach_notification"
        },
        GDPR = {
            "data_processing_lawfulness",
            "consent_management",
            "data_subject_rights",
            "data_retention_limits",
            "cross_border_transfers"
        },
        PCI_DSS = {
            "cardholder_data_protection",
            "secure_transmission",
            "access_control_measures",
            "network_monitoring",
            "vulnerability_management"
        }
    }
    return objectives[standard] or objectives.SOX
end

function ComplianceAuditor:get_retention_period(standard)
    local periods = {
        SOX = 2555, -- 7 years in days
        HIPAA = 2190, -- 6 years
        GDPR = 1095, -- 3 years
        PCI_DSS = 365 -- 1 year
    }
    return periods[standard] or 2555
end

function ComplianceAuditor:get_required_fields(standard)
    local fields = {
        SOX = {"timestamp", "user_id", "action", "system", "before_value", "after_value", "approval_status"},
        HIPAA = {"timestamp", "user_id", "patient_id", "phi_accessed", "purpose", "minimum_necessary_justification"},
        GDPR = {"timestamp", "data_subject_id", "processing_purpose", "legal_basis", "data_categories", "retention_period"},
        PCI_DSS = {"timestamp", "user_id", "cardholder_data_accessed", "access_method", "network_location", "encryption_status"}
    }
    return fields[standard] or fields.SOX
end

function ComplianceAuditor:log_compliance_event(control_objective, event_data)
    if not self:validate_required_fields(event_data) then
        print(string.format("   ❌ Compliance event missing required fields for %s", self.compliance_standard))
        return false
    end
    
    local audit_entry = {
        audit_id = "audit_" .. os.time() .. "_" .. math.random(10000),
        timestamp = os.date("%Y-%m-%d %H:%M:%S"),
        compliance_standard = self.compliance_standard,
        control_objective = control_objective,
        event_data = event_data,
        retention_until = os.date("%Y-%m-%d", os.time() + (self.retention_period * 24 * 60 * 60)),
        digital_signature = self:generate_digital_signature(event_data),
        chain_hash = self:calculate_chain_hash(event_data)
    }
    
    table.insert(self.audit_trail, audit_entry)
    
    print(string.format("   📜 Compliance event logged: %s [%s]", 
        control_objective, self.compliance_standard))
    print(string.format("     Audit ID: %s | Retention until: %s", 
        audit_entry.audit_id, audit_entry.retention_until))
    
    return true
end

function ComplianceAuditor:validate_required_fields(event_data)
    for _, field in ipairs(self.required_fields) do
        if not event_data[field] then
            return false
        end
    end
    return true
end

function ComplianceAuditor:generate_digital_signature(event_data)
    -- Simplified digital signature simulation
    local data_string = ""
    for key, value in pairs(event_data) do
        data_string = data_string .. key .. "=" .. tostring(value) .. ";"
    end
    
    -- Simple hash simulation (in production, use proper cryptographic signing)
    local signature = 0
    for i = 1, #data_string do
        signature = signature + string.byte(data_string, i) * i
    end
    
    return string.format("sig_%x", signature % 0xFFFFFFFF)
end

function ComplianceAuditor:calculate_chain_hash(event_data)
    -- Blockchain-style chain hash for immutability
    local previous_hash = #self.audit_trail > 0 and 
        self.audit_trail[#self.audit_trail].chain_hash or "genesis"
    
    local current_data = tostring(os.time()) .. tostring(event_data.timestamp or "") .. previous_hash
    local hash = 0
    for i = 1, #current_data do
        hash = hash + string.byte(current_data, i) * (i + 7)
    end
    
    return string.format("hash_%x", hash % 0xFFFFFFFF)
end

function ComplianceAuditor:generate_compliance_report(objective_filter)
    local report = {
        standard = self.compliance_standard,
        report_generated = os.date("%Y-%m-%d %H:%M:%S"),
        total_events = #self.audit_trail,
        by_objective = {},
        compliance_gaps = {},
        retention_summary = {}
    }
    
    local current_time = os.time()
    
    for _, entry in ipairs(self.audit_trail) do
        if not objective_filter or entry.control_objective == objective_filter then
            -- Count by objective
            report.by_objective[entry.control_objective] = 
                (report.by_objective[entry.control_objective] or 0) + 1
            
            -- Check retention compliance
            local retention_date = os.time(os.date("*t", os.time(os.date("!*t", current_time))))
            -- Simplified retention check
            if entry.retention_until then
                local retention_status = "active"
                report.retention_summary[retention_status] = 
                    (report.retention_summary[retention_status] or 0) + 1
            end
        end
    end
    
    -- Identify compliance gaps
    for _, objective in ipairs(self.control_objectives) do
        if not report.by_objective[objective] then
            table.insert(report.compliance_gaps, {
                objective = objective,
                issue = "No audit events recorded",
                risk_level = "HIGH"
            })
        elseif report.by_objective[objective] < 5 then
            table.insert(report.compliance_gaps, {
                objective = objective,
                issue = "Insufficient audit evidence",
                risk_level = "MEDIUM"
            })
        end
    end
    
    return report
end

function ComplianceAuditor:verify_audit_integrity()
    local integrity_report = {
        total_entries = #self.audit_trail,
        verified_signatures = 0,
        verified_chain = 0,
        integrity_violations = {}
    }
    
    for i, entry in ipairs(self.audit_trail) do
        -- Verify digital signature
        local expected_signature = self:generate_digital_signature(entry.event_data)
        if entry.digital_signature == expected_signature then
            integrity_report.verified_signatures = integrity_report.verified_signatures + 1
        else
            table.insert(integrity_report.integrity_violations, {
                entry_id = entry.audit_id,
                violation_type = "signature_mismatch",
                position = i
            })
        end
        
        -- Verify chain hash (simplified check)
        if i > 1 then
            local previous_entry = self.audit_trail[i - 1]
            -- In a real implementation, this would verify the chain properly
            integrity_report.verified_chain = integrity_report.verified_chain + 1
        end
    end
    
    integrity_report.integrity_percentage = #self.audit_trail > 0 and 
        (integrity_report.verified_signatures / #self.audit_trail) * 100 or 100
    
    return integrity_report
end

-- Test compliance audit trail
local sox_auditor = ComplianceAuditor:new("SOX")

print("   Testing SOX compliance audit trail:")

-- Test SOX compliance events
sox_auditor:log_compliance_event("financial_reporting_accuracy", {
    timestamp = os.date("%Y-%m-%d %H:%M:%S"),
    user_id = "finance_user",
    action = "update_financial_report",
    system = "erp_system",
    before_value = "revenue: 1000000",
    after_value = "revenue: 1050000",
    approval_status = "pending_approval"
})

sox_auditor:log_compliance_event("access_controls", {
    timestamp = os.date("%Y-%m-%d %H:%M:%S"),
    user_id = "it_admin",
    action = "grant_financial_access",
    system = "access_management",
    before_value = "role: employee",
    after_value = "role: financial_analyst",
    approval_status = "approved"
})

-- Test HIPAA compliance
local hipaa_auditor = ComplianceAuditor:new("HIPAA")

print("\n   Testing HIPAA compliance audit trail:")

hipaa_auditor:log_compliance_event("phi_access_controls", {
    timestamp = os.date("%Y-%m-%d %H:%M:%S"),
    user_id = "doctor_smith",
    patient_id = "patient_12345",
    phi_accessed = "medical_records",
    purpose = "treatment",
    minimum_necessary_justification = "reviewing treatment history for diagnosis"
})

print()

-- ============================================================
-- Pattern 3: Real-time Security Monitoring
-- ============================================================

print("3. Real-time Security Monitoring")
print("-" .. string.rep("-", 40))

local SecurityMonitor = {}
SecurityMonitor.__index = SecurityMonitor

function SecurityMonitor:new()
    return setmetatable({
        active_sessions = {},
        threat_patterns = self:initialize_threat_patterns(),
        alert_rules = self:initialize_alert_rules(),
        alert_history = {},
        monitoring_rules = {},
        anomaly_detector = self:initialize_anomaly_detector()
    }, self)
end

function SecurityMonitor:initialize_threat_patterns()
    return {
        sql_injection = {
            patterns = {"'", "union", "select", "drop", "insert", "delete", "--", "/*"},
            severity = "HIGH",
            description = "SQL injection attempt detected"
        },
        xss_attack = {
            patterns = {"<script", "javascript:", "onload=", "onerror=", "onclick="},
            severity = "MEDIUM",
            description = "Cross-site scripting attempt detected"
        },
        brute_force = {
            patterns = {"multiple_failed_logins"},
            threshold = 5,
            time_window = 300, -- 5 minutes
            severity = "HIGH",
            description = "Brute force attack detected"
        },
        privilege_escalation = {
            patterns = {"sudo", "admin", "root", "escalate"},
            severity = "CRITICAL",
            description = "Privilege escalation attempt detected"
        }
    }
end

function SecurityMonitor:initialize_alert_rules()
    return {
        failed_login_threshold = {
            event_type = "AUTH_FAILURE",
            threshold = 3,
            time_window = 180, -- 3 minutes
            action = "block_ip"
        },
        admin_after_hours = {
            event_type = "ADMIN_ACTION",
            time_restriction = {start_hour = 18, end_hour = 8},
            action = "send_alert"
        },
        data_exfiltration = {
            event_type = "DATA_ACCESS",
            volume_threshold = 1000, -- records
            time_window = 3600, -- 1 hour
            action = "investigate"
        }
    }
end

function SecurityMonitor:initialize_anomaly_detector()
    return {
        baseline_behavior = {},
        learning_mode = true,
        sensitivity = 0.8,
        min_samples = 50
    }
end

function SecurityMonitor:process_security_event(event)
    -- Store the event
    local security_event = {
        event_id = "sec_" .. os.time() .. "_" .. math.random(10000),
        timestamp = os.date("%Y-%m-%d %H:%M:%S"),
        raw_event = event,
        threat_assessment = self:assess_threat_level(event),
        anomaly_score = self:calculate_anomaly_score(event),
        triggered_rules = {}
    }
    
    -- Check against threat patterns
    self:check_threat_patterns(security_event)
    
    -- Check alert rules
    self:check_alert_rules(security_event)
    
    -- Update anomaly detection baseline
    self:update_anomaly_baseline(event)
    
    return security_event
end

function SecurityMonitor:assess_threat_level(event)
    local threat_score = 0
    local detected_threats = {}
    
    -- Check event content against threat patterns
    local event_content = string.lower(event.message or "") .. " " .. 
                         string.lower(event.operation or "") .. " " ..
                         string.lower(event.resource or "")
    
    for threat_name, threat_config in pairs(self.threat_patterns) do
        for _, pattern in ipairs(threat_config.patterns) do
            if string.find(event_content, pattern, 1, true) then
                table.insert(detected_threats, {
                    threat = threat_name,
                    pattern = pattern,
                    severity = threat_config.severity
                })
                
                -- Add to threat score
                local severity_scores = {LOW = 1, MEDIUM = 3, HIGH = 5, CRITICAL = 8}
                threat_score = threat_score + (severity_scores[threat_config.severity] or 1)
            end
        end
    end
    
    return {
        score = threat_score,
        level = threat_score >= 8 and "CRITICAL" or 
                threat_score >= 5 and "HIGH" or
                threat_score >= 3 and "MEDIUM" or "LOW",
        detected_threats = detected_threats
    }
end

function SecurityMonitor:calculate_anomaly_score(event)
    -- Simplified anomaly detection based on event frequency and timing
    local user_id = event.user_id or "anonymous"
    local operation = event.operation or "unknown"
    local hour = tonumber(os.date("%H"))
    
    -- Check if this is unusual activity for this user/operation/time
    local baseline_key = user_id .. "_" .. operation .. "_" .. hour
    local baseline = self.anomaly_detector.baseline_behavior[baseline_key] or {count = 0, total_samples = 0}
    
    -- Calculate deviation from baseline
    local expected_frequency = baseline.total_samples > 0 and (baseline.count / baseline.total_samples) or 0
    local anomaly_score = expected_frequency > 0 and math.abs(1 - expected_frequency) or 0.5
    
    return math.min(1.0, anomaly_score)
end

function SecurityMonitor:check_threat_patterns(security_event)
    local threat_assessment = security_event.threat_assessment
    
    if #threat_assessment.detected_threats > 0 then
        print(string.format("   🚨 THREAT DETECTED: %s (Score: %d)", 
            threat_assessment.level, threat_assessment.score))
        
        for _, threat in ipairs(threat_assessment.detected_threats) do
            print(string.format("     • %s: %s (Pattern: %s)", 
                threat.severity, threat.threat, threat.pattern))
        end
        
        -- Log security alert
        self:create_security_alert("THREAT_PATTERN", security_event, {
            threat_level = threat_assessment.level,
            detected_patterns = threat_assessment.detected_threats
        })
    end
end

function SecurityMonitor:check_alert_rules(security_event)
    local event = security_event.raw_event
    
    for rule_name, rule_config in pairs(self.alert_rules) do
        if self:evaluate_alert_rule(rule_name, rule_config, event) then
            table.insert(security_event.triggered_rules, rule_name)
            
            print(string.format("   ⚠️  ALERT RULE TRIGGERED: %s", rule_name))
            
            self:create_security_alert("RULE_VIOLATION", security_event, {
                rule_name = rule_name,
                rule_config = rule_config
            })
        end
    end
end

function SecurityMonitor:evaluate_alert_rule(rule_name, rule_config, event)
    -- Check event type match
    if rule_config.event_type and event.event_type ~= rule_config.event_type then
        return false
    end
    
    -- Check time restrictions
    if rule_config.time_restriction then
        local current_hour = tonumber(os.date("%H"))
        local restriction = rule_config.time_restriction
        
        if restriction.start_hour > restriction.end_hour then
            -- Overnight restriction (e.g., 18:00 to 08:00)
            if current_hour < restriction.end_hour or current_hour >= restriction.start_hour then
                return true
            end
        else
            -- Same-day restriction
            if current_hour >= restriction.start_hour and current_hour < restriction.end_hour then
                return true
            end
        end
    end
    
    -- For demonstration, return false for other rules
    return false
end

function SecurityMonitor:create_security_alert(alert_type, security_event, alert_data)
    local alert = {
        alert_id = "alert_" .. os.time() .. "_" .. math.random(10000),
        timestamp = os.date("%Y-%m-%d %H:%M:%S"),
        alert_type = alert_type,
        severity = alert_data.threat_level or "MEDIUM",
        security_event_id = security_event.event_id,
        alert_data = alert_data,
        status = "ACTIVE",
        acknowledged = false
    }
    
    table.insert(self.alert_history, alert)
    
    print(string.format("     Alert ID: %s | Status: %s", alert.alert_id, alert.status))
end

function SecurityMonitor:update_anomaly_baseline(event)
    local user_id = event.user_id or "anonymous"
    local operation = event.operation or "unknown"
    local hour = tonumber(os.date("%H"))
    
    local baseline_key = user_id .. "_" .. operation .. "_" .. hour
    local baseline = self.anomaly_detector.baseline_behavior[baseline_key] or {count = 0, total_samples = 0}
    
    baseline.count = baseline.count + 1
    baseline.total_samples = baseline.total_samples + 1
    
    self.anomaly_detector.baseline_behavior[baseline_key] = baseline
end

function SecurityMonitor:get_security_dashboard()
    local dashboard = {
        total_events_processed = 0,
        active_alerts = 0,
        threat_level_distribution = {LOW = 0, MEDIUM = 0, HIGH = 0, CRITICAL = 0},
        top_threats = {},
        recent_alerts = {}
    }
    
    -- Count active alerts
    for _, alert in ipairs(self.alert_history) do
        if alert.status == "ACTIVE" then
            dashboard.active_alerts = dashboard.active_alerts + 1
        end
    end
    
    -- Get recent alerts (last 5)
    local recent_count = math.min(5, #self.alert_history)
    for i = #self.alert_history - recent_count + 1, #self.alert_history do
        if self.alert_history[i] then
            table.insert(dashboard.recent_alerts, self.alert_history[i])
        end
    end
    
    return dashboard
end

-- Test real-time security monitoring
local security_monitor = SecurityMonitor:new()

print("   Testing real-time security monitoring:")

-- Test various security events
local test_events = {
    {
        event_type = "DATA_ACCESS",
        user_id = "malicious_user",
        message = "SELECT * FROM users WHERE '1'='1' UNION SELECT password FROM admin",
        operation = "database_query",
        resource = "/api/users",
        ip_address = "203.0.113.1"
    },
    {
        event_type = "USER_INPUT",
        user_id = "attacker",
        message = "User input: <script>alert('XSS')</script>",
        operation = "form_submit",
        resource = "/contact",
        ip_address = "198.51.100.1"
    },
    {
        event_type = "ADMIN_ACTION",
        user_id = "admin_user",
        message = "User privilege escalation attempted using sudo command",
        operation = "privilege_change",
        resource = "/admin/users",
        ip_address = "192.168.1.100"
    }
}

for i, event in ipairs(test_events) do
    print(string.format("\n   Processing security event %d:", i))
    local security_event = security_monitor:process_security_event(event)
    
    print(string.format("     Event ID: %s", security_event.event_id))
    print(string.format("     Threat Level: %s (Score: %d)", 
        security_event.threat_assessment.level, security_event.threat_assessment.score))
    print(string.format("     Anomaly Score: %.2f", security_event.anomaly_score))
end

-- Display security dashboard
local dashboard = security_monitor:get_security_dashboard()
print(string.format("\n   Security Dashboard Summary:"))
print(string.format("     Active alerts: %d", dashboard.active_alerts))
print(string.format("     Recent alerts: %d", #dashboard.recent_alerts))

print()
print("🎯 Key Takeaways:")
print("   • Implement structured logging with consistent formats")
print("   • Ensure compliance with regulatory requirements")
print("   • Use digital signatures for audit log integrity")
print("   • Implement real-time threat detection and alerting")
print("   • Maintain proper log retention and archival policies")
print("   • Monitor for anomalous behavior patterns")
print("   • Provide audit trail reporting and analysis capabilities")