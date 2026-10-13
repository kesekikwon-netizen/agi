add_executable(test_agi
    test_agi.cpp
)

target_link_libraries(test_agi PRIVATE agi::core)
agi_set_compiler_warnings(test_agi)

add_test(NAME AGI_Core_UnitTests COMMAND test_agi)
