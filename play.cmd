@echo off
rem Starts the viewer built last, without cargo (about 0.7 s sooner than
rem `cargo play`). Same options, e.g.  play --world DeadCity
rem After the code changes, run `cargo play` once so it is built again:
rem this script never builds.
cd /d "%~dp0"
if not exist "target\release\kq8-viewer.exe" (
    echo The viewer is not built yet: run "cargo play" once first.
    exit /b 1
)
"target\release\kq8-viewer.exe" %*
