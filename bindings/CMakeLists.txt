cmake_minimum_required(VERSION 3.5.1)
cmake_policy(SET CMP0069 NEW)
set(CMAKE_POLICY_DEFAULT_CMP0069 NEW)

if(NOT DEFINED CMAKE_C_COMPILER)
    set(CMAKE_C_COMPILER "/usr/bin/clang")
    set(CMAKE_CXX_COMPILER "/usr/bin/clang++")
endif()

project(bergamot-sys)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
#set(CMAKE_INTERPROCEDURAL_OPTIMIZATION TRUE)
#set(CMAKE_CXX_FLAGS "${CMAKE_CXX_FLAGS} -flto")
#set(CMAKE_C_FLAGS "${CMAKE_C_FLAGS} -flto")
#set(CMAKE_CXX_FLAGS "${CMAKE_CXX_FLAGS} -ffat-lto-objects")
#set(CMAKE_C_FLAGS "${CMAKE_C_FLAGS} -ffat-lto-objects")

set(CMAKE_BUILD_TYPE Release CACHE STRING "" FORCE)
set(COMPILE_WASM OFF CACHE BOOL "" FORCE)
set(COMPILE_TESTS OFF CACHE BOOL "" FORCE)
set(COMPILE_UNIT_TESTS OFF CACHE BOOL "" FORCE)
set(COMPILE_LIBRARY_ONLY ON CACHE BOOL "" FORCE)
set(USE_STATIC_LIBS ON CACHE BOOL "" FORCE)
set(USE_SENTENCEPIECE ON CACHE BOOL "" FORCE)
set(USE_MKL OFF CACHE BOOL "" FORCE)
set(ENABLE_CACHE_STATS OFF CACHE BOOL "" FORCE)
set(SSPLIT_COMPILE_LIBRARY_ONLY ON CACHE BOOL "" FORCE)
set(SSPLIT_USE_INTERNAL_PCRE2 ON CACHE BOOL "" FORCE)
set(USE_RUY ON CACHE BOOL "" FORCE)
set(USE_RUY_SGEMM ON CACHE BOOL "" FORCE)
set(INTGEMM_DONT_BUILD_TESTS ON)
set(COMPILE_WITHOUT_EXCEPTIONS ON)
set(AUTO_CPU_DETECT OFF CACHE BOOL "" FORCE)

# marian-dev builds some other targets
set(COMPILE_LIBRARY_ONLY ON CACHE BOOL "" FORCE)
set(COMPILE_UNIT_TESTS OFF CACHE BOOL "" FORCE)

if(NOT DEFINED USE_THREADS)
    set(USE_THREADS OFF CACHE BOOL "" FORCE)
endif()

# Reproducible builds
#set(CMAKE_C_FLAGS "${CMAKE_C_FLAGS} -s")
#set(CMAKE_CXX_FLAGS "${CMAKE_CXX_FLAGS} -s")
set(CMAKE_C_FLAGS "${CMAKE_C_FLAGS} -fdebug-prefix-map=${CMAKE_CURRENT_SOURCE_DIR}=.")
set(CMAKE_CXX_FLAGS "${CMAKE_CXX_FLAGS} -fdebug-prefix-map=${CMAKE_CURRENT_SOURCE_DIR}=.")
set(CMAKE_C_FLAGS "${CMAKE_C_FLAGS} -Wno-builtin-macro-redefined")
set(CMAKE_CXX_FLAGS "${CMAKE_CXX_FLAGS} -Wno-builtin-macro-redefined")

# Size
set(CMAKE_C_FLAGS "${CMAKE_C_FLAGS} -ffunction-sections -fdata-sections -fvisibility=hidden")
set(CMAKE_CXX_FLAGS "${CMAKE_CXX_FLAGS} -fdata-sections -ffunction-sections -fvisibility=hidden -fvisibility-inlines-hidden")
set(CMAKE_EXE_LINKER_FLAGS "${CMAKE_EXE_LINKER_FLAGS} -Wl,--gc-sections -Wl,--strip-all")
set(CMAKE_SHARED_LINKER_FLAGS "${CMAKE_SHARED_LINKER_FLAGS} -Wl,--gc-sections -Wl,--strip-all")
set(CMAKE_STATIC_LINKER_FLAGS "${CMAKE_STATIC_LINKER_FLAGS}")
set(CMAKE_CXX_FLAGS "${CMAKE_CXX_FLAGS} -g0 -s")
set(CMAKE_C_FLAGS "${CMAKE_C_FLAGS} -g0 -s")

add_subdirectory(../bergamot-translator bergamot-translator)

add_library(bergamot_wrapper STATIC bergamot_wrapper.cpp)

target_link_libraries(bergamot_wrapper PRIVATE bergamot-translator)
