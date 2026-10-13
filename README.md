# AGI

[![CI](https://github.com/kesekikwon-netizen/agi/actions/workflows/ci.yml/badge.svg)](https://github.com/kesekikwon-netizen/agi/actions/workflows/ci.yml)

High-Performance Modern C++20 Core Architecture & System Framework.

## 🚀 Features

- **Modern C++20 Architecture**: Strict RAII, safe move semantics, and zero raw owning pointers.
- **Cross-Platform Support**: macOS (AppleClang), Linux (GCC/Clang), and Windows (MSVC).
- **Automated CI/CD**: GitHub Actions workflow automatically builds, tests, and validates code on every push and pull request.
- **Built-in Unit Testing**: Comprehensive CTest testing suite with automated sanity checks.

## 🛠️ Requirements

- **C++ Compiler**: Supporting C++20 standard (AppleClang 15+, GCC 12+, Clang 14+, MSVC 2022+)
- **CMake**: 3.20 or newer
- **Build System**: Ninja or Make

## 📦 Build & Test

### Local Build

```bash
# Configure
cmake -B build -S . -DCMAKE_BUILD_TYPE=Release

# Build
cmake --build build --config Release

# Run tests
ctest --test-dir build --output-on-failure

# Execute application
./build/agi
```

## 📂 Project Structure

```
agi/
├── .github/
│   └── workflows/
│       └── ci.yml            # Multi-platform GitHub Actions CI
├── include/
│   └── agi/
│       └── agi.hpp           # Core C++20 Header
├── src/
│   ├── agi.cpp               # Core Implementation
│   └── main.cpp              # Application Entry Point
├── tests/
│   ├── CMakeLists.txt        # Test Configuration
│   └── test_agi.cpp          # Unit Tests
├── .gitignore                # Git Ignore Patterns
├── CMakeLists.txt            # Root CMake Build Configuration
└── README.md                 # Project Documentation
```

## 📄 License

MIT License (or as specified by author).
