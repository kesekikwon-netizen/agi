cmake_minimum_required(VERSION 3.20)
project(agi VERSION 0.1.0 LANGUAGES CXX)

# Enforce Modern C++20
set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CXX_EXTENSIONS OFF)
set(CMAKE_EXPORT_COMPILE_COMMANDS ON)

# Compiler warnings configuration
function(agi_set_compiler_warnings target_name)
    if(MSVC)
        target_compile_options(${target_name} PRIVATE
            /W4
            /permissive-
            /utf-8
            /EHsc
        )
    else()
        target_compile_options(${target_name} PRIVATE
            -Wall
            -Wextra
            -Wpedantic
            -Wconversion
            -Wshadow
            -Wnon-virtual-dtor
        )
    endif()
endfunction()

# Core Library
add_library(agi_core
    src/agi.cpp
)
add_library(agi::core ALIAS agi_core)

target_include_directories(agi_core PUBLIC
    $<BUILD_INTERFACE:${CMAKE_CURRENT_SOURCE_DIR}/include>
    $<INSTALL_INTERFACE:include>
)
agi_set_compiler_warnings(agi_core)

# Main Executable
add_executable(agi_app
    src/main.cpp
)
target_link_libraries(agi_app PRIVATE agi::core)
agi_set_compiler_warnings(agi_app)
set_target_properties(agi_app PROPERTIES OUTPUT_NAME "agi")

# Testing
include(CTest)
if(BUILD_TESTING)
    enable_testing()
    add_subdirectory(tests)
endif()
