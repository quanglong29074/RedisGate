#!/usr/bin/env python3
"""
RedisGate Development Test Suite Runner

This script runs the complete development test suite for RedisGate.
It is designed to be used during development workflow:

1. Developer runs setup script (./setup-dev.sh)
2. Developer builds the project (cargo build)
3. Developer starts the server (cargo run)
4. Developer runs this test suite to verify all APIs work

Usage:
    python run_tests.py [options]

Examples:
    # Run all tests
    python run_tests.py

    # Run only public API tests
    python run_tests.py -m public

    # Run tests with verbose output
    python run_tests.py -v

    # Install dependencies and run tests
    python run_tests.py --install-deps

    # Run tests and generate HTML report
    python run_tests.py --report
"""

import argparse
import asyncio
import os
import subprocess
import sys
import time
from pathlib import Path
from typing import List, Optional

try:
    from rich.console import Console
    from rich.panel import Panel
    from rich.progress import Progress, SpinnerColumn, TextColumn
    from rich.table import Table
    RICH_AVAILABLE = True
except ImportError:
    RICH_AVAILABLE = False
    class Console:
        def print(self, *args, **kwargs):
            print(*args)
    console = Console()

if RICH_AVAILABLE:
    console = Console()
else:
    console = Console()


class TestRunner:
    """Main test runner for RedisGate development test suite."""
    
    def __init__(self, args):
        self.args = args
        self.test_dir = Path(__file__).parent
        self.project_root = self.test_dir.parent.parent
        self.venv_dir = self.test_dir / ".venv"
        self.results = {}
    
    def run_tests(self) -> bool:
        """Run the development test suite based on the selected options."""
        if RICH_AVAILABLE:
            console.print(Panel.fit("🚀 RedisGate Development Test Suite", style="bold blue"))
        else:
            console.print("🚀 RedisGate Development Test Suite")
        
        # Check server is running
        if not self._wait_for_server():
            return False
        
        # Install dependencies if requested
        if self.args.install_deps and not self._install_dependencies():
            return False
        
        # Check dependencies
        if not self._check_dependencies():
            return False
        
        # Run the tests
        start_time = time.time()
        success = self._execute_tests()
        end_time = time.time()
        
        # Generate report if requested
        if self.args.report:
            self._generate_report(start_time, end_time)
        
        return success
    
    def _wait_for_server(self) -> bool:
        """Wait for RedisGate server to be available."""
        console.print("[blue]Checking if RedisGate server is running...[/blue]")
        
        import httpx
        
        max_attempts = 10
        for attempt in range(max_attempts):
            try:
                with httpx.Client() as client:
                    response = client.get(f"http://{self.args.host}:{self.args.port}/health", timeout=5.0)
                    if response.status_code == 200:
                        console.print(f"[green]✓ Server is ready at http://{self.args.host}:{self.args.port}[/green]")
                        return True
            except (httpx.ConnectError, httpx.TimeoutException):
                if attempt < max_attempts - 1:
                    console.print(f"[yellow]Waiting for server... (attempt {attempt + 1}/{max_attempts})[/yellow]")
                    time.sleep(2)
                else:
                    console.print(f"[red]✗ Server not available at http://{self.args.host}:{self.args.port}[/red]")
                    console.print("[yellow]Please make sure the RedisGate server is running:[/yellow]")
                    console.print("  1. Run: cargo build")
                    console.print("  2. Run: cargo run")
                    console.print("  3. Wait for server to start on port 8080")
                    return False
        
        return False
    
    def _install_dependencies(self) -> bool:
        """Install Python dependencies in a virtual environment."""
        console.print("[blue]Installing Python dependencies...[/blue]")
        
        try:
            # Create virtual environment if it doesn't exist
            if not self.venv_dir.exists():
                console.print("[blue]Creating virtual environment...[/blue]")
                subprocess.run([
                    sys.executable, "-m", "venv", str(self.venv_dir)
                ], check=True, cwd=self.test_dir)
            
            # Determine python executable in venv
            if os.name == 'nt':
                python_cmd = self.venv_dir / "Scripts" / "python.exe"
                pip_cmd = self.venv_dir / "Scripts" / "pip.exe"
            else:
                python_cmd = self.venv_dir / "bin" / "python"
                pip_cmd = self.venv_dir / "bin" / "pip"
            
            # Upgrade pip
            subprocess.run([
                str(pip_cmd), "install", "--upgrade", "pip"
            ], check=True, cwd=self.test_dir)
            
            # Install requirements
            subprocess.run([
                str(pip_cmd), "install", "-r", "requirements.txt"
            ], check=True, cwd=self.test_dir)
            
            console.print("[green]✓ Dependencies installed successfully[/green]")
            return True
            
        except subprocess.CalledProcessError as e:
            console.print(f"[red]✗ Failed to install dependencies: {e}[/red]")
            return False
    
    def _check_dependencies(self) -> bool:
        """Check if required dependencies are available."""
        console.print("[blue]Checking dependencies...[/blue]")
        
        # Determine python executable
        if self.venv_dir.exists():
            if os.name == 'nt':
                python_cmd = self.venv_dir / "Scripts" / "python.exe"
            else:
                python_cmd = self.venv_dir / "bin" / "python"
        else:
            python_cmd = "python3"
        
        # Check if pytest is available
        try:
            result = subprocess.run([
                str(python_cmd), "-c", "import pytest; import httpx; print('OK')"
            ], capture_output=True, text=True, cwd=self.test_dir)
            
            if result.returncode != 0:
                console.print("[red]✗ Required dependencies not found[/red]")
                console.print("Run with --install-deps to install them automatically")
                return False
            
            console.print("[green]✓ Dependencies are available[/green]")
            return True
            
        except FileNotFoundError:
            console.print("[red]✗ Python executable not found[/red]")
            return False
    
    def _execute_tests(self) -> bool:
        """Execute the tests using pytest."""
        console.print("[blue]Running development tests...[/blue]")
        
        # Determine python executable
        if self.venv_dir.exists():
            if os.name == 'nt':
                python_cmd = self.venv_dir / "Scripts" / "python.exe"
            else:
                python_cmd = self.venv_dir / "bin" / "python"
        else:
            python_cmd = "python3"
        
        # Build pytest command
        pytest_args = self._build_pytest_args()
        cmd = [str(python_cmd), "-m", "pytest"] + pytest_args
        
        console.print(f"[green]Running: {' '.join(cmd)}[/green]")
        
        # Set environment
        env = os.environ.copy()
        env["PYTHONPATH"] = str(self.test_dir)
        env["REDISGATE_TEST_HOST"] = self.args.host
        env["REDISGATE_TEST_PORT"] = str(self.args.port)
        
        # Run tests
        try:
            result = subprocess.run(
                cmd,
                cwd=self.test_dir,
                env=env,
                timeout=self.args.timeout
            )
            
            if result.returncode == 0:
                console.print("[green]✓ All tests passed![/green]")
                return True
            else:
                console.print(f"[red]✗ Tests failed with exit code {result.returncode}[/red]")
                return False
                
        except subprocess.TimeoutExpired:
            console.print(f"[red]✗ Tests timed out after {self.args.timeout} seconds[/red]")
            return False
        except KeyboardInterrupt:
            console.print("[yellow]Tests interrupted by user[/yellow]")
            return False
    
    def _build_pytest_args(self) -> List[str]:
        """Build pytest command line arguments."""
        args = []
        
        # Test selection by marker
        if self.args.marker:
            args.extend(["-m", self.args.marker])
        
        # Verbosity
        if self.args.verbose:
            args.append("-v")
        else:
            args.append("-q")
        
        # Show output
        if self.args.capture == "no":
            args.append("-s")
        
        # Parallel execution
        if self.args.workers and self.args.workers > 1:
            args.extend(["-n", str(self.args.workers)])
        
        # Test file selection
        if self.args.test_files:
            args.extend(self.args.test_files)
        else:
            args.append(".")  # Run all tests in current directory
        
        # HTML report
        if self.args.report:
            report_file = self.test_dir / "test_report.html"
            args.extend(["--html", str(report_file), "--self-contained-html"])
        
        # JSON report
        if self.args.json_report:
            json_file = self.test_dir / "test_report.json"
            args.extend(["--json-report", "--json-report-file", str(json_file)])
        
        return args
    
    def _generate_report(self, start_time: float, end_time: float):
        """Generate a detailed test report."""
        duration = end_time - start_time
        
        if RICH_AVAILABLE:
            # Create report table
            table = Table(title="RedisGate Development Test Report")
            table.add_column("Metric", style="cyan")
            table.add_column("Value", style="magenta")
            
            table.add_row("Test Mode", self.args.marker or "all")
            table.add_row("Duration", f"{duration:.2f}s")
            table.add_row("Python Version", sys.version.split()[0])
            table.add_row("Test Directory", str(self.test_dir))
            table.add_row("Server URL", f"http://{self.args.host}:{self.args.port}")
            
            console.print(table)
        else:
            console.print(f"Test Mode: {self.args.marker or 'all'}")
            console.print(f"Duration: {duration:.2f}s")
            console.print(f"Python Version: {sys.version.split()[0]}")
        
        # Save report to file if HTML report was generated
        if self.args.report:
            report_file = self.test_dir / "test_report.html"
            if report_file.exists():
                console.print(f"[green]HTML report saved to: {report_file}[/green]")


def parse_args():
    """Parse command line arguments."""
    parser = argparse.ArgumentParser(
        description="RedisGate Development Test Suite Runner",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog=__doc__
    )
    
    # Server configuration
    parser.add_argument("--host", default="127.0.0.1",
                       help="RedisGate server host (default: 127.0.0.1)")
    parser.add_argument("--port", type=int, default=8080,
                       help="RedisGate server port (default: 8080)")
    
    # Test selection
    parser.add_argument("-m", "--marker", 
                       choices=["public", "auth", "protected", "redis", "integration"],
                       help="Run tests with specific marker")
    parser.add_argument("test_files", nargs="*",
                       help="Specific test files to run")
    
    # Test execution
    parser.add_argument("-v", "--verbose", action="store_true",
                       help="Verbose output")
    parser.add_argument("-s", "--capture", choices=["yes", "no"], default="yes",
                       help="Capture output (default: yes)")
    parser.add_argument("-n", "--workers", type=int, default=1,
                       help="Number of parallel test workers")
    parser.add_argument("--timeout", type=int, default=300,
                       help="Test timeout in seconds (default: 300)")
    
    # Dependencies and setup
    parser.add_argument("--install-deps", action="store_true",
                       help="Install Python dependencies before running tests")
    
    # Reporting
    parser.add_argument("--report", action="store_true",
                       help="Generate HTML test report")
    parser.add_argument("--json-report", action="store_true",
                       help="Generate JSON test report")
    
    return parser.parse_args()


def main():
    """Main entry point for the test runner."""
    args = parse_args()
    
    runner = TestRunner(args)
    success = runner.run_tests()
    
    # Exit with appropriate code
    sys.exit(0 if success else 1)


if __name__ == "__main__":
    main()