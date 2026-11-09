// Punto de entrada de ytm-liviano.
//
// En Rust, cada `mod` declara un módulo: el compilador busca su código en
// `src/<nombre>.rs`. Por ahora todos están vacíos; se irán llenando paso a paso
// (ver docs/PLAN.md).

mod audio;
mod config;
mod queue;
mod search;
mod stream;
mod ui;

fn main() {
    println!("ytm-liviano: esqueleto listo (todavía no reproduce nada).");
}
