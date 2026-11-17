//! Envoltorio de libmpv: reproducir, pausa, volumen y posición.
//!
//! ## Por qué cargamos la DLL "a mano"
//! libmpv es una librería en C. En vez de enlazarla al compilar (que en Windows
//! exige archivos `.lib` extra), la abrimos al arrancar con `libloading`.
//! Ventaja: si falta `libmpv-2.dll`, la app muestra un error claro en vez de
//! no abrir.
//!
//! ## Sobre `unsafe`
//! Llamar funciones de C es `unsafe` en Rust: el compilador no puede verificar
//! lo que pasa del otro lado. Por eso TODO el `unsafe` queda encerrado en este
//! archivo, y el resto de la app usa `Reproductor` con métodos normales y seguros.

use libloading::Library;
use std::ffi::{CStr, CString, c_char, c_int, c_void};
use std::ptr;

// Firmas de las funciones de C que usamos (ver mpv/client.h).
type FnCreate = unsafe extern "C" fn() -> *mut c_void;
type FnInitialize = unsafe extern "C" fn(*mut c_void) -> c_int;
type FnSetOptionString = unsafe extern "C" fn(*mut c_void, *const c_char, *const c_char) -> c_int;
type FnCommand = unsafe extern "C" fn(*mut c_void, *mut *const c_char) -> c_int;
type FnSetPropertyString = unsafe extern "C" fn(*mut c_void, *const c_char, *const c_char) -> c_int;
type FnGetPropertyString = unsafe extern "C" fn(*mut c_void, *const c_char) -> *mut c_char;
type FnFree = unsafe extern "C" fn(*mut c_void);
type FnTerminateDestroy = unsafe extern "C" fn(*mut c_void);
type FnErrorString = unsafe extern "C" fn(c_int) -> *const c_char;

/// Un reproductor de audio. Al soltarlo (`drop`) se cierra mpv solo.
pub struct Reproductor {
    // El orden importa poco acá porque cerramos mpv en `Drop` antes de que se
    // suelte `_lib`, pero `_lib` tiene que vivir tanto como el reproductor:
    // si se descargara la DLL, los punteros a funciones quedarían inválidos.
    _lib: Library,
    ctx: *mut c_void,
    command: FnCommand,
    set_property: FnSetPropertyString,
    get_property: FnGetPropertyString,
    free: FnFree,
    terminate_destroy: FnTerminateDestroy,
    error_string: FnErrorString,
}

// mpv documenta que su handle se puede usar desde varios hilos, así que es
// seguro mover el reproductor a un hilo de fondo (lo necesitamos en el paso 6).
unsafe impl Send for Reproductor {}

/// Nombre de la DLL. Se puede cambiar con la variable `YTM_LIBMPV`.
fn ruta_dll() -> String {
    std::env::var("YTM_LIBMPV").unwrap_or_else(|_| {
        if cfg!(windows) { "libmpv-2.dll".into() } else { "libmpv.so.2".into() }
    })
}

/// Convierte texto de Rust a texto de C (terminado en `\0`).
fn c(texto: &str) -> Result<CString, String> {
    CString::new(texto).map_err(|_| format!("Texto con carácter nulo: {texto:?}"))
}

/// Busca una función de la DLL por nombre y copia su puntero.
///
/// SAFETY: quien llama debe pedir el tipo `T` que coincide con la firma real en C.
unsafe fn simbolo<T: Copy>(lib: &Library, nombre: &str) -> Result<T, String> {
    unsafe {
        lib.get::<T>(nombre.as_bytes())
            .map(|s| *s)
            .map_err(|e| format!("Falta la función {nombre} en libmpv: {e}"))
    }
}

impl Reproductor {
    /// Abre libmpv y la configura para reproducir solo audio.
    pub fn nuevo() -> Result<Self, String> {
        let ruta = ruta_dll();
        // SAFETY: cargamos una librería conocida y tomamos funciones con las
        // firmas exactas de mpv/client.h.
        unsafe {
            let lib = Library::new(&ruta)
                .map_err(|e| format!("No se pudo abrir {ruta} (¿está junto al .exe?): {e}"))?;

            let create: FnCreate = simbolo(&lib, "mpv_create")?;
            let initialize: FnInitialize = simbolo(&lib, "mpv_initialize")?;
            let set_option: FnSetOptionString = simbolo(&lib, "mpv_set_option_string")?;

            let ctx = create();
            if ctx.is_null() {
                return Err("mpv_create devolvió nulo".into());
            }

            let r = Reproductor {
                command: simbolo(&lib, "mpv_command")?,
                set_property: simbolo(&lib, "mpv_set_property_string")?,
                get_property: simbolo(&lib, "mpv_get_property_string")?,
                free: simbolo(&lib, "mpv_free")?,
                terminate_destroy: simbolo(&lib, "mpv_terminate_destroy")?,
                error_string: simbolo(&lib, "mpv_error_string")?,
                ctx,
                _lib: lib,
            };

            // Opciones antes de inicializar: sin video, sin ventana, sin terminal.
            // `ytdl=no` porque la URL ya la resolvemos nosotros con yt-dlp.
            for (k, v) in [("vid", "no"), ("video", "no"), ("audio-display", "no"),
                           ("terminal", "no"), ("ytdl", "no"), ("idle", "yes")] {
                let (k, v) = (c(k)?, c(v)?);
                set_option(r.ctx, k.as_ptr(), v.as_ptr());
            }
            let codigo = initialize(r.ctx);
            r.revisar(codigo, "inicializar mpv")?;
            Ok(r)
        }
    }

    /// Convierte un código de error de mpv en `Result`.
    fn revisar(&self, codigo: c_int, accion: &str) -> Result<(), String> {
        if codigo >= 0 {
            return Ok(());
        }
        // SAFETY: mpv_error_string devuelve un texto estático.
        let msg = unsafe { CStr::from_ptr((self.error_string)(codigo)) };
        Err(format!("Error al {accion}: {}", msg.to_string_lossy()))
    }

    /// Ejecuta un comando de mpv, por ejemplo `["loadfile", url]`.
    fn comando(&self, partes: &[&str]) -> Result<(), String> {
        let textos: Vec<CString> = partes.iter().map(|p| c(p)).collect::<Result<_, _>>()?;
        // mpv espera una lista de punteros terminada en nulo.
        let mut punteros: Vec<*const c_char> = textos.iter().map(|t| t.as_ptr()).collect();
        punteros.push(ptr::null());
        // SAFETY: `textos` vive hasta el final de la función, así que los punteros son válidos.
        let codigo = unsafe { (self.command)(self.ctx, punteros.as_mut_ptr()) };
        self.revisar(codigo, partes[0])
    }

    fn poner(&self, nombre: &str, valor: &str) -> Result<(), String> {
        let (n, v) = (c(nombre)?, c(valor)?);
        // SAFETY: punteros válidos durante la llamada.
        let codigo = unsafe { (self.set_property)(self.ctx, n.as_ptr(), v.as_ptr()) };
        self.revisar(codigo, nombre)
    }

    fn leer(&self, nombre: &str) -> Option<String> {
        let n = c(nombre).ok()?;
        // SAFETY: si mpv devuelve un texto, lo copiamos y lo liberamos con mpv_free.
        unsafe {
            let p = (self.get_property)(self.ctx, n.as_ptr());
            if p.is_null() {
                return None;
            }
            let texto = CStr::from_ptr(p).to_string_lossy().into_owned();
            (self.free)(p.cast());
            Some(texto)
        }
    }

    // ---- API pública, sin nada de `unsafe` ----

    /// Empieza a reproducir una URL (reemplaza lo que estuviera sonando).
    pub fn reproducir(&self, url: &str) -> Result<(), String> {
        self.comando(&["loadfile", url, "replace"])?;
        self.poner("pause", "no")
    }

    pub fn pausar(&self, pausa: bool) -> Result<(), String> {
        self.poner("pause", if pausa { "yes" } else { "no" })
    }

    pub fn alternar_pausa(&self) -> Result<(), String> {
        self.comando(&["cycle", "pause"])
    }

    pub fn en_pausa(&self) -> bool {
        self.leer("pause").as_deref() == Some("yes")
    }

    /// Volumen de 0 a 100 (se recorta si te pasás).
    pub fn volumen(&self, valor: u8) -> Result<(), String> {
        self.poner("volume", &valor.min(100).to_string())
    }

    /// Mover a un segundo concreto de la pista.
    pub fn ir_a(&self, segundos: f64) -> Result<(), String> {
        self.comando(&["seek", &segundos.max(0.0).to_string(), "absolute"])
    }

    pub fn detener(&self) -> Result<(), String> {
        self.comando(&["stop"])
    }

    /// Segundo actual de la reproducción, si hay algo sonando.
    pub fn posicion(&self) -> Option<f64> {
        self.leer("time-pos")?.parse().ok()
    }

    /// Duración total en segundos, si ya se conoce.
    pub fn duracion(&self) -> Option<f64> {
        self.leer("duration")?.parse().ok()
    }

    /// `true` cuando no hay nada cargado (terminó la pista o se detuvo).
    pub fn inactivo(&self) -> bool {
        self.leer("idle-active").as_deref() == Some("yes")
    }
}

impl Drop for Reproductor {
    fn drop(&mut self) {
        // SAFETY: ctx es válido y no se vuelve a usar después de esto.
        unsafe { (self.terminate_destroy)(self.ctx) };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dll_inexistente_da_error_claro() {
        // SAFETY del test: cambiar variables de entorno en tests puede chocar
        // con otros hilos; este es el único test que toca YTM_LIBMPV.
        unsafe { std::env::set_var("YTM_LIBMPV", "no-existe-esta-dll.dll") };
        let err = Reproductor::nuevo().err().unwrap();
        unsafe { std::env::remove_var("YTM_LIBMPV") };
        assert!(err.contains("No se pudo abrir"), "{err}");
    }

    /// Reproduce audio real unos segundos: necesita libmpv, yt-dlp y red.
    /// Se corre con `cargo test -- --ignored`. ¡Va a sonar por los parlantes!
    #[test]
    #[ignore]
    fn real_reproduce_audio() {
        let url = crate::stream::obtener_url_audio("dQw4w9WgXcQ").unwrap();
        let r = Reproductor::nuevo().unwrap();
        r.volumen(30).unwrap();
        r.reproducir(&url).unwrap();
        std::thread::sleep(std::time::Duration::from_secs(6));
        let pos = r.posicion().unwrap_or(0.0);
        assert!(pos > 1.0, "debería haber avanzado, posición = {pos}");
        r.pausar(true).unwrap();
        assert!(r.en_pausa());
    }
}
