cmake_minimum_required(VERSION 3.15)

project(greq
    VERSION 0.1.0
    DESCRIPTION "A CLI tool to search files using BM25 ranking - like grep but with linguistic analysis"
    LANGUAGES NONE  # We're using Rust, not C/C++
)

# Set default build type
if(NOT CMAKE_BUILD_TYPE)
    set(CMAKE_BUILD_TYPE Release)
endif()

# Find required tools
find_program(CARGO cargo REQUIRED)
find_program(RUSTC rustc REQUIRED)

# Set Cargo build profile based on CMAKE_BUILD_TYPE
if(CMAKE_BUILD_TYPE STREQUAL "Debug")
    set(CARGO_BUILD_TYPE "")
    set(CARGO_TARGET_DIR "${CMAKE_CURRENT_BINARY_DIR}/target/debug")
else()
    set(CARGO_BUILD_TYPE "--release")
    set(CARGO_TARGET_DIR "${CMAKE_CURRENT_BINARY_DIR}/target/release")
endif()

# Set the target architecture for multi-platform builds
if(CMAKE_SYSTEM_NAME STREQUAL "Windows")
    if(CMAKE_SYSTEM_PROCESSOR MATCHES "AMD64|x86_64")
        set(RUST_TARGET "x86_64-pc-windows-gnu")
    endif()
elseif(CMAKE_SYSTEM_NAME STREQUAL "Darwin")
    if(CMAKE_SYSTEM_PROCESSOR MATCHES "arm64")
        set(RUST_TARGET "aarch64-apple-darwin")
    elseif(CMAKE_SYSTEM_PROCESSOR MATCHES "x86_64")
        set(RUST_TARGET "x86_64-apple-darwin")
    endif()
elseif(CMAKE_SYSTEM_NAME STREQUAL "Linux")
    if(CMAKE_SYSTEM_PROCESSOR MATCHES "x86_64")
        set(RUST_TARGET "x86_64-unknown-linux-gnu")
    endif()
endif()

# Build the Rust binary
if(RUST_TARGET)
    set(CARGO_TARGET_FLAG "--target=${RUST_TARGET}")
    set(BINARY_PATH "${CMAKE_CURRENT_BINARY_DIR}/target/${RUST_TARGET}/$<IF:$<CONFIG:Debug>,debug,release>/greq")
else()
    set(CARGO_TARGET_FLAG "")
    set(BINARY_PATH "${CARGO_TARGET_DIR}/greq")
endif()

# Add executable suffix for Windows
if(CMAKE_SYSTEM_NAME STREQUAL "Windows")
    set(BINARY_PATH "${BINARY_PATH}.exe")
endif()

# Custom target to build the Rust project
add_custom_target(greq_build ALL
    COMMAND ${CARGO} build ${CARGO_BUILD_TYPE} ${CARGO_TARGET_FLAG}
    WORKING_DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR}
    COMMENT "Building greq with Cargo"
    VERBATIM
)

# Custom target for development builds (equivalent to cargo build)
add_custom_target(dev
    COMMAND ${CARGO} build
    WORKING_DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR}
    COMMENT "Building greq in development mode"
    VERBATIM
)

# Custom target for testing
add_custom_target(test
    COMMAND ${CARGO} test
    WORKING_DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR}
    COMMENT "Running Rust tests"
    VERBATIM
)

# Custom target for linting
add_custom_target(lint
    COMMAND ${CARGO} clippy -- -D warnings
    WORKING_DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR}
    COMMENT "Running Rust linter (clippy)"
    VERBATIM
)

# Custom target for formatting
add_custom_target(fmt
    COMMAND ${CARGO} fmt
    WORKING_DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR}
    COMMENT "Formatting Rust code"
    VERBATIM
)

# Custom target to clean build artifacts
add_custom_target(clean-rust
    COMMAND ${CARGO} clean
    WORKING_DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR}
    COMMENT "Cleaning Cargo build artifacts"
    VERBATIM
)

# Installation configuration
include(GNUInstallDirs)

# Install the binary
install(FILES ${BINARY_PATH}
    DESTINATION ${CMAKE_INSTALL_BINDIR}
    PERMISSIONS OWNER_READ OWNER_WRITE OWNER_EXECUTE GROUP_READ GROUP_EXECUTE WORLD_READ WORLD_EXECUTE
    RENAME greq${CMAKE_EXECUTABLE_SUFFIX}
)

# Install documentation
install(FILES README.md
    DESTINATION ${CMAKE_INSTALL_DOCDIR}
    OPTIONAL
)

install(FILES LICENSE
    DESTINATION ${CMAKE_INSTALL_DOCDIR}
    OPTIONAL
)

# CPack configuration for packaging
set(CPACK_PACKAGE_NAME "greq")
set(CPACK_PACKAGE_VERSION "${PROJECT_VERSION}")
set(CPACK_PACKAGE_DESCRIPTION_SUMMARY "${PROJECT_DESCRIPTION}")
set(CPACK_PACKAGE_VENDOR "Your Name")
set(CPACK_PACKAGE_CONTACT "your.email@example.com")

# Platform-specific packaging
if(CMAKE_SYSTEM_NAME STREQUAL "Linux")
    set(CPACK_GENERATOR "DEB;RPM;TGZ")
    set(CPACK_DEBIAN_PACKAGE_MAINTAINER "Your Name <your.email@example.com>")
    set(CPACK_DEBIAN_PACKAGE_SECTION "utils")
    set(CPACK_RPM_PACKAGE_GROUP "Applications/Text")
    set(CPACK_RPM_PACKAGE_LICENSE "MIT")
elseif(CMAKE_SYSTEM_NAME STREQUAL "Windows")
    set(CPACK_GENERATOR "NSIS;ZIP")
    set(CPACK_NSIS_DISPLAY_NAME "greq - BM25 File Search Tool")
    set(CPACK_NSIS_PACKAGE_NAME "greq")
elseif(CMAKE_SYSTEM_NAME STREQUAL "Darwin")
    set(CPACK_GENERATOR "DragNDrop;TGZ")
endif()

include(CPack)

# Print build information
message(STATUS "greq build configuration:")
message(STATUS "  Build type: ${CMAKE_BUILD_TYPE}")
message(STATUS "  Install prefix: ${CMAKE_INSTALL_PREFIX}")
message(STATUS "  Binary will be installed to: ${CMAKE_INSTALL_FULL_BINDIR}")
if(RUST_TARGET)
    message(STATUS "  Rust target: ${RUST_TARGET}")
endif()