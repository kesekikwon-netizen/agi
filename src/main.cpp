#include "agi/agi.hpp"

#include <iostream>

int main() {
    std::cout << agi::get_system_banner() << '\n';

    agi::Engine engine("AGI Primary Engine");
    std::cout << "Engine Name: " << engine.get_name() << '\n';
    std::cout << "Engine Version: " << engine.get_version() << '\n';
    std::cout << "Engine Status: " << (engine.is_ready() ? "Operational" : "Offline") << '\n';

    return 0;
}
