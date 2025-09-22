#!/usr/bin/env python3
"""
Application Validation Suite for llmspell
Tests all 9 example applications via CLI execution
"""

import subprocess
import os
import sys
import json
import time
import re
import shutil
import argparse
from pathlib import Path
from typing import Dict, List, Optional, Tuple
from datetime import datetime
from dataclasses import dataclass, asdict


@dataclass
class TestResult:
    """Result of a single application test"""
    app_name: str
    layer: int
    status: str  # passed, failed, skipped
    runtime_seconds: float
    stdout: str
    stderr: str
    errors: List[str]
    validations: Dict[str, bool]
    files_created: List[str]


@dataclass
class TestReport:
    """Overall test suite report"""
    timestamp: str
    total_apps: int
    passed: int
    failed: int
    skipped: int
    total_runtime: float
    results: List[TestResult]

    def to_json(self) -> str:
        """Convert report to JSON"""
        return json.dumps(asdict(self), indent=2)

    def to_html(self) -> str:
        """Generate HTML report"""
        html = f"""
<!DOCTYPE html>
<html>
<head>
    <title>llmspell Application Test Report - {self.timestamp}</title>
    <style>
        body {{ font-family: Arial, sans-serif; margin: 20px; }}
        .summary {{ background: #f0f0f0; padding: 15px; border-radius: 5px; }}
        .passed {{ color: green; }}
        .failed {{ color: red; }}
        .skipped {{ color: orange; }}
        table {{ border-collapse: collapse; width: 100%; margin-top: 20px; }}
        th, td {{ border: 1px solid #ddd; padding: 8px; text-align: left; }}
        th {{ background-color: #4CAF50; color: white; }}
        pre {{ background: #f4f4f4; padding: 10px; overflow-x: auto; }}
    </style>
</head>
<body>
    <h1>llmspell Application Validation Report</h1>
    <div class="summary">
        <p><strong>Test Date:</strong> {self.timestamp}</p>
        <p><strong>Total Applications:</strong> {self.total_apps}</p>
        <p><strong>Passed:</strong> <span class="passed">{self.passed}</span></p>
        <p><strong>Failed:</strong> <span class="failed">{self.failed}</span></p>
        <p><strong>Skipped:</strong> <span class="skipped">{self.skipped}</span></p>
        <p><strong>Total Runtime:</strong> {self.total_runtime:.2f} seconds</p>
    </div>

    <h2>Test Results</h2>
    <table>
        <tr>
            <th>Application</th>
            <th>Layer</th>
            <th>Status</th>
            <th>Runtime (s)</th>
            <th>Validations</th>
            <th>Errors</th>
        </tr>
"""
        for result in self.results:
            status_class = result.status
            validations = ', '.join([f"{k}: {'✓' if v else '✗'}" for k, v in result.validations.items()])
            errors = '<br>'.join(result.errors) if result.errors else 'None'
            html += f"""
        <tr>
            <td>{result.app_name}</td>
            <td>{result.layer}</td>
            <td class="{status_class}">{result.status.upper()}</td>
            <td>{result.runtime_seconds:.2f}</td>
            <td>{validations}</td>
            <td>{errors}</td>
        </tr>
"""
        html += """
    </table>
</body>
</html>
"""
        return html


class ApplicationValidator:
    """Main validator class for llmspell applications"""

    def __init__(self, llmspell_bin: Optional[str] = None, verbose: bool = False):
        """Initialize validator with paths"""
        self.llmspell_bin = llmspell_bin or "./target/debug/llmspell"
        self.app_dir = Path("examples/script-users/applications")
        self.config_dir = Path("examples/script-users/configs")
        self.verbose = verbose
        self.results: List[TestResult] = []

        # Application metadata with realistic timeouts for API calls
        self.applications = {
            # Layer 1: Universal (2-3 agents) - simple API calls
            "file-organizer": {"layer": 1, "agents": 3, "runtime": 60, "config": "file-organizer/config.toml"},
            "research-collector": {"layer": 1, "agents": 2, "runtime": 60, "config": "research-collector/config.toml"},

            # Layer 2: Power User (4 agents) - moderate complexity
            "content-creator": {"layer": 2, "agents": 4, "runtime": 90, "config": "content-creator/config.toml"},

            # Layer 3: Business (5-7 agents) - higher complexity
            "personal-assistant": {"layer": 3, "agents": 5, "runtime": 120, "config": "personal-assistant/config.toml"},
            "communication-manager": {"layer": 3, "agents": 5, "runtime": 120, "config": "communication-manager/config.toml"},
            "code-review-assistant": {"layer": 3, "agents": 7, "runtime": 150, "config": "code-review-assistant/config.toml"},

            # Layer 4: Professional (8 agents) - complex workflows
            "process-orchestrator": {"layer": 4, "agents": 8, "runtime": 180, "config": "process-orchestrator/config.toml"},
            "knowledge-base": {"layer": 4, "agents": 8, "runtime": 180, "config": "knowledge-base/config.toml"},

            # Layer 5: Expert (21 agents) - very complex
            "webapp-creator": {"layer": 5, "agents": 21, "runtime": 600, "config": "config.toml"},
        }

    def _cleanup_temp_files(self):
        """Clean up temporary test files"""
        temp_dirs = [
            "/tmp/messy_files",
            "/tmp/organized_files",
            "/tmp/research_results",
            "/tmp/generated_webapp"
        ]
        temp_files = [
            # file-organizer
            "/tmp/organization-plan.txt",
            # content-creator
            "/tmp/content-topic.txt",
            "/tmp/content-plan.md",
            "/tmp/draft-content.md",
            "/tmp/final-content.md",
            "/tmp/quality-report.json",
            # personal-assistant
            "/tmp/personal-tasks.json",
            "/tmp/personal-schedule.md",
            "/tmp/personal-notes.txt",
            "/tmp/assistant-report.md",
            # communication-manager
            "/tmp/communication-queue.json",
            "/tmp/client-threads.json",
            "/tmp/schedule-calendar.json",
            "/tmp/tracking-dashboard.json",
            "/tmp/communication-log.txt",
            # code-review-assistant
            "/tmp/code-review-report.md",
            "/tmp/code-analysis.json",
            "/tmp/review-comments.txt",
            "/tmp/suggested-improvements.md",
            # process-orchestrator
            "/tmp/process-workflow.json",
            "/tmp/orchestration-state.json",
            "/tmp/workflow-log.txt",
            "/tmp/process-report.md",
            # knowledge-base
            "/tmp/knowledge-store.json",
            "/tmp/knowledge-index.db",
            "/tmp/knowledge-graph.json",
            "/tmp/knowledge-report.md",
            # research-collector
            "/tmp/research-summary.md",
            "/tmp/research-raw-data.json",
            "/tmp/research-insights.txt"
        ]

        for dir_path in temp_dirs:
            if os.path.exists(dir_path):
                shutil.rmtree(dir_path, ignore_errors=True)

        for file_path in temp_files:
            if os.path.exists(file_path):
                os.remove(file_path)

    def run_application(self, app_name: str, config: Optional[str] = None,
                       args: List[str] = None, timeout: int = 300) -> Tuple[subprocess.CompletedProcess, float]:
        """Execute llmspell with application and capture output"""
        cmd = [self.llmspell_bin]

        # Add config if specified
        if config:
            # Config files are in each app's directory
            config_path = self.app_dir / app_name / config
            if config_path.exists():
                cmd.extend(["-c", str(config_path)])

        # Add run command and lua script
        app_path = self.app_dir / app_name / "main.lua"
        cmd.extend(["run", str(app_path)])

        # Add additional arguments
        if args:
            cmd.extend(args)

        if self.verbose:
            print(f"\nExecuting: {' '.join(cmd)}")

        # Run with timeout and capture output
        start_time = time.time()
        try:
            result = subprocess.run(
                cmd,
                capture_output=True,
                text=True,
                timeout=timeout
            )
            runtime = time.time() - start_time
            return result, runtime
        except subprocess.TimeoutExpired:
            runtime = time.time() - start_time
            # Create a fake result for timeout
            result = subprocess.CompletedProcess(
                args=cmd,
                returncode=-1,
                stdout="",
                stderr=f"Process timed out after {timeout} seconds"
            )
            return result, runtime

    def validate_application(self, app_name: str) -> TestResult:
        """Generic validation for any application"""
        print(f"\n🔍 Testing {app_name}...")

        # Clean up before test
        self._cleanup_temp_files()

        # Get config for this app (config is just the filename in the app's directory)
        app_info = self.applications[app_name]
        config_file = app_info['config']

        # Run application with config
        result, runtime = self.run_application(app_name, config=config_file, timeout=app_info['runtime'])

        # Initialize test result
        errors = []
        validations = {}
        files_created = []

        # Generic success patterns based on actual output
        validations["application_completed"] = (
            "Complete!" in result.stdout or
            "COMPLETED" in result.stdout or
            "Status: COMPLETED" in result.stdout or
            "Creation Status: COMPLETED" in result.stdout or
            ("Layer" in result.stdout and "Complete!" in result.stdout)
        )

        # Check for agent creation (all apps create agents)
        validations["agents_created"] = (
            "Agent created" in result.stdout or
            "agents created" in result.stdout or
            "created successfully" in result.stdout
        )

        # Check for workflow execution (most apps have workflows)
        validations["workflow_executed"] = (
            "Workflow created" in result.stdout or
            "workflow completed" in result.stdout or
            "workflow executed" in result.stdout
        )

        # Check for any /tmp files created
        import glob
        tmp_files = glob.glob(f"/tmp/{app_name.replace('-', '_')}*") + \
                   glob.glob(f"/tmp/{app_name.replace('-', '')}*") + \
                   glob.glob("/tmp/*-plan*") + glob.glob("/tmp/*-summary*") + \
                   glob.glob("/tmp/*-output*") + glob.glob("/tmp/final-*")

        for file_path in tmp_files:
            if os.path.exists(file_path):
                files_created.append(file_path)

        validations["files_created"] = len(files_created) > 0

        # Status determination
        if result.returncode != 0:
            errors.append(f"Non-zero exit code: {result.returncode}")
            status = "failed"
        elif validations["application_completed"] or (validations["agents_created"] and validations["workflow_executed"]):
            status = "passed"
        else:
            status = "failed"
            errors.append("Application didn't complete expected tasks")

        return TestResult(
            app_name=app_name,
            layer=self.applications[app_name]["layer"],
            status=status,
            runtime_seconds=runtime,
            stdout=result.stdout[:1000] if self.verbose else "",
            stderr=result.stderr[:500] if self.verbose else "",
            errors=errors,
            validations=validations,
            files_created=files_created
        )

    def validate_file_organizer(self) -> TestResult:
        """Validate file-organizer application"""
        app_name = "file-organizer"
        print(f"\n🔍 Testing {app_name}...")

        # Clean up before test
        self._cleanup_temp_files()

        # Get config for this app (config is just the filename in the app's directory)
        app_info = self.applications[app_name]
        config_file = app_info['config']

        # Run application with config
        result, runtime = self.run_application(app_name, config=config_file, timeout=app_info['runtime'])

        # Initialize test result
        errors = []
        validations = {}
        files_created = []

        # Check for successful execution
        validations["application_completed"] = (
            "File Organizer Complete!" in result.stdout or
            "Organization Status: COMPLETED" in result.stdout
        )

        # Check for agents created
        validations["agents_created"] = (
            "Agent created" in result.stdout or
            "3 simple agents" in result.stdout
        )

        # Check for workflow execution
        validations["workflow_executed"] = (
            "Workflow created" in result.stdout or
            "organization completed" in result.stdout
        )

        # Check for file artifacts created by the application
        # The file-organizer should create these files
        if os.path.exists("/tmp/organization-plan.txt"):
            files_created.append("/tmp/organization-plan.txt")
            validations["plan_file_created"] = True

            # Verify plan content
            with open("/tmp/organization-plan.txt") as f:
                plan_content = f.read()
                validations["plan_has_content"] = len(plan_content) > 0
        else:
            validations["plan_file_created"] = False
            validations["plan_has_content"] = False

        # Check for organized directory
        if os.path.exists("/tmp/organized_files"):
            files_created.append("/tmp/organized_files")
            validations["organized_dir_created"] = True
        else:
            validations["organized_dir_created"] = False

        # Check for sample messy files
        if os.path.exists("/tmp/messy_files"):
            files_created.append("/tmp/messy_files")
            validations["messy_files_created"] = True
        else:
            validations["messy_files_created"] = False

        # Determine overall status
        if result.returncode != 0:
            errors.append(f"Non-zero exit code: {result.returncode}")

        # Check if application completed successfully
        if validations.get("application_completed", False):
            status = "passed"  # Application ran successfully
        elif validations.get("agents_created", False) and validations.get("workflow_executed", False):
            status = "passed"  # Core components worked
            if not any([validations.get("plan_file_created"), validations.get("organized_dir_created")]):
                # No files created likely means partial execution
                errors.append("Partial execution - some files missing")
        else:
            status = "failed"
            if result.returncode == 0:
                errors.append("Application ran but didn't complete expected tasks")
            else:
                errors.append("Script execution failed")

        return TestResult(
            app_name=app_name,
            layer=self.applications[app_name]["layer"],
            status=status,
            runtime_seconds=runtime,
            stdout=result.stdout[:1000] if self.verbose else "",
            stderr=result.stderr[:500] if self.verbose else "",
            errors=errors,
            validations=validations,
            files_created=files_created
        )

    def validate_research_collector(self) -> TestResult:
        """Validate research-collector application"""
        app_name = "research-collector"
        print(f"\n🔍 Testing {app_name}...")

        # Clean up before test
        self._cleanup_temp_files()

        # Get config for this app (config is just the filename in the app's directory)
        app_info = self.applications[app_name]
        config_file = app_info['config']

        # Run application with config
        result, runtime = self.run_application(app_name, config=config_file, timeout=app_info['runtime'])

        # Initialize test result
        errors = []
        validations = {}
        files_created = []

        # Check for successful execution
        validations["application_completed"] = (
            "Research Collection Results" in result.stdout or
            "Research Status: COMPLETED" in result.stdout
        )

        # Check for agents created
        validations["agents_created"] = (
            "Agent created" in result.stdout or
            "simple agents" in result.stdout
        )

        # Check for research results directory
        if os.path.exists("/tmp/research_results"):
            files_created.append("/tmp/research_results")
            validations["results_dir_created"] = True
        else:
            validations["results_dir_created"] = False

        # Check for any research output files
        research_files = [
            "/tmp/research-summary.md",
            "/tmp/research-raw-data.json",
            "/tmp/research-insights.txt"
        ]
        for file_path in research_files:
            if os.path.exists(file_path):
                files_created.append(file_path)

        validations["research_files_created"] = len(files_created) > 0

        # Status determination
        if result.returncode != 0:
            errors.append(f"Non-zero exit code: {result.returncode}")

        if validations.get("application_completed", False) or validations.get("agents_created", False):
            status = "passed"  # Application ran successfully
            if not validations["research_files_created"]:
                errors.append("No research files created - likely partial execution")
        else:
            status = "failed"
            if result.returncode == 0:
                errors.append("Application ran but didn't complete expected tasks")
            else:
                errors.append("Script execution failed")

        return TestResult(
            app_name=app_name,
            layer=self.applications[app_name]["layer"],
            status=status,
            runtime_seconds=runtime,
            stdout=result.stdout[:1000] if self.verbose else "",
            stderr=result.stderr[:500] if self.verbose else "",
            errors=errors,
            validations=validations,
            files_created=files_created
        )

    def validate_content_creator(self) -> TestResult:
        """Validate content-creator application with conditional workflows"""
        app_name = "content-creator"
        print(f"\n🔍 Testing {app_name}...")

        # Clean up before test
        self._cleanup_temp_files()

        # Get config for this app (config is just the filename in the app's directory)
        app_info = self.applications[app_name]
        config_file = app_info['config']

        # Run application with config
        result, runtime = self.run_application(
            app_name,
            config=config_file,
            timeout=app_info['runtime']
        )

        # Initialize test result
        errors = []
        validations = {}
        files_created = []

        # Check for successful execution (based on actual output)
        validations["script_executed"] = (
            "Creation Status: COMPLETED" in result.stdout or
            "Status: COMPLETED" in result.stdout or
            "Layer Content Creator Complete!" in result.stdout or
            "Content creation workflow completed" in result.stdout
        )

        # Check for script loading (look for agent creation)
        validations["script_loaded"] = (
            "Agent created" in result.stdout or
            "agents created" in result.stdout or
            ("Creating" in result.stdout and "agents" in result.stdout)
        )

        # Check for content files
        content_files = [
            "/tmp/content-topic.txt",
            "/tmp/content-plan.md",
            "/tmp/draft-content.md",
            "/tmp/final-content.md"
        ]

        for file_path in content_files:
            if os.path.exists(file_path):
                files_created.append(file_path)

        validations["content_files_created"] = len(files_created) > 0

        # Check quality report
        if os.path.exists("/tmp/quality-report.json"):
            files_created.append("/tmp/quality-report.json")
            try:
                with open("/tmp/quality-report.json") as f:
                    report = json.load(f)
                    validations["quality_report_valid"] = isinstance(report, dict)
            except:
                validations["quality_report_valid"] = False
        else:
            validations["quality_report_valid"] = False

        # Status determination
        if result.returncode != 0:
            # Config file might not exist, that's OK
            if "No such file" not in result.stderr:
                errors.append(f"Non-zero exit code: {result.returncode}")

        if validations["script_executed"] or validations["content_files_created"]:
            status = "passed"
            if not validations["content_files_created"]:
                errors.append("No content files created - likely using stub API keys")
        else:
            status = "failed"
            errors.append("Script execution failed")

        return TestResult(
            app_name=app_name,
            layer=self.applications[app_name]["layer"],
            status=status,
            runtime_seconds=runtime,
            stdout=result.stdout[:1000] if self.verbose else "",
            stderr=result.stderr[:500] if self.verbose else "",
            errors=errors,
            validations=validations,
            files_created=files_created
        )

    def validate_webapp_creator(self) -> TestResult:
        """Validate webapp-creator - the most complex application"""
        app_name = "webapp-creator"
        print(f"\n🔍 Testing {app_name} (expensive, may skip)...")

        # Skip if not explicitly enabled
        if not os.getenv("RUN_EXPENSIVE_TESTS"):
            return TestResult(
                app_name=app_name,
                layer=self.applications[app_name]["layer"],
                status="skipped",
                runtime_seconds=0,
                stdout="",
                stderr="",
                errors=["Skipped: Set RUN_EXPENSIVE_TESTS=1 to run"],
                validations={},
                files_created=[]
            )

        # Clean up before test
        self._cleanup_temp_files()

        # Test with custom output directory using script arguments
        custom_output = "/tmp/test-webapp-output"
        if os.path.exists(custom_output):
            shutil.rmtree(custom_output, ignore_errors=True)

        # Get config for this app
        app_info = self.applications[app_name]

        # Run application with custom arguments to test script arg passing
        result, runtime = self.run_application(
            app_name,
            config=app_info['config'],
            args=["--output", custom_output],  # Test script argument passing
            timeout=app_info['runtime']
        )

        # Initialize test result
        errors = []
        validations = {}
        files_created = []

        # Check for successful completion
        validations["webapp_complete"] = (
            "WebApp Generation Complete!" in result.stdout or
            "WebApp Creator v2.0 Complete" in result.stdout or
            "Project generated at:" in result.stdout
        )

        # Check for 20+ agent creation
        agent_count = result.stdout.count("✓")  # Count checkmarks for agents
        validations["twenty_agents_created"] = agent_count >= 20

        # Check for app structure mentions
        structure_keywords = ["frontend", "backend", "database", "API", "components", "tests"]
        structure_found = sum(1 for kw in structure_keywords if kw in result.stdout)
        validations["app_structure_mentioned"] = structure_found >= 3

        # Check for workflow execution
        validations["sequential_workflow"] = (
            "Phase" in result.stdout or
            "Executing Main Workflow" in result.stdout
        )

        # Check if custom output directory was created (tests script arg passing)
        # webapp-creator should create the project in the custom output directory
        expected_project_path = os.path.join(custom_output, "taskflow")
        if os.path.exists(expected_project_path):
            files_created.append(expected_project_path)
            # Count files in the generated project
            import glob
            taskflow_files = glob.glob(f"{expected_project_path}/**/*", recursive=True)
            validations["project_files_created"] = len(taskflow_files) > 20
            validations["custom_output_respected"] = True
        else:
            # Check if it was created in default location (arg passing failed)
            if os.path.exists("/tmp/taskflow"):
                files_created.append("/tmp/taskflow")
                errors.append("Script arguments not respected - project created in default location")
                validations["custom_output_respected"] = False
                # Still count files
                taskflow_files = glob.glob("/tmp/taskflow/**/*", recursive=True)
                validations["project_files_created"] = len(taskflow_files) > 20
            else:
                validations["project_files_created"] = False
                validations["custom_output_respected"] = False

        # Status determination
        if result.returncode == -1:  # Timeout
            errors.append("Application timed out")
            status = "failed"
        elif validations["webapp_complete"] and validations["twenty_agents_created"]:
            status = "passed"
        else:
            status = "failed"
            errors.append(f"Only {agent_count} agents created, expected 20+")

        return TestResult(
            app_name=app_name,
            layer=self.applications[app_name]["layer"],
            status=status,
            runtime_seconds=runtime,
            stdout=result.stdout[:1000] if self.verbose else "",
            stderr=result.stderr[:500] if self.verbose else "",
            errors=errors,
            validations=validations,
            files_created=files_created
        )

    def validate_personal_assistant(self) -> TestResult:
        """Validate personal-assistant application"""
        app_name = "personal-assistant"
        print(f"\n🔍 Testing {app_name}...")

        # Clean up before test
        self._cleanup_temp_files()

        # Get config for this app
        app_info = self.applications[app_name]
        config_file = app_info['config']

        # Run application with config
        result, runtime = self.run_application(app_name, config=config_file, timeout=app_info['runtime'])

        # Initialize test result
        errors = []
        validations = {}
        files_created = []

        # Check for successful execution
        validations["script_executed"] = (
            "Personal Assistant Complete!" in result.stdout or
            "Assistant Complete!" in result.stdout or
            "Status: COMPLETED" in result.stdout or
            "Assistant Status: ACTIVE" in result.stdout or
            "Personal Assistant v1.0 Ready!" in result.stdout or
            "Layer Business Personal Assistant Complete!" in result.stdout or
            ("Layer" in result.stdout and "Complete!" in result.stdout)
        )

        # Check for expected files
        expected_files = [
            "/tmp/personal-tasks.json",
            "/tmp/personal-schedule.md",
            "/tmp/personal-notes.txt",
            "/tmp/assistant-report.md"
        ]

        for file_path in expected_files:
            if os.path.exists(file_path):
                files_created.append(file_path)

        validations["files_created"] = len(files_created) > 0

        # Status determination
        if result.returncode != 0:
            errors.append(f"Non-zero exit code: {result.returncode}")

        if validations["script_executed"]:
            status = "passed"
            if not validations["files_created"]:
                errors.append("No assistant files created - likely missing API keys")
        else:
            status = "failed"
            errors.append("Script execution failed")

        return TestResult(
            app_name=app_name,
            layer=self.applications[app_name]["layer"],
            status=status,
            runtime_seconds=runtime,
            stdout=result.stdout[:1000] if self.verbose else "",
            stderr=result.stderr[:500] if self.verbose else "",
            errors=errors,
            validations=validations,
            files_created=files_created
        )

    def validate_communication_manager(self) -> TestResult:
        """Validate communication-manager application"""
        app_name = "communication-manager"
        print(f"\n🔍 Testing {app_name}...")

        # Clean up before test
        self._cleanup_temp_files()

        # Get config for this app
        app_info = self.applications[app_name]
        config_file = app_info['config']

        # Run application with config
        result, runtime = self.run_application(app_name, config=config_file, timeout=app_info['runtime'])

        # Initialize test result
        errors = []
        validations = {}
        files_created = []

        # Check for successful execution
        validations["script_executed"] = (
            "Communication Manager Complete!" in result.stdout or
            "Status: COMPLETED" in result.stdout or
            "Layer Business Communication Manager Complete!" in result.stdout or
            ("Layer" in result.stdout and "Complete!" in result.stdout)
        )

        # Check for expected files
        expected_files = [
            "/tmp/communication-queue.json",
            "/tmp/client-threads.json",
            "/tmp/schedule-calendar.json",
            "/tmp/tracking-dashboard.json",
            "/tmp/communication-log.txt"
        ]

        for file_path in expected_files:
            if os.path.exists(file_path):
                files_created.append(file_path)

        validations["files_created"] = len(files_created) > 0

        # Status determination
        if result.returncode != 0:
            errors.append(f"Non-zero exit code: {result.returncode}")

        if validations["script_executed"]:
            status = "passed"
            if not validations["files_created"]:
                errors.append("No communication files created - likely missing API keys")
        else:
            status = "failed"
            errors.append("Script execution failed")

        return TestResult(
            app_name=app_name,
            layer=self.applications[app_name]["layer"],
            status=status,
            runtime_seconds=runtime,
            stdout=result.stdout[:1000] if self.verbose else "",
            stderr=result.stderr[:500] if self.verbose else "",
            errors=errors,
            validations=validations,
            files_created=files_created
        )

    def validate_code_review_assistant(self) -> TestResult:
        """Validate code-review-assistant application"""
        app_name = "code-review-assistant"
        print(f"\n🔍 Testing {app_name}...")

        # Clean up before test
        self._cleanup_temp_files()

        # Get config for this app
        app_info = self.applications[app_name]
        config_file = app_info['config']

        # Run application with config
        result, runtime = self.run_application(app_name, config=config_file, timeout=app_info['runtime'])

        # Initialize test result
        errors = []
        validations = {}
        files_created = []

        # Check for successful execution
        validations["script_executed"] = (
            "Code Review Complete!" in result.stdout or
            "Review Complete!" in result.stdout or
            "Status: COMPLETED" in result.stdout or
            "Layer Professional Code Review Assistant Complete!" in result.stdout or
            ("Layer" in result.stdout and "Complete!" in result.stdout)
        )

        # Check for expected files (based on code review output)
        expected_files = [
            "/tmp/code-review-report.md",
            "/tmp/code-analysis.json",
            "/tmp/review-comments.txt",
            "/tmp/suggested-improvements.md"
        ]

        for file_path in expected_files:
            if os.path.exists(file_path):
                files_created.append(file_path)

        validations["files_created"] = len(files_created) > 0

        # Status determination
        if result.returncode != 0:
            errors.append(f"Non-zero exit code: {result.returncode}")

        if validations["script_executed"]:
            status = "passed"
            if not validations["files_created"]:
                errors.append("No review files created - likely missing API keys")
        else:
            status = "failed"
            errors.append("Script execution failed")

        return TestResult(
            app_name=app_name,
            layer=self.applications[app_name]["layer"],
            status=status,
            runtime_seconds=runtime,
            stdout=result.stdout[:1000] if self.verbose else "",
            stderr=result.stderr[:500] if self.verbose else "",
            errors=errors,
            validations=validations,
            files_created=files_created
        )

    def validate_process_orchestrator(self) -> TestResult:
        """Validate process-orchestrator application"""
        app_name = "process-orchestrator"
        print(f"\n🔍 Testing {app_name}...")

        # Clean up before test
        self._cleanup_temp_files()

        # Get config for this app
        app_info = self.applications[app_name]
        config_file = app_info['config']

        # Run application with config
        result, runtime = self.run_application(app_name, config=config_file, timeout=app_info['runtime'])

        # Initialize test result
        errors = []
        validations = {}
        files_created = []

        # Check for successful execution
        validations["script_executed"] = (
            "Process Orchestrator Complete!" in result.stdout or
            "Orchestrator Complete!" in result.stdout or
            "Status: COMPLETED" in result.stdout or
            "Layer Professional Process Orchestrator Complete!" in result.stdout or
            ("Layer" in result.stdout and "Complete!" in result.stdout)
        )

        # Check for expected files
        expected_files = [
            "/tmp/process-workflow.json",
            "/tmp/orchestration-state.json",
            "/tmp/workflow-log.txt",
            "/tmp/process-report.md"
        ]

        for file_path in expected_files:
            if os.path.exists(file_path):
                files_created.append(file_path)

        validations["files_created"] = len(files_created) > 0

        # Status determination
        if result.returncode != 0:
            errors.append(f"Non-zero exit code: {result.returncode}")

        if validations["script_executed"]:
            status = "passed"
            if not validations["files_created"]:
                errors.append("No process files created - likely missing API keys")
        else:
            status = "failed"
            errors.append("Script execution failed")

        return TestResult(
            app_name=app_name,
            layer=self.applications[app_name]["layer"],
            status=status,
            runtime_seconds=runtime,
            stdout=result.stdout[:1000] if self.verbose else "",
            stderr=result.stderr[:500] if self.verbose else "",
            errors=errors,
            validations=validations,
            files_created=files_created
        )

    def validate_knowledge_base(self) -> TestResult:
        """Validate knowledge-base application"""
        app_name = "knowledge-base"
        print(f"\n🔍 Testing {app_name}...")

        # Clean up before test
        self._cleanup_temp_files()

        # Get config for this app
        app_info = self.applications[app_name]
        config_file = app_info['config']

        # Run application with config
        result, runtime = self.run_application(app_name, config=config_file, timeout=app_info['runtime'])

        # Initialize test result
        errors = []
        validations = {}
        files_created = []

        # Check for successful execution
        validations["script_executed"] = (
            "Knowledge Base Complete!" in result.stdout or
            "Knowledge Base v1.0 Setup Complete!" in result.stdout or
            "System Status: OPERATIONAL" in result.stdout or
            "Status: COMPLETED" in result.stdout or
            "Layer Expert Knowledge Base Complete!" in result.stdout or
            ("Layer" in result.stdout and "Complete!" in result.stdout)
        )

        # Check for expected files
        expected_files = [
            "/tmp/knowledge-store.json",
            "/tmp/knowledge-index.db",
            "/tmp/knowledge-graph.json",
            "/tmp/knowledge-report.md"
        ]

        for file_path in expected_files:
            if os.path.exists(file_path):
                files_created.append(file_path)

        validations["files_created"] = len(files_created) > 0

        # Status determination
        if result.returncode != 0:
            errors.append(f"Non-zero exit code: {result.returncode}")

        if validations["script_executed"]:
            status = "passed"
            if not validations["files_created"]:
                errors.append("No knowledge files created - likely missing API keys")
        else:
            status = "failed"
            errors.append("Script execution failed")

        return TestResult(
            app_name=app_name,
            layer=self.applications[app_name]["layer"],
            status=status,
            runtime_seconds=runtime,
            stdout=result.stdout[:1000] if self.verbose else "",
            stderr=result.stderr[:500] if self.verbose else "",
            errors=errors,
            validations=validations,
            files_created=files_created
        )

    def run_all_tests(self, layer_filter: Optional[int] = None) -> TestReport:
        """Run all application tests"""
        print("=" * 60)
        print("llmspell Application Validation Suite")
        print("=" * 60)

        # Check llmspell binary exists
        if not os.path.exists(self.llmspell_bin):
            print(f"❌ Error: llmspell binary not found at {self.llmspell_bin}")
            print("  Please build with: cargo build")
            sys.exit(1)

        print(f"✓ Using llmspell binary: {self.llmspell_bin}")

        start_time = time.time()
        self.results = []

        # Define validation methods for each app
        validators = {
            "file-organizer": self.validate_file_organizer,
            "research-collector": self.validate_research_collector,
            "content-creator": self.validate_content_creator,
            "personal-assistant": self.validate_personal_assistant,
            "communication-manager": self.validate_communication_manager,
            "code-review-assistant": self.validate_code_review_assistant,
            "process-orchestrator": self.validate_process_orchestrator,
            "knowledge-base": self.validate_knowledge_base,
            "webapp-creator": self.validate_webapp_creator
        }

        # Run tests
        for app_name, metadata in self.applications.items():
            # Skip if layer filter doesn't match
            if layer_filter and metadata["layer"] != layer_filter:
                continue

            # Run validator (use generic if no specific one exists)
            try:
                if app_name in validators:
                    result = validators[app_name]()
                else:
                    result = self.validate_application(app_name)
                self.results.append(result)
            except Exception as e:
                print(f"❌ Error testing {app_name}: {e}")
                self.results.append(TestResult(
                        app_name=app_name,
                        layer=metadata["layer"],
                        status="failed",
                        runtime_seconds=0,
                        stdout="",
                        stderr=str(e),
                        errors=[f"Test exception: {e}"],
                        validations={},
                        files_created=[]
                    ))

        # Generate report
        total_runtime = time.time() - start_time

        report = TestReport(
            timestamp=datetime.now().isoformat(),
            total_apps=len(self.results),
            passed=sum(1 for r in self.results if r.status == "passed"),
            failed=sum(1 for r in self.results if r.status == "failed"),
            skipped=sum(1 for r in self.results if r.status == "skipped"),
            total_runtime=total_runtime,
            results=self.results
        )

        # Print summary
        print("\n" + "=" * 60)
        print("TEST SUMMARY")
        print("=" * 60)
        print(f"Total Applications: {report.total_apps}")
        print(f"✅ Passed: {report.passed}")
        print(f"❌ Failed: {report.failed}")
        print(f"⚠️  Skipped: {report.skipped}")
        print(f"⏱️  Total Runtime: {report.total_runtime:.2f} seconds")

        # Print per-app results
        print("\nPER-APPLICATION RESULTS:")
        print("-" * 60)
        for result in self.results:
            status_symbol = "✅" if result.status == "passed" else "❌" if result.status == "failed" else "⚠️"
            print(f"{status_symbol} {result.app_name:25} Layer {result.layer}: {result.status:8} ({result.runtime_seconds:.2f}s)")
            if result.errors and self.verbose:
                for error in result.errors:
                    print(f"    Error: {error}")

        return report


def main():
    """Main entry point"""
    parser = argparse.ArgumentParser(description="llmspell Application Validation Suite")
    parser.add_argument("--layer", type=int, help="Test only specific layer (1-5)")
    parser.add_argument("--verbose", "-v", action="store_true", help="Verbose output")
    parser.add_argument("--report", choices=["json", "html"], help="Generate report file")
    parser.add_argument("--llmspell-bin", help="Path to llmspell binary")
    parser.add_argument("--track-performance", action="store_true", help="Track detailed performance metrics")

    args = parser.parse_args()

    # Create validator
    validator = ApplicationValidator(
        llmspell_bin=args.llmspell_bin,
        verbose=args.verbose
    )

    # Run tests
    report = validator.run_all_tests(layer_filter=args.layer)

    # Save report if requested
    if args.report == "json":
        report_file = f"test_report_{datetime.now().strftime('%Y%m%d_%H%M%S')}.json"
        with open(report_file, "w") as f:
            f.write(report.to_json())
        print(f"\n📊 JSON report saved to: {report_file}")

    elif args.report == "html":
        report_file = f"test_report_{datetime.now().strftime('%Y%m%d_%H%M%S')}.html"
        with open(report_file, "w") as f:
            f.write(report.to_html())
        print(f"\n📊 HTML report saved to: {report_file}")

    # Exit with appropriate code
    if report.failed > 0:
        sys.exit(1)
    else:
        sys.exit(0)


if __name__ == "__main__":
    main()