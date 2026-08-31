# RigForge POC-CORE-01 C++ research build
# RESEARCH ONLY / W0-P / NON-PRODUCTION
cmake_minimum_required(VERSION 3.16)
project(rigforge_poc_core_cpp LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

if(NOT UFBX_ROOT)
  message(FATAL_ERROR "UFBX_ROOT must point at ufbx v0.23.0 sources (ufbx.h)")
endif()
if(NOT UFBX_LIB_DIR)
  message(FATAL_ERROR "UFBX_LIB_DIR must point at the prebuilt ufbx.lib directory")
endif()

add_library(rigforge_poc_core_cpp SHARED poc_core.cpp)
target_include_directories(rigforge_poc_core_cpp PRIVATE
  ${CMAKE_CURRENT_SOURCE_DIR}/../abi
  ${UFBX_ROOT})
target_compile_definitions(rigforge_poc_core_cpp PRIVATE RF_POC_EXPORT)
target_link_directories(rigforge_poc_core_cpp PRIVATE ${UFBX_LIB_DIR})
target_link_libraries(rigforge_poc_core_cpp PRIVATE ufbx)
set_target_properties(rigforge_poc_core_cpp PROPERTIES OUTPUT_NAME "rigforge_poc_core_cpp")

if(MSVC)
  target_compile_options(rigforge_poc_core_cpp PRIVATE /W3 /EHsc /utf-8)
endif()

add_executable(poc_core_worker_cpp worker_main.cpp)
target_include_directories(poc_core_worker_cpp PRIVATE
  ${CMAKE_CURRENT_SOURCE_DIR}/../abi)
target_link_libraries(poc_core_worker_cpp PRIVATE rigforge_poc_core_cpp)
if(MSVC)
  target_compile_options(poc_core_worker_cpp PRIVATE /W3 /EHsc /utf-8)
endif()
