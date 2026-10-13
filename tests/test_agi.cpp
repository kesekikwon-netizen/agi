#include "agi/agi.hpp"

#include <cassert>
#include <iostream>

void test_engine_initialization() {
    agi::Engine engine("TestEngine");
    assert(engine.get_name() == "TestEngine");
    assert(engine.get_version() == "0.1.0");
    assert(engine.is_ready() == true);
    std::cout << "[PASS] test_engine_initialization\n";
}

void test_engine_move_semantics() {
    agi::Engine e1("MovableEngine");
    agi::Engine e2 = std::move(e1);
    assert(e2.get_name() == "MovableEngine");
    assert(e2.is_ready() == true);
    std::cout << "[PASS] test_engine_move_semantics\n";
}

void test_system_banner() {
    const std::string banner = agi::get_system_banner();
    assert(!banner.empty());
    assert(banner.find("AGI") != std::string::npos);
    std::cout << "[PASS] test_system_banner\n";
}

int main() {
    std::cout << "Running AGI Core Test Suite...\n";
    test_engine_initialization();
    test_engine_move_semantics();
    test_system_banner();
    std::cout << "All unit tests completed successfully.\n";
    return 0;
}
