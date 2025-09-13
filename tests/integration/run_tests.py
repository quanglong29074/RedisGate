#!/usr/bin/env python3
"""
RedisGate Integration Test Runner

This script provides a comprehensive test runner for the RedisGate integration tests
with automatic dependency installation, server compilation, and test execution.

Usage:
    python run_tests.py [options]

Options:
    --mode MODE           Test mode: basic, advanced, all, ci, benchmark
    --host HOST           Server host (default: 127.0.0.1)
    --port PORT           Server port (default: 8080)
    --workers N           Number of parallel workers (default: auto)
    --verbose             Verbose output
    --report              Generate detailed report
    --install-deps        Install Python dependencies
    --skip-server-build   Skip building the RedisGate server
    --timeout SECONDS     Test timeout in seconds (default: 300)
    --help                Show this help message
"""

import argparse
import os
import sys
import subprocess
import time
import shutil
import venv
from pathlib import Path
from typing import List, Dict, Any

try:
    from rich.console import Console
    from rich.table import Table
    from rich.panel import Panel
    from rich.progress import Progress, SpinnerColumn, TextColumn
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
    """Main test runner for RedisGate integration tests."""
    
    def __init__(self, args):
        self.args = args
        self.test_dir = Path(__file__).parent
        self.project_root = self.test_dir.parent.parent
        self.venv_dir = self.test_dir / ".venv"
        self.results = {}
        
    def run_tests(self) -> bool:
        """Run the integration tests based on the selected mode."""
        if RICH_AVAILABLE:
            console.print(Panel.fit("🚀 RedisGate Integration Test Suite", style="bold blue"))
        else:
            console.print("🚀 RedisGate Integration Test Suite")
        
        # Setup environment
        if not self._setup_environment():
            return False
        
        # Build server if needed
        if not self.args.skip_server_build and not self._build_server():
            return False
        
        # Install dependencies if needed
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
    
    def _setup_environment(self) -> bool:
        """Set up the test environment."""
        console.print("[blue]Setting up test environment...[/blue]")
        
        # Check if we're in the right directory
        if not (self.project_root / "Cargo.toml").exists():
            console.print("[red]Error: Not in RedisGate project root directory[/red]")
            return False
        
        # Set environment variables
        os.environ["REDISGATE_TEST_HOST"] = self.args.host
        os.environ["REDISGATE_TEST_PORT"] = str(self.args.port)
        os.environ["RUST_LOG"] = "info"
        
        return True
    
    def _build_server(self) -> bool:
        """Build the RedisGate server."""
        console.print("[blue]Building RedisGate server...[/blue]")
        
        try:
            result = subprocess.run(
                ["cargo", "build"],
                cwd=self.project_root,
                capture_output=not self.args.verbose,
                text=True,
                timeout=300  # 5 minute timeout for build
            )
            
            if result.returncode == 0:
                console.print("[green]✅ Server built successfully[/green]")
                return True
            else:
                console.print(f"[red]❌ Server build failed with exit code {result.returncode}[/red]")
                if not self.args.verbose and result.stderr:
                    console.print(f"[red]Build errors:[/red]\n{result.stderr}")
                return False
                
        except subprocess.TimeoutExpired:
            console.print("[red]❌ Server build timed out[/red]")
            return False
        except Exception as e:
            console.print(f"[red]❌ Failed to build server: {e}[/red]")
            return False
    
    def _install_dependencies(self) -> bool:
        """Install Python dependencies in a virtual environment."""
        console.print("[blue]Installing Python dependencies...[/blue]")
        
        try:
            # Create virtual environment if it doesn't exist
            if not self.venv_dir.exists():
                console.print("[yellow]Creating virtual environment...[/yellow]")
                venv.create(self.venv_dir, with_pip=True)
            
            # Get pip executable
            if os.name == 'nt':  # Windows
                pip_cmd = self.venv_dir / "Scripts" / "pip.exe"
                python_cmd = self.venv_dir / "Scripts" / "python.exe"
            else:  # Unix-like
                pip_cmd = self.venv_dir / "bin" / "pip"
                python_cmd = self.venv_dir / "bin" / "python"
            
            # Upgrade pip
            subprocess.run([str(python_cmd), "-m", "pip", "install", "--upgrade", "pip"], 
                         check=True, capture_output=not self.args.verbose)
            
            # Install requirements
            requirements_file = self.test_dir / "requirements.txt"
            if requirements_file.exists():
                result = subprocess.run(
                    [str(pip_cmd), "install", "-r", str(requirements_file)],
                    capture_output=not self.args.verbose,
                    text=True
                )
                
                if result.returncode == 0:
                    console.print("[green]✅ Dependencies installed successfully[/green]")
                    return True
                else:
                    console.print(f"[red]❌ Failed to install dependencies[/red]")
                    if not self.args.verbose and result.stderr:
                        console.print(f"[red]Error:[/red]\n{result.stderr}")
                    return False
            else:
                console.print("[yellow]No requirements.txt found, skipping dependency installation[/yellow]")
                return True
                
        except Exception as e:
            console.print(f"[red]❌ Failed to install dependencies: {e}[/red]")
            return False
    
    def _check_dependencies(self) -> bool:
        """Check if required dependencies are available."""
        console.print("[blue]Checking dependencies...[/blue]")
        
        required_commands = ["cargo", "python3"]
        missing_commands = []
        
        for cmd in required_commands:
            if not shutil.which(cmd):
                missing_commands.append(cmd)
        
        if missing_commands:
            console.print(f"[red]❌ Missing required commands: {', '.join(missing_commands)}[/red]")
            return False
        
        # Check Python modules
        try:
            # Use the virtual environment python if available
            if self.venv_dir.exists():
                if os.name == 'nt':
                    python_cmd = self.venv_dir / "Scripts" / "python.exe"
                else:
                    python_cmd = self.venv_dir / "bin" / "python"
            else:
                python_cmd = "python3"
            
            result = subprocess.run(
                [str(python_cmd), "-c", "import pytest, httpx, upstash_redis"],
                capture_output=True,
                text=True
            )
            
            if result.returncode != 0:
                console.print("[red]❌ Required Python modules not available[/red]")
                console.print("[yellow]Run with --install-deps to install them automatically[/yellow]")
                return False
        
        except Exception as e:
            console.print(f"[red]❌ Failed to check Python dependencies: {e}[/red]")
            return False
        
        console.print("[green]✅ All dependencies available[/green]")
        return True
    
    def _execute_tests(self) -> bool:
        """Execute the tests using pytest."""
        console.print("[blue]Running integration tests...[/blue]")
        
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
        
        # Run tests
        try:
            result = subprocess.run(
                cmd,
                cwd=self.test_dir,
                env=env,
                capture_output=not self.args.verbose,
                text=True,
                timeout=self.args.timeout
            )
            
            if result.returncode == 0:
                console.print("[green]✅ All tests passed![/green]")
                return True
            else:
                console.print(f"[red]❌ Tests failed with exit code {result.returncode}[/red]")
                if not self.args.verbose and result.stdout:
                    console.print(result.stdout)
                if result.stderr:
                    console.print(f"[red]Errors:[/red]\n{result.stderr}")
                return False
                
        except subprocess.TimeoutExpired:
            console.print(f"[red]❌ Tests timed out after {self.args.timeout} seconds[/red]")
            return False
        except Exception as e:
            console.print(f"[red]❌ Failed to run tests: {e}[/red]")
            return False
    
    def _build_pytest_args(self) -> List[str]:
        """Build pytest command line arguments."""
        args = [
            ".",
            "-v",
            "--tb=short",
            "--color=yes",
        ]
        
        # Add mode-specific arguments
        if self.args.mode == "basic":
            args.extend([
                "-k", "test_basic_redis_operations",
                "-m", "not benchmark and not slow"
            ])
        elif self.args.mode == "advanced":
            args.extend([
                "-k", "test_advanced_redis_operations",
                "-m", "not benchmark"
            ])
        elif self.args.mode == "benchmark":
            args.extend([
                "-m", "benchmark"
            ])
        elif self.args.mode == "ci":
            args.extend([
                "-m", "not benchmark and not slow",
                "--junitxml=test-results.xml",
            ])
        elif self.args.mode == "all":
            # Run all tests except benchmarks by default
            args.extend([
                "-m", "not benchmark"
            ])
        
        # Add parallel execution
        if self.args.workers:
            args.extend(["-n", str(self.args.workers)])
        
        # Add verbose output
        if self.args.verbose:
            args.append("-s")
        
        # Add timeout
        args.extend(["--timeout", str(self.args.timeout)])
        
        return args
    
    def _generate_report(self, start_time: float, end_time: float):
        """Generate a detailed test report."""
        duration = end_time - start_time
        
        if RICH_AVAILABLE:
            # Create report table
            table = Table(title="RedisGate Integration Test Report")
            table.add_column("Metric", style="cyan")
            table.add_column("Value", style="magenta")
            
            table.add_row("Test Mode", self.args.mode)
            table.add_row("Duration", f"{duration:.2f}s")
            table.add_row("Python Version", sys.version.split()[0])
            table.add_row("Test Directory", str(self.test_dir))
            table.add_row("Project Root", str(self.project_root))
            
            console.print(table)
        else:
            console.print(f"Test Mode: {self.args.mode}")
            console.print(f"Duration: {duration:.2f}s")
            console.print(f"Python Version: {sys.version.split()[0]}")
        
        # Save report to file
        report_file = self.test_dir / "test_report.txt"
        with open(report_file, "w") as f:
            f.write(f"RedisGate Integration Test Report\n")
            f.write(f"==================================\n")
            f.write(f"Test Mode: {self.args.mode}\n")
            f.write(f"Duration: {duration:.2f}s\n")
            f.write(f"Python Version: {sys.version}\n")
            f.write(f"Test Directory: {self.test_dir}\n")
            f.write(f"Project Root: {self.project_root}\n")
            f.write(f"Timestamp: {time.strftime('%Y-%m-%d %H:%M:%S')}\n")
        
        console.print(f"[green]Report saved to: {report_file}[/green]")

def main():
    """Main entry point for the test runner."""
    parser = argparse.ArgumentParser(
        description="RedisGate Integration Test Runner",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  python run_tests.py --mode basic --install-deps      # Install deps and run basic tests
  python run_tests.py --mode advanced --verbose        # Run advanced tests with verbose output
  python run_tests.py --mode benchmark                 # Run performance benchmarks
  python run_tests.py --mode ci                        # Run CI/CD tests
  python run_tests.py --mode all --workers 4           # Run all tests with 4 workers
        """
    )
    
    parser.add_argument(
        "--mode",
        choices=["basic", "advanced", "all", "ci", "benchmark"],
        default="basic",
        help="Test mode to run (default: basic)"
    )
    
    parser.add_argument(
        "--host",
        default="127.0.0.1",
        help="Server host (default: 127.0.0.1)"
    )
    
    parser.add_argument(
        "--port",
        type=int,
        default=8080,
        help="Server port (default: 8080)"
    )
    
    parser.add_argument(
        "--workers",
        type=int,
        help="Number of parallel workers (default: auto)"
    )
    
    parser.add_argument(
        "--verbose",
        action="store_true",
        help="Verbose output"
    )
    
    parser.add_argument(
        "--report",
        action="store_true",
        help="Generate detailed report"
    )
    
    parser.add_argument(
        "--install-deps",
        action="store_true",
        help="Install Python dependencies automatically"
    )
    
    parser.add_argument(
        "--skip-server-build",
        action="store_true",
        help="Skip building the RedisGate server"
    )
    
    parser.add_argument(
        "--timeout",
        type=int,
        default=300,
        help="Test timeout in seconds (default: 300)"
    )
    
    args = parser.parse_args()
    
    # Run tests
    runner = TestRunner(args)
    success = runner.run_tests()
    
    # Exit with appropriate code
    sys.exit(0 if success else 1)

if __name__ == "__main__":
    main()