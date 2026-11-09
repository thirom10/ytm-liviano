// Punto de entrada de ytm-liviano.
//
// En Rust, cada `mod` declara un módulo: el compilador busca su código en
// `src/<nombre>.rs`. Por ahora todos están vacíos; se irán llenando paso a paso
// (ver docs/PLAN.md).

mod audio;
mod config;
// `allow(dead_code)` es temporal: la cola y `Pista` todavía no se usan desde
// `main`, y sin esto el compilador avisa "código sin usar". Se quita en el paso 6.
#[allow(dead_code)]
mod queue;
#[allow(dead_code)]
mod search;
mod stream;
mod ui;

fn main() {
    println!("ytm-liviano: esqueleto listo (todavía no reproduce nada).");
}
