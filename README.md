# ytm-liviano

Reproductor de escritorio **liviano** para YouTube Music, escrito en Rust. Objetivo: consumir decenas de MB de RAM (no cientos) para escuchar música mientras jugás.

> **Estado:** fase de planificación. Todavía no hay código; ver [`docs/PLAN.md`](docs/PLAN.md).

## Requisitos (previstos)
- [Rust](https://rustup.rs) (stable)
- [yt-dlp](https://github.com/yt-dlp/yt-dlp) en el `PATH`
- `libmpv` (en Windows: `libmpv-2.dll` junto al `.exe`)

## Compilar y correr
Pendiente hasta tener la primera versión (`cargo run --release`).

## Limitaciones importantes
- La API oficial YouTube Data v3 **no** reproduce música de YouTube Music.
- No existe API oficial para la biblioteca de YTM; se usaría la API interna (no oficial), que **puede romperse sin aviso**.
- Las ventajas de Premium (sin anuncios, descargas offline, segundo plano oficial) **no se replican** desde una app propia. No se implementan descargas ni bloqueo de anuncios.
- Extraer streams de YouTube va contra sus Términos de Servicio. Proyecto de **uso personal**; no distribuir públicamente sin revisarlo.
- Las credenciales/cookies nunca van en el repo (ver `.gitignore`); se guardarán cifradas o en el keyring del sistema.
