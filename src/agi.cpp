#include "agi/agi.hpp"

#include <format>
#include <utility>

namespace agi {

Engine::Engine(std::string name)
    : m_name(std::move(name)) {}

std::string_view Engine::get_name() const noexcept {
    return m_name;
}

std::string Engine::get_version() const {
    return "0.1.0";
}

bool Engine::is_ready() const noexcept {
    return m_initialized;
}

std::string get_system_banner() {
    return "AGI Native Engine v0.1.0 (C++20 Native Engine)";
}

} // namespace agi
