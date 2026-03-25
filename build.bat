@echo off
chcp 65001 > nul
setlocal enabledelayedexpansion

echo ==========================================
echo SAK Editor - Release Build (Windows)
echo ==========================================
echo.

set "GREEN=[92m"
set "YELLOW=[93m"
set "RED=[91m"
set "BLUE=[94m"
set "NC=[0m"

:: Get script directory
set "SCRIPT_DIR=%~dp0"
cd /d "%SCRIPT_DIR%"

:: Check prerequisites
echo Checking prerequisites...

node --version > nul 2>&1
if %errorLevel% neq 0 (
    echo %RED%Node.js not found. Please run install.bat first%NC%
    pause
    exit /b 1
)

cargo --version > nul 2>&1
if %errorLevel% neq 0 (
    echo %RED%Rust not found. Please run install.bat first%NC%
    pause
    exit /b 1
)

echo %GREEN%✓%NC% Prerequisites met
echo.

:: Clean previous builds
echo Cleaning previous builds...
if exist "src-frontend\dist" rmdir /s /q "src-frontend\dist"
if exist "src-tauri\target\release" rmdir /s /q "src-tauri\target\release"
echo %GREEN%✓%NC% Clean complete
echo.

:: Build frontend
echo %BLUE%Building frontend...%NC%
cd src-frontend
call npm run build
if %errorLevel% neq 0 (
    echo %RED%✗%NC% Frontend build failed
    pause
    exit /b 1
)
echo %GREEN%✓%NC% Frontend build complete
echo.

:: Build Tauri release
echo %BLUE%Building Tauri release...%NC%
cd ..\src-tauri

cargo tauri build
if %errorLevel% neq 0 (
    echo %RED%✗%NC% Tauri build failed
    pause
    exit /b 1
)
echo %GREEN%✓%NC% Tauri build complete
echo.

:: Find the built executable
echo Locating built executable...

set "RELEASE_DIR=target\release"
set "BUNDLE_DIR=%RELEASE_DIR%\bundle"

:: Check for MSI installer
for /r "%BUNDLE_DIR%" %%f in (*.msi) do (
    echo %GREEN%✓%NC% MSI installer: %%f
)

:: Check for NSIS installer
for /r "%BUNDLE_DIR%" %%f in (*.exe) do (
    echo %GREEN%✓%NC% Installer: %%f
)

:: Standalone executable
if exist "%RELEASE_DIR%\sak-editor.exe" (
    echo %GREEN%✓%NC% Standalone executable: %RELEASE_DIR%\sak-editor.exe
    echo.
    echo To run: %RELEASE_DIR%\sak-editor.exe
)

echo.
echo ==========================================
echo %GREEN%Build complete!%NC%
echo ==========================================
echo.
echo Release artifacts are in: src-tauri\target\release\bundle\
echo.
pause
