cmake_minimum_required(VERSION 3.20)
project(oxidized-cell-cpp LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_EXPORT_COMPILE_COMMANDS ON)

# Platform-specific
if(CMAKE_SYSTEM_PROCESSOR MATCHES "x86_64|AMD64")
    set(ARCH_X64 TRUE)
    add_compile_definitions(ARCH_X64)
    if(MSVC)
        add_compile_options(/arch:AVX2)
    else()
        add_compile_options(-mavx2 -mbmi2)
    endif()
elseif(CMAKE_SYSTEM_PROCESSOR MATCHES "aarch64|ARM64")
    set(ARCH_ARM64 TRUE)
    add_compile_definitions(ARCH_ARM64)
endif()

# Library
add_library(oc_cpp STATIC
    src/ffi.cpp
    src/ppu_jit.cpp
    src/spu_jit.cpp
    src/rsx_shaders.cpp
    src/atomics.cpp
    src/dma.cpp
)

if(ARCH_X64)
    target_sources(oc_cpp PRIVATE
        src/simd_avx.cpp
    )
endif()

target_include_directories(oc_cpp PUBLIC include)

# Install for Rust linking
install(TARGETS oc_cpp
    ARCHIVE DESTINATION lib
    LIBRARY DESTINATION lib
)
