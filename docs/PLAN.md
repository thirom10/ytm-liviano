# Plan y arquitectura (propuesta, pendiente de confirmación)

## 1. Flujo de datos
```
 UI (egui)  ──comandos──▶  hilo de fondo (trabajador)
    ▲                        ├─ search  → InnerTube/ytmusicapi → Vec<Pista>
    │                        ├─ stream  → yt-dlp -f bestaudio -g → URL
    └──────eventos───────────└─ audio   → libmpv.loadfile(URL)
```
La UI nunca bloquea: manda `Comando` por un canal (`std::sync::mpsc`) y recibe `Evento` por otro. En Rust esto evita compartir estado mutable entre hilos: se *mueven* mensajes en vez de compartir memoria.

## 2. Módulos
| Módulo | Responsabilidad |
|---|---|
| `ui` | Ventana egui: búsqueda, resultados, cola, controles |
| `search` | Buscar y parsear respuestas → `Pista`. **Depende de API no oficial** |
| `stream` | Ejecutar `yt-dlp` y devolver la URL de audio |
| `audio` | Envolver libmpv: play/pausa, volumen, posición |
| `queue` | Cola pura (sin I/O), fácil de testear |
| `config` | Rutas, volumen guardado, (fase 2) credenciales vía keyring |

## 3. Comparativa de decisiones
**UI**: `egui` (recomendado: Rust puro, simple, ~20–40 MB) · `slint` (DSL propio, similar peso) · `tauri` (UI web, más pesado y suma frontend).
**Audio**: `libmpv` (recomendado: robusto, hay que distribuir la DLL) · `rodio`+`symphonia` (más código propio, sin DLL).
**Búsqueda**: InnerTube directo (un solo binario, pero hay que mantener el parseo) · sidecar Python con `ytmusicapi` (mantenido por la comunidad, pero exige Python en la máquina). Recomendación inicial: InnerTube directo, aislado en `search` para poder cambiarlo.
**Async**: `reqwest` en modo bloqueante + hilos; sin `tokio` en la v1.

## 4. Pasos de la fase 1
1. Esqueleto Cargo + módulos vacíos + CI de `cargo test`.
2. `queue` con tests.
3. `search`: parseo con respuestas JSON de ejemplo + tests.
4. `stream`: llamada a yt-dlp.
5. `audio`: reproducir una URL fija.
6. `ui`: unir todo (buscar → elegir → reproducir → play/pausa/siguiente).
7. README final con compilación y distribución de la DLL.

## 5. Preguntas abiertas
- ¿Confirmás `egui` + `libmpv` + InnerTube directo?
- ¿Linux desde el inicio o primero solo Windows?
