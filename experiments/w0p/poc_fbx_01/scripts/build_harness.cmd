@echo off
setlocal
call "C:\Program Files\Microsoft Visual Studio\2022\Professional\VC\Auxiliary\Build\vcvars64.bat" || exit /b 1
cmake -S "%~dp0..\harness" -B "F:\NewResearch\rigforge_w0p_work\poc_fbx_01\build" -G "NMake Makefiles" -DCMAKE_BUILD_TYPE=RelWithDebInfo -DUFBX_ROOT="F:\NewResearch\rigforge_w0p_work\poc_core_01\deps\ufbx-0.23.0" || exit /b 1
cmake --build "F:\NewResearch\rigforge_w0p_work\poc_fbx_01\build" --config RelWithDebInfo || exit /b 1
