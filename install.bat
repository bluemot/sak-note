@echo off
chcp 65001 > nul
setlocal enabledelayedexpansion

echo ==========================================
echo SAK Editor - Environment Setup (Windows)
echo ==========================================
echo.

set "GREEN=[92m"
set "YELLOW=[93m"
set "RED=[91m"
set "NC=[0m"

:: Check if running as administrator
net session > nul 2>&1
if %errorLevel% neq 0 (
    echo %RED%Please run this script as Administrator%NC%
    pause
    exit /b 1
)

:: Check Node.js
echo Checking Node.js...
node --version > nul 2>&1
if %errorLevel% equ 0 (
    for /f "tokens=*" %%a in ('node --version') do echo %GREEN%✓%NC% Node.js already installed: %%a
) else (
    echo %YELLOW%Installing Node.js...%NC%
    echo Downloading Node.js installer...
    powershell -Command "Invoke-WebRequest -Uri 'https://nodejs.org/dist/v20.11.0/node-v20.11.0-x64.msi' -OutFile '%TEMP%\node-installer.msi'"
    echo Installing Node.js...
    msiexec /i "%TEMP%\node-installer.msi" /qn
    if %errorLevel% neq 0 (
        echo %RED%✗%NC% Failed to install Node.js. Please install manually from https://nodejs.org/
        pause
        exit /b 1
    )
    echo %GREEN%✓%NC% Node.js installed
)
echo.

:: Check Rust
echo Checking Rust...
cargo --version > nul 2>&1
if %errorLevel% equ 0 (
    for /f "tokens=*" %%a in ('cargo --version') do echo %GREEN%✓%NC% Rust already installed: %%a
) else (
    echo %YELLOW%Installing Rust...%NC%
    echo Downloading Rust installer...
    powershell -Command "Invoke-WebRequest -Uri 'https://win.rustup.rs/x86_64' -OutFile '%TEMP%\rustup-init.exe'"
    echo Installing Rust...
    "%TEMP%\rustup-init.exe" -y
    if %errorLevel% neq 0 (
        echo %RED%✗%NC% Failed to install Rust. Please install manually from https://rustup.rs/
        pause
        exit /b 1
    )
    echo %GREEN%✓%NC% Rust installed
)
echo.

:: Install WebView2 Runtime
echo Checking WebView2 Runtime...
reg query "HKLM\SOFTWARE\WOW6432Node\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-5D655E7B46A7}" > nul 2>&1
if %errorLevel% equ 0 (
    echo %GREEN%✓%NC% WebView2 Runtime already installed
) else (
    echo %YELLOW%Installing WebView2 Runtime...%NC%
    powershell -Command "Invoke-WebRequest -Uri 'https://go.microsoft.com/fwlink/p/?LinkId=2124703' -OutFile '%TEMP%\MicrosoftEdgeWebview2Setup.exe'"
    "%TEMP%\MicrosoftEdgeWebview2Setup.exe" /silent /install
    echo %GREEN%✓%NC% WebView2 Runtime installed
)
echo.

:: Install Visual C++ Build Tools
echo Checking Visual C++ Build Tools...
where cl > nul 2>&1
if %errorLevel% equ 0 (
    echo %GREEN%✓%NC% Visual C++ Build Tools already installed
) else (
    echo %YELLOW%Visual C++ Build Tools not found.%NC%
    echo Please install Visual Studio Build Tools from:
    echo https://visualstudio.microsoft.com/visual-cpp-build-tools/
    echo Make sure to select "Desktop development with C++"
    pause
)
echo.

:: Install project dependencies
echo Installing project dependencies...
echo Installing root npm dependencies...
call npm install
if %errorLevel% neq 0 (
    echo %RED%✗%NC% Failed to install root dependencies
    pause
    exit /b 1
)

echo Installing frontend npm dependencies...
cd src-frontend
call npm install
if %errorLevel% neq 0 (
    echo %RED%✗%NC% Failed to install frontend dependencies
    pause
    exit /b 1
)
cd ..

echo %GREEN%✓%NC% Project dependencies installed
echo.

:: Install Tauri CLI
echo Installing Tauri CLI...
call cargo install tauri-cli
if %errorLevel% neq 0 (
    echo %YELLOW%Tauri CLI may already be installed%NC%
)
echo %GREEN%✓%NC% Tauri CLI installed
echo.

echo ==========================================
echo %GREEN%Setup complete!%NC%
echo ==========================================
echo.
echo You can now run:
echo   npm run dev       - Start development server
echo   npm run tauri-dev - Start Tauri development
echo   npm run build     - Build release version
echo   npm test          - Run all tests
echo.
pause
