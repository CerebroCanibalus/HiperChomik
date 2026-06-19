# Chomik Hamster — AGENTS.md

## Proyecto
Desktop pet (hámster) para Windows 10/11 en Rust. Come archivos → Papelera de Reciclaje.

## Stack
- **Lenguaje:** Rust 2021
- **Windowing:** winit 0.30 (event loop + window creation)
- **Sprites:** image crate (carga PNG → BGRA premultiplied)
- **Rendering:** UpdateLayeredWindow (per-pixel alpha via GDI)
- **Win32 FFI:** raw extern "system" calls (user32 + gdi32)

## Arquitectura de ventana (CRÍTICO)

### ❌ NO usar
- `with_transparent(true)` de winit — **roto en Win8+** (llama `SetLayeredWindowAttributes` que bloquea `UpdateLayeredWindow` y causa click-through bug confirmado por Microsoft)
- `SetWindowSubclass` / `GWLP_WNDPROC` — no reciben `WM_NCHITTEST` en ventanas `UpdateLayeredWindow` en Win10

### ✅ Cómo funciona
1. winit crea ventana NORMAL (`with_decorations(false)`, SIN `with_transparent`)
2. `SetWindowLongPtrW(GWL_EXSTYLE)` añade `WS_EX_LAYERED | WS_EX_TOOLWINDOW | WS_EX_ACCEPTFILES | WS_EX_NOACTIVATE`
3. `SetWindowPos(SWP_FRAMECHANGED)` para registrar el estilo correctamente
4. `UpdateLayeredWindow` para render per-pixel alpha
5. `SetWindowRgn` para hit-testing (región desde alpha > 10)

### Estilos de ventana
- `WS_POPUP` (de winit `with_decorations(false)`)
- `WS_EX_LAYERED` — per-pixel alpha via UpdateLayeredWindow
- `WS_EX_TOOLWINDOW` — no aparece en taskbar
- `WS_EX_ACCEPTFILES` — drag & drop
- `WS_EX_NOACTIVATE` (0x08000000) — no roba foco

### Drag sin lag
`CursorMoved` da coordenadas relativas a la ventana. Al mover la ventana, la siguiente posición es relativa a la NUEVA posición. **Solución:** calcular la posición absoluta del cursor como `window_position + cursor_in_window`, luego `new_window = absolute_cursor - original_offset`. Ver `CursorMoved` handler en `window_event`.

## Animaciones
- Módulo separado: `src/animation.rs` — state machine con 3 estados:
  - `Idle`: idle aleatorio (filtra AnimIdleStart*/Loop*/Finish* y frames vacíos)
  - `Sequence`: secuencia one-shot (ej: DragFileStart → Processing → Finish → vuelve a Idle)
  - `Loop`: single animación en loop infinito
- Métodos: `play_idle()`, `play_seq(&[...])`, `play_loop(name)`, `current_sprite()`, `update(dt)`
- Constructor llama `pick_random_idle()` para empezar con idle válido
- `is_busy()`: true si hay secuencia o loop activo
- Formato: `anims.txt` con nombre, frame count, luego frames (archivo duration_ms)
- Nombres de animaciones disponibles:
  - `AnimMainIdle`, `AnimIdle1`-`AnimIdle11`, `AnimIdleStart1`-`AnimIdleFinish3`
  - `AnimScreenshotStart/Processing/Cancel/Finish`
  - `AnimTypingStart/Typing/TypingStop`
  - `AnimDragFileStart/Processing/Cancel/Finish`
  - `AnimSpeakingStart/Speaking/SpeakingFinish`
  - `AnimCharacterEnter/Leave/MoveStart/Moving/MoveFinish`
  - `AnimMusicStart`

## File Eater
- `WS_EX_ACCEPTFILES` + `WindowEvent::DroppedFile` → acumula en `DROPPED_FILES` (Vec, no reemplaza)
- `Mutex<Vec<String>>` — se acumulan múltiples archivos en mismos drop
- Normal → `send_to_bin()`: `SHFileOperationW` con `FO_DELETE` + `FOF_ALLOWUNDO` → papelera
- Turbo → `turbo_delete()`: `std::fs::remove_file`/`remove_dir_all` permanente (paralelo con `std::thread::scope`)
- Confirmación: `confirm_turbo()` con mensaje aleatorio de `quotes.txt` (o defaults integrados)
- `config.ini` guarda `trash_enabled` + `turbo_enabled`
- Toggleable desde menú contextual (Trash Enabled + Turbo Eater)

## Build
```powershell
cd D:\Mis Juegos\HyperChomik\chomik-hamster
cargo build
Copy-Item sprites\* target\x86_64-pc-windows-gnu\debug\sprites\
Copy-Item quotes.txt target\x86_64-pc-windows-gnu\debug\
.\target\x86_64-pc-windows-gnu\debug\chomik-hamster.exe
```
⚠️ **GNU toolchain** → exe en `target\x86_64-pc-windows-gnu\debug\`, NO `target\debug\`

## Sprites
- `sprites/` junto al .exe o en `D:\Mis Juegos\HyperChomik\chomik-hamster\sprites\`
- Copiar al build dir: `Copy-Item sprites\*.png target\x86_64-pc-windows-gnu\debug\sprites\`
- PNG con canal alpha (carga → `image::load_from_memory` → premultiply BGRA)

## Config
- `%APPDATA%\chomik-hamster\config.ini`
- Guarda: posición x/y, trash_enabled, **turbo_enabled**, start_with_windows
- Startup via `HKCU\Software\Microsoft\Windows\CurrentVersion\Run\ChomikHamster`

## Menú contextual (right-click)
- `CreatePopupMenu` + `TrackPopupMenu`
- Opciones: Open Trash, Empty Trash, Trash Enabled toggle, Turbo Eater toggle, Start with Windows toggle, Quit

## Quotes
- `quotes.txt` junto al .exe — un mensaje por línea, se lee al confirmar turbo delete
- Si no existe el archivo, usa defaults hardcodeados en `main.rs`
- Se muestra aleatoriamente (por `SystemTime::now().as_nanos() % len`)
- Usuario puede añadir/quitar líneas libremente
- 27 quotes por defecto (español + polaco, tono soez/cringe/humor negro)

## Sprite files
- `sprites/` junto al exe — ~1620 PNGs (hamster_0000.png a hamster_1639.png)
- Copiar al build dir: `Copy-Item sprites\* target\x86_64-pc-windows-gnu\debug\sprites\`
- Se cargan lazy (`HashMap::new()` + `load_sprite()`)

## Desktop integration (PENDIENTE)
- Poner ventana detrás de iconos del escritorio (Progman/WorkerW)
- `SetParent(Progman)` deshabilitado por ahora (rompe clicks en algunos sistemas)
- Investigar técnica de DesktopGoose/Shimeji

## Event Loop (animación con timer dinámico)
- **NO** `ControlFlow::Poll` — causa 98% CPU en Windows
- `ControlFlow::Wait` + `SetTimer` en tray window para wake periódico
- `WM_TIMER` → `ANIM_WAKE = true` → `about_to_wait` → `request_redraw()`
- Timer dinámico: en cada `RedrawRequested`, setea próximo timer a `remaining_frame_ms()`
  - Frames largos (1000ms): timer una vez por segundo
  - Frames cortos (40ms): timer cada 40ms
- Mínimo 16ms, máximo 5000ms
- `wake_render()`: fuerza `request_redraw()` inmediato tras interacción del usuario
- DIB cache (`DibCache`): reusa DIBSection del mismo tamaño
- Region cache: reusa `HRGN` del mismo frame; `DeleteObject` antes de sobrescribir

## Hover + Beg Animation
- `CursorEntered` → `AnimBegStart` (0820→0832, anticipación) → `AnimBegLoop` (0833→0859, bucle)
- `CursorLeft` → `AnimBegEnd` (0832→0820, reversed) → idle
- `HoveredFile` → mismo flujo (archivo arrastrado encima)
- `HoveredFileCancelled` → limpia si no hay hover normal
- `begging: bool` + `hovered: bool` controlan transiciones en `RedrawRequested`
- Animaciones definidas en `anims.txt` como `AnimBegStart/Loop/End`

## Audio Detection (Windows Core Audio)
- `src/audio.rs`: usa raw COM vtable para `IAudioMeterInformation`
- IID: `{C02216F6-8C67-4B5B-9D00-D008E73E0064}`
- `IMMDevice::Activate` vía vtable (slot 3, después de IUnknown) con `CLSCTX_ALL`
- `GetPeakValue` vía vtable (slot 3 de IAudioMeterInformation)
- Peak > 0.001 → hay audio activo
- Poll cada 2s en `about_to_wait`
- `music_playing: bool` + transición: `AnimMusicStart` → `AnimMusicLoop` → `AnimMusicFinish`
- Interacción usuario (hover/click) tiene prioridad sobre música
- **CUIDADO:** `IAudioMeterInformation` no existe en `windows` crate v0.58. Implementación manual via raw COM.
- `windows` crate v0.62+ requiere `dlltool.exe` → incompatible con GNU toolchain. Mantener v0.58.

## Peso idle animations
- `pick_random_idle()`: 80% `AnimMainIdle`, 20% otra idle aleatoria
- `stop_loop()` + `current_loop()` métodos en `HamsterAnims`

## Bugs conocidos
1. No hay animación de caminar/desplazarse automática (solo idle flotante)
2. Sin integración de escritorio (detrás de iconos DesktopGoose-style)
3. Si el event loop se queda sin redraws, reiniciar app
4. Turbo delete seq (no paralelo) — pendiente restaurar `std::thread::scope` tras debug
5. No hay manifest (Vista/ Win7 compatibilidad no testeada)
6. Click derecho para menú: a veces el menú no aparece si el cursor se movió mucho
