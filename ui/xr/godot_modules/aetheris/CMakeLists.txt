cmake_minimum_required(VERSION 3.16)
project(aetheris)

# Set C++ standard
set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

# Find required packages
find_package(PkgConfig REQUIRED)
find_package(Threads REQUIRED)

# Find Godot headers
find_path(GODOT_HEADERS_DIR
    NAMES godot_cpp/godot.hpp
    PATHS
        ${CMAKE_CURRENT_SOURCE_DIR}/third_party/godot-cpp
        ${CMAKE_CURRENT_SOURCE_DIR}/../third_party/godot-cpp
        /usr/local/include
        /usr/include
    PATH_SUFFIXES godot-cpp
)

if(NOT GODOT_HEADERS_DIR)
    message(FATAL_ERROR "Godot headers not found. Please set GODOT_HEADERS_DIR or install godot-cpp")
endif()

# Find OpenXR (optional, for XR support)
find_path(OPENXR_INCLUDE_DIR
    NAMES openxr/openxr.h
    PATHS
        ${CMAKE_CURRENT_SOURCE_DIR}/third_party/OpenXR-SDK
        /usr/local/include
        /usr/include
    PATH_SUFFIXES OpenXR-SDK/include
)

# Find gRPC
find_package(gRPC REQUIRED)
find_package(Protobuf REQUIRED)

# Include directories
include_directories(${GODOT_HEADERS_DIR})
include_directories(${CMAKE_CURRENT_SOURCE_DIR}/include)

if(OPENXR_INCLUDE_DIR)
    include_directories(${OPENXR_INCLUDE_DIR})
    add_definitions(-DOPENXR_ENABLED)
    message(STATUS "OpenXR support enabled")
else()
    message(STATUS "OpenXR support disabled (headers not found)")
endif()

# Source files
set(SOURCES
    src/aetheris.cpp
    src/aetheris_scene.cpp
    src/aetheris_avatar.cpp
    src/aetheris_bus.cpp
)

# Header files
set(HEADERS
    include/aetheris_scene.hpp
    include/aetheris_avatar.hpp
    include/aetheris_bus.hpp
)

# Create the library
add_library(aetheris SHARED ${SOURCES} ${HEADERS})

# Link libraries
target_link_libraries(aetheris
    gRPC::grpc++
    protobuf::libprotobuf
    Threads::Threads
)

# Compiler-specific options
if(MSVC)
    target_compile_options(aetheris PRIVATE /W4)
else()
    target_compile_options(aetheris PRIVATE -Wall -Wextra -Wpedantic)
endif()

# Set output directory
set_target_properties(aetheris PROPERTIES
    LIBRARY_OUTPUT_DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR}/bin
    RUNTIME_OUTPUT_DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR}/bin
)

# Create the GDExtension configuration file
configure_file(
    ${CMAKE_CURRENT_SOURCE_DIR}/aetheris.gdextension.in
    ${CMAKE_CURRENT_SOURCE_DIR}/bin/aetheris.gdextension
    @ONLY
)

# Install targets
install(TARGETS aetheris
    LIBRARY DESTINATION lib
    RUNTIME DESTINATION bin
)

# Copy the GDExtension file to the output directory
install(FILES ${CMAKE_CURRENT_SOURCE_DIR}/bin/aetheris.gdextension
    DESTINATION bin
)

# Print configuration summary
message(STATUS "=== Aetheris GDExtension Configuration ===")
message(STATUS "Godot headers: ${GODOT_HEADERS_DIR}")
message(STATUS "OpenXR support: ${OPENXR_INCLUDE_DIR}")
message(STATUS "gRPC: ${gRPC_FOUND}")
message(STATUS "Protobuf: ${Protobuf_FOUND}")
message(STATUS "Output directory: ${CMAKE_CURRENT_SOURCE_DIR}/bin")
message(STATUS "==========================================")
