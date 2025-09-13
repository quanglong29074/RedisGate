#!/usr/bin/env python3
"""
Validation script for RedisGate integration test setup.

This script validates that the integration test environment is properly configured
without requiring a running RedisGate server.
"""

import sys
import os
import subprocess
from pathlib import Path


def check_python_version():
    """Check if Python version is adequate."""
    version = sys.version_info
    if version.major < 3 or (version.major == 3 and version.minor < 8):
        print(f"❌ Python {version.major}.{version.minor} is too old. Python 3.8+ required.")
        return False
    print(f"✅ Python {version.major}.{version.minor}.{version.micro} is supported")
    return True


def check_rust_cargo():
    """Check if Rust and Cargo are available."""
    try:
        result = subprocess.run(["cargo", "--version"], capture_output=True, text=True)
        if result.returncode == 0:
            print(f"✅ Cargo available: {result.stdout.strip()}")
            return True
        else:
            print("❌ Cargo not available")
            return False
    except FileNotFoundError:
        print("❌ Cargo not found in PATH")
        return False


def check_dependencies():
    """Check if Python dependencies can be imported."""
    test_dir = Path(__file__).parent
    venv_dir = test_dir / ".venv"
    
    if not venv_dir.exists():
        print("❌ Virtual environment not created. Run with --install-deps first.")
        return False
    
    # Check if dependencies are installed
    if os.name == 'nt':
        python_cmd = venv_dir / "Scripts" / "python.exe"
    else:
        python_cmd = venv_dir / "bin" / "python"
    
    if not python_cmd.exists():
        print("❌ Python executable not found in virtual environment")
        return False
    
    try:
        result = subprocess.run(
            [str(python_cmd), "-c", "import pytest, httpx, upstash_redis"],
            capture_output=True,
            text=True
        )
        if result.returncode == 0:
            print("✅ All Python dependencies available")
            return True
        else:
            print(f"❌ Missing Python dependencies: {result.stderr}")
            return False
    except Exception as e:
        print(f"❌ Error checking dependencies: {e}")
        return False


def check_pytest_config():
    """Check if pytest configuration is valid."""
    test_dir = Path(__file__).parent
    venv_dir = test_dir / ".venv"
    
    if os.name == 'nt':
        python_cmd = venv_dir / "Scripts" / "python"
    else:
        python_cmd = venv_dir / "bin" / "python"
    
    try:
        result = subprocess.run(
            [str(python_cmd), "-m", "pytest", "--collect-only", "-q"],
            cwd=test_dir,
            capture_output=True,
            text=True
        )
        if result.returncode == 0:
            lines = result.stdout.strip().split('\n')
            test_count = 0
            for line in lines:
                if '::test_' in line:
                    test_count += 1
            print(f"✅ Pytest discovered {test_count} tests successfully")
            return True
        else:
            print(f"❌ Pytest configuration error: {result.stderr}")
            return False
    except Exception as e:
        print(f"❌ Error running pytest: {e}")
        return False


def check_project_structure():
    """Check if project structure is correct."""
    test_dir = Path(__file__).parent
    project_root = test_dir.parent.parent
    
    required_files = [
        test_dir / "requirements.txt",
        test_dir / "pytest.ini", 
        test_dir / "conftest.py",
        test_dir / "run_tests.py",
        project_root / "Cargo.toml",
        project_root / "src" / "main.rs",
    ]
    
    all_present = True
    for file_path in required_files:
        if file_path.exists():
            print(f"✅ {file_path.name} exists")
        else:
            print(f"❌ {file_path.name} missing")
            all_present = False
    
    return all_present


def check_server_can_build():
    """Check if RedisGate server can be built."""
    test_dir = Path(__file__).parent
    project_root = test_dir.parent.parent
    
    print("🔨 Checking if RedisGate server can be built...")
    try:
        result = subprocess.run(
            ["cargo", "check"],
            cwd=project_root,
            capture_output=True,
            text=True,
            timeout=60
        )
        if result.returncode == 0:
            print("✅ RedisGate server builds successfully")
            return True
        else:
            print(f"❌ Server build check failed: {result.stderr}")
            return False
    except subprocess.TimeoutExpired:
        print("❌ Server build check timed out")
        return False
    except Exception as e:
        print(f"❌ Error checking server build: {e}")
        return False


def main():
    """Main validation function."""
    print("🔍 Validating RedisGate Integration Test Setup")
    print("=" * 50)
    
    checks = [
        ("Python Version", check_python_version),
        ("Rust/Cargo", check_rust_cargo),
        ("Project Structure", check_project_structure),
        ("Python Dependencies", check_dependencies),
        ("Pytest Configuration", check_pytest_config),
        ("Server Build Check", check_server_can_build),
    ]
    
    passed = 0
    total = len(checks)
    
    for name, check_func in checks:
        print(f"\n📋 {name}:")
        if check_func():
            passed += 1
        else:
            print(f"   ⚠️  {name} check failed")
    
    print("\n" + "=" * 50)
    print(f"📊 Validation Results: {passed}/{total} checks passed")
    
    if passed == total:
        print("🎉 All checks passed! Integration test environment is ready.")
        print("\nNext steps:")
        print("  1. Start PostgreSQL database")
        print("  2. Run: python run_tests.py --mode basic")
        return True
    else:
        print("❌ Some checks failed. Please fix the issues above.")
        print("\nTo fix common issues:")
        print("  - Install dependencies: python run_tests.py --install-deps")
        print("  - Check Rust installation: rustup --version")
        print("  - Verify project structure")
        return False


if __name__ == "__main__":
    success = main()
    sys.exit(0 if success else 1)