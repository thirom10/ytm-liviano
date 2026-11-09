# Proyecto: reproductor liviano de YouTube Music (Rust)

App de escritorio (Windows, y Linux si se puede) para escuchar YouTube Music con muy poco consumo de recursos, para poder jugar y escuchar música a la vez sin tener Chrome abierto.

## Contexto del usuario
- Conoce otros lenguajes (Git, Linux en VPS, PowerShell) pero **nunca escribió Rust**.
- Explicar las decisiones de Rust con claridad; preferir código idiomático y simple (nada de lifetimes complejos, macros propias o async innecesario).
- Comentarios, README y mensajes de UI en **español**.
- Primera versión chica y funcional; después iterar.

## Forma de trabajo
- Primero plan y arquitectura; **esperar confirmación** antes de escribir mucho código.
- Pasos chicos y verificables; tras cada paso explicar qué cambió y por qué, en lenguaje simple.
- Si una decisión de Rust no es obvia (ownership del estado de la UI, comunicación hilo de audio ↔ UI), explicarla con un ejemplo corto.
- Marcar con `// API NO OFICIAL:` todo código que dependa de la API interna de YouTube, para poder reemplazarlo.

## Requisitos funcionales
1. Buscar canciones/videos de YouTube Music.
2. Reproducir solo audio (sin renderizar video).
3. Play/pausa, siguiente, anterior, volumen, barra de progreso.
4. Cola de reproducción simple.
5. (Fase 2) Login para playlists, likes y biblioteca.

## Requisitos no funcionales
- RAM: decenas de MB. Sin navegador embebido (no Electron). Arranque rápido, binario único si es posible.

## Stack propuesto (a validar, ver docs/PLAN.md)
- UI: `egui`/`eframe` (alternativas: `slint`, `tauri`).
- Audio: `libmpv` (alternativa: `rodio` + `symphonia`).
- Stream: `yt-dlp` como subproceso (`-f bestaudio -g`). No reimplementar la extracción.
- Búsqueda: InnerTube de YTM con `reqwest` + `serde_json`, o sidecar Python con `ytmusicapi`.
- Async: `tokio` solo si hace falta.

## Limitaciones (no ocultar)
- YouTube Data API v3 no reproduce música de YTM.
- No hay API oficial de la biblioteca de YTM; las opciones no oficiales pueden romperse sin aviso.
- Ventajas de Premium (sin anuncios, offline, segundo plano oficial) no se replican legítimamente. **No** implementar descargas offline ni bloqueo de anuncios.
- Extraer streams va contra los ToS de YouTube: uso personal, no distribuir sin revisar.
- Cookies/credenciales: cifradas o en el keyring del sistema, nunca en texto plano en el repo.

## Fuera de alcance
Descargas offline, eliminar anuncios, distribución pública, cuentas de otros usuarios, versión móvil.
