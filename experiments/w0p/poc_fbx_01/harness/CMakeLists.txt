cmake_minimum_required(VERSION 3.16)
project(poc_fbx_01_inspect C)

# RESEARCH ONLY / W0-P / NON-PRODUCTION
# C99 is the test harness language only.
# This is not evidence selecting the RigForge Core language.

set(CMAKE_C_STANDARD 99)
set(CMAKE_C_STANDARD_REQUIRED ON)

if(NOT UFBX_ROOT)
  message(FATAL_ERROR "UFBX_ROOT must point at the pinned ufbx v0.23.0 extract (outside the repo).")
endif()

add_executable(inspect_fbx inspect_fbx.c "${UFBX_ROOT}/ufbx.c")
target_include_directories(inspect_fbx PRIVATE "${UFBX_ROOT}")

if(MSVC)
  target_compile_definitions(inspect_fbx PRIVATE _CRT_SECURE_NO_WARNINGS)
  target_link_libraries(inspect_fbx PRIVATE psapi)
endif()
