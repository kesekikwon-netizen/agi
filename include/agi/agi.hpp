#pragma once

#include <string>
#include <string_view>

namespace agi {

class [[nodiscard]] Engine {
public:
    explicit Engine(std::string name = "AGI Core");
    ~Engine() = default;

    // Rule of 5: Modern move semantics & RAII
    Engine(const Engine&) = default;
    Engine& operator=(const Engine&) = default;
    Engine(Engine&&) noexcept = default;
    Engine& operator=(Engine&&) noexcept = default;

    [[nodiscard]] std::string_view get_name() const noexcept;
    [[nodiscard]] std::string get_version() const;
    [[nodiscard]] bool is_ready() const noexcept;

private:
    std::string m_name;
    bool m_initialized{true};
};

[[nodiscard]] std::string get_system_banner();

} // namespace agi
