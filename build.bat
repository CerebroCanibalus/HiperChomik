@echo off
chcp 65001 > nul
echo ============================================
echo  CHOMIK HAMSTER - Build + Run
echo ============================================
echo.

:: Directorio base (donde está este script)
set "HAMSTER_DIR=%~dp0"

:: Verificar Rust
rustc --version > nul 2>&1
if errorlevel 1 (
    echo [ERROR] Rust no está instalado. Instalar desde: https://rustup.rs
    exit /b 1
)

:: ============================================
:: BUILD
:: ============================================
echo [BUILD] Compilando chomik-hamster...
cd /d "%HAMSTER_DIR%"
cargo build --release
if errorlevel 1 (
    echo [ERROR] Falló la compilación
    exit /b 1
)

:: ============================================
:: COPIAR SPRITES
:: ============================================
echo [COPY] Copiando sprites y anims.txt...
if exist "%HAMSTER_DIR%target\x86_64-pc-windows-gnu\release" (
    set "TARGET=%HAMSTER_DIR%target\x86_64-pc-windows-gnu\release"
) else (
    set "TARGET=%HAMSTER_DIR%target\release"
)
if exist "%TARGET%\sprites" rmdir /s /q "%TARGET%\sprites"
if exist "%HAMSTER_DIR%\sprites" (
    mkdir "%TARGET%\sprites" > nul 2>&1
    xcopy /e /i /q "%HAMSTER_DIR%\sprites\*" "%TARGET%\sprites\" > nul
)
copy /y "%HAMSTER_DIR%\anims.txt" "%TARGET%\anims.txt" > nul

:: ============================================
:: INFO
:: ============================================
echo.
echo ============================================
echo  BUILD COMPLETADO
echo ============================================
echo  Ejecutable: %TARGET%\chomik-hamster.exe
echo  RAM estimada: ~5 MB
echo  Sprites: %HAMSTER_DIR%sprites\
echo.
echo  Controles:
echo    Arrastrar     - Mover el hamster
echo    Doble click   - Abrir papelera
echo    Click derecho - Menu contextual
echo    Arrastrar archivo - Enviar a papelera
echo ============================================
echo.

:: Preguntar si ejecutar
choice /c SN /n /t 5 /d S /m "Ejecutar ahora? (S/n): " > nul 2>&1
if errorlevel 2 goto :end

echo [RUN] Iniciando chomik-hamster...
start "" "%TARGET%\chomik-hamster.exe"
goto :end

:end
cd /d "%HAMSTER_DIR%"
