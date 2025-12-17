#!/bin/bash

# Test script for all Rust and Python libraries in the BLE plugin project
# Usage: ./test_all.sh [rust|python|all]

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

# Function to test Rust crates
test_rust_crates() {
    local skip_combinations="${1:-false}"
    print_status "Testing Rust crates..."
    
    # Find all Cargo.toml files and test each crate
    local rust_crates=(
        "lib_utils"
        "protocol"
        "protocol_io"
        "plugin_state_machine_std"
        "plugin_config"
    )
    
    local failed_crates=()
    local successful_crates=()
    
    for crate in "${rust_crates[@]}"; do
        if [ -d "$crate" ] && [ -f "$crate/Cargo.toml" ]; then
            print_status "Testing $crate..."
            
            cd "$crate"
            
            # Special handling for protocol crate to test all feature combinations
            if [ "$crate" = "protocol" ]; then
                local testing_success=true
                
                if [ "$skip_combinations" = "false" ]; then
                    print_status "Testing $crate with valid feature combinations..."
                    
                    # Test specific valid feature combinations manually
                    local valid_combinations=(
                        "protocol_buffer"
                        "quick_protocol_buffer" 
                        "protocol_buffer,bincode_serialization"
                        "quick_protocol_buffer,bincode_serialization"
                        "protocol_buffer,std"
                        "quick_protocol_buffer,std"
                        "protocol_buffer,std,bincode_serialization"
                        "quick_protocol_buffer,std,bincode_serialization"
                    )
                    
                    local combo_success=true
                    for combo in "${valid_combinations[@]}"; do
                        # Run tests with the same feature combination
                        if cargo test --no-default-features --features "$combo" --quiet >/dev/null 2>&1; then
                            continue
                        else
                            combo_success=false
                            break
                        fi
                    done
                    
                    if $combo_success; then
                        print_success "$crate valid feature combinations passed"
                    else
                        print_error "$crate some feature combinations failed"
                        testing_success=false
                    fi
                else
                    print_status "Skipping feature combinations for $crate (--skip-combinations flag set)"
                fi
                
                # Test default features - build first, then test
                if cargo test --quiet >/dev/null 2>&1; then
                    print_success "$crate default features passed"
                else
                    print_error "$crate default features failed"
                    testing_success=false
                fi
                
                if $testing_success; then
                    successful_crates+=("$crate")
                else
                    failed_crates+=("$crate (feature testing failed)")
                fi
            else
                # Standard testing for other crates
                # Check if tests exist
                if [ -d "tests" ] || grep -q "\[\[test\]\]" Cargo.toml 2>/dev/null || find src -name "*.rs" -exec grep -l "#\[test\]" {} \; | head -1 >/dev/null 2>&1; then
                    if cargo test --quiet >/dev/null 2>&1; then
                        print_success "$crate tests passed"
                        successful_crates+=("$crate")
                    else
                        print_error "$crate tests failed"
                        failed_crates+=("$crate")
                    fi
                else
                    # If no tests, just check if it compiles
                    if cargo check --quiet 2>/dev/null; then
                        print_warning "$crate has no tests, but compiles successfully"
                        successful_crates+=("$crate (compile only)")
                    else
                        print_error "$crate failed to compile"
                        failed_crates+=("$crate (compile failed)")
                    fi
                fi
            fi
            
            cd ..
        else
            print_warning "Skipping $crate - directory or Cargo.toml not found"
        fi
    done
    
    # Summary for Rust
    print_status "Rust Testing Summary:"
    if [ ${#successful_crates[@]} -gt 0 ]; then
        print_success "Successful crates (${#successful_crates[@]}):"
        for crate in "${successful_crates[@]}"; do
            echo "  ✓ $crate"
        done
    fi
    
    if [ ${#failed_crates[@]} -gt 0 ]; then
        print_error "Failed crates (${#failed_crates[@]}):"
        for crate in "${failed_crates[@]}"; do
            echo "  ✗ $crate"
        done
        return 1
    fi
    
    return 0
}

# Function to check compilation for selected crates
compile_selected_crates() {
    print_status "Checking compilation for selected crates..."
    
    # Additional crates to check for compilation only
    local compile_crates=(
        "host-std"
        "host-no-std"
        "plugin-std"
        "plugin-no-std"
        "plugin-host-std"
        "plugin-echo-example"
    )
    
    local failed_crates=()
    local successful_crates=()
    
    for crate in "${compile_crates[@]}"; do
        if [ -d "$crate" ] && [ -f "$crate/Cargo.toml" ]; then
            print_status "Checking compilation for $crate..."
            
            cd "$crate"
            
            if cargo check --quiet 2>/dev/null; then
                print_success "$crate compiles successfully"
                successful_crates+=("$crate")
            else
                print_error "$crate failed to compile"
                failed_crates+=("$crate")
            fi
            
            cd ..
        else
            print_warning "Skipping $crate - directory or Cargo.toml not found"
        fi
    done
    
    # Summary for compilation check
    print_status "Compilation Check Summary:"
    if [ ${#successful_crates[@]} -gt 0 ]; then
        print_success "Successfully compiled crates (${#successful_crates[@]}):"
        for crate in "${successful_crates[@]}"; do
            echo "  ✓ $crate"
        done
    fi
    
    if [ ${#failed_crates[@]} -gt 0 ]; then
        print_error "Failed to compile crates (${#failed_crates[@]}):"
        for crate in "${failed_crates[@]}"; do
            echo "  ✗ $crate"
        done
        return 1
    fi
    
    return 0
}

# Function to test Python packages
test_python_packages() {
    print_status "Testing Python packages..."
    
    local python_dirs=(
        "pc/python"
    )
    
    local failed_packages=()
    local successful_packages=()
    
    for pkg_dir in "${python_dirs[@]}"; do
        if [ -d "$pkg_dir" ]; then
            print_status "Testing Python package in $pkg_dir..."
            
            cd "$pkg_dir"
            
            # Check if virtual environment exists
            if [ -f "pyvenv.cfg" ]; then
                # Activate virtual environment
                if [ -f "bin/activate" ]; then
                    source bin/activate
                elif [ -f "Scripts/activate" ]; then
                    source Scripts/activate
                fi
            fi
            
            # Check if pytest is available and tests exist
            if [ -f "pytest.ini" ] || [ -d "tests" ]; then
                if command -v pytest >/dev/null 2>&1; then
                    # Only test the tests/ directory to avoid virtual environment packages
                    if pytest tests/ --quiet >/dev/null 2>&1; then
                        print_success "$pkg_dir tests passed"
                        successful_packages+=("$pkg_dir")
                    else
                        print_error "$pkg_dir tests failed"
                        failed_packages+=("$pkg_dir")
                    fi
                else
                    print_warning "$pkg_dir has tests but pytest not available"
                    # Try with python -m pytest
                    if python -m pytest tests/ --quiet >/dev/null 2>&1; then
                        print_success "$pkg_dir tests passed (using python -m pytest)"
                        successful_packages+=("$pkg_dir")
                    else
                        print_error "$pkg_dir tests failed"
                        failed_packages+=("$pkg_dir")
                    fi
                fi
            else
                # Try basic import test
                if python -c "import plugin_host" 2>/dev/null; then
                    print_warning "$pkg_dir has no tests, but imports successfully"
                    successful_packages+=("$pkg_dir (import only)")
                else
                    print_error "$pkg_dir failed basic import test"
                    failed_packages+=("$pkg_dir (import failed)")
                fi
            fi
            
            # Deactivate virtual environment if it was activated
            if [ -n "$VIRTUAL_ENV" ]; then
                deactivate 2>/dev/null || true
            fi
            
            cd - >/dev/null
        else
            print_warning "Skipping $pkg_dir - directory not found"
        fi
    done
    
    # Summary for Python
    print_status "Python Testing Summary:"
    if [ ${#successful_packages[@]} -gt 0 ]; then
        print_success "Successful packages (${#successful_packages[@]}):"
        for pkg in "${successful_packages[@]}"; do
            echo "  ✓ $pkg"
        done
    fi
    
    if [ ${#failed_packages[@]} -gt 0 ]; then
        print_error "Failed packages (${#failed_packages[@]}):"
        for pkg in "${failed_packages[@]}"; do
            echo "  ✗ $pkg"
        done
        return 1
    fi
    
    return 0
}

# Main execution
main() {
    local test_type="${1:-all}"
    local skip_combinations=false
    
    # Check for --skip-combinations or -s flag
    for arg in "$@"; do
        if [ "$arg" = "--skip-combinations" ] || [ "$arg" = "-s" ]; then
            skip_combinations=true
        fi
    done
    
    local rust_success=0
    local python_success=0
    local compile_success=0
    
    print_status "Starting test suite for BLE Plugin project"
    print_status "Test type: $test_type"
    echo
    
    case "$test_type" in
        "rust")
            test_rust_crates "$skip_combinations"
            rust_success=$?
            ;;
        "python")
            test_python_packages
            python_success=$?
            ;;
        "compile")
            compile_selected_crates
            compile_success=$?
            ;;
        "all"|*)
            test_rust_crates "$skip_combinations"
            rust_success=$?
            echo
            compile_selected_crates
            compile_success=$?
            echo
            test_python_packages  
            python_success=$?
            ;;
    esac
    
    echo
    print_status "Overall Summary:"
    
    if [ "$test_type" = "all" ] || [ "$test_type" = "rust" ]; then
        if [ $rust_success -eq 0 ]; then
            print_success "All Rust crates passed"
        else
            print_error "Some Rust crates failed"
        fi
    fi
    
    if [ "$test_type" = "all" ] || [ "$test_type" = "compile" ]; then
        if [ $compile_success -eq 0 ]; then
            print_success "All selected crates compiled successfully"
        else
            print_error "Some selected crates failed to compile"
        fi
    fi
    
    if [ "$test_type" = "all" ] || [ "$test_type" = "python" ]; then
        if [ $python_success -eq 0 ]; then
            print_success "All Python packages passed"
        else
            print_error "Some Python packages failed"
        fi
    fi
    
    # Exit with error if any tests failed
    if [ $rust_success -ne 0 ] || [ $python_success -ne 0 ] || [ $compile_success -ne 0 ]; then
        exit 1
    fi
    
    print_success "All tests completed successfully!"
}

# Show usage if help requested
if [ "$1" = "--help" ] || [ "$1" = "-h" ]; then
    echo "Usage: $0 [rust|python|compile|all] [-s|--skip-combinations]"
    echo
    echo "Test all Rust and Python libraries in the BLE plugin project"
    echo
    echo "Options:"
    echo "  rust    - Test only Rust crates"
    echo "  python  - Test only Python packages"
    echo "  compile - Check compilation for selected crates"
    echo "  all     - Test Rust crates, check compilation, and test Python packages (default)"
    echo
    echo "Flags:"
    echo "  -s, --skip-combinations - Skip testing all feature combinations for protocol crate"
    echo "  -h, --help              - Show this help message"
    exit 0
fi

main "$@"