//! Ejecuta `yt-dlp` como subproceso y devuelve la URL directa del audio.
//!
//! No reimplementamos la extracción: yt-dlp se actualiza seguido cuando YouTube
//! cambia cosas, así que delegarle eso es lo más robusto.

use std::process::Command;

/// Programa a ejecutar. Se puede cambiar con la variable de entorno `YTM_YTDLP`
/// (por ejemplo, si `yt-dlp.exe` está junto al `.exe` de la app).
fn programa() -> String {
    std::env::var("YTM_YTDLP").unwrap_or_else(|_| "yt-dlp".to_string())
}

/// Un ID de YouTube tiene 11 caracteres: letras, números, `-` o `_`.
/// Lo validamos antes de pasarlo a yt-dlp para que nunca se cuele algo raro
/// (por ejemplo un texto que empiece con `-` y yt-dlp lo tome como opción).
pub fn id_valido(id: &str) -> bool {
    id.len() == 11
        && id.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

/// Argumentos para yt-dlp: mejor audio disponible, solo imprimir la URL (`-g`).
fn argumentos(id: &str) -> Vec<String> {
    vec![
        "-f".into(),
        "bestaudio".into(),
        "-g".into(),
        "--no-playlist".into(),
        "--no-warnings".into(),
        format!("https://music.youtube.com/watch?v={id}"),
    ]
}

/// De la salida de yt-dlp, toma la primera línea que sea una URL.
fn primera_url(salida: &str) -> Option<String> {
    salida
        .lines()
        .map(str::trim)
        .find(|l| l.starts_with("http"))
        .map(str::to_string)
}

/// Pide a yt-dlp la URL del audio de un video. Tarda 1-3 segundos, así que
/// la UI la va a llamar desde un hilo de fondo (paso 6), nunca directamente.
pub fn obtener_url_audio(id: &str) -> Result<String, String> {
    if !id_valido(id) {
        return Err(format!("ID de video inválido: {id:?}"));
    }

    let mut cmd = Command::new(programa());
    cmd.args(argumentos(id));

    // En Windows, evita que aparezca una ventana negra de consola cada vez.
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }

    let salida = cmd
        .output()
        .map_err(|e| format!("No se pudo ejecutar yt-dlp (¿está instalado?): {e}"))?;

    if !salida.status.success() {
        let err = String::from_utf8_lossy(&salida.stderr);
        return Err(format!("yt-dlp falló: {}", err.trim()));
    }

    let texto = String::from_utf8_lossy(&salida.stdout);
    primera_url(&texto).ok_or_else(|| "yt-dlp no devolvió ninguna URL".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_validos_e_invalidos() {
        assert!(id_valido("dQw4w9WgXcQ"));
        assert!(id_valido("abc-_123XYZ"));
        assert!(!id_valido("corto"));
        assert!(!id_valido("--exec=calc")); // 11 chars pero con '=' : rechazado
        assert!(!id_valido("dQw4w9WgXcQ extra"));
    }

    #[test]
    fn arma_la_url_de_youtube_music() {
        let args = argumentos("dQw4w9WgXcQ");
        assert_eq!(args.last().unwrap(), "https://music.youtube.com/watch?v=dQw4w9WgXcQ");
        assert!(args.contains(&"bestaudio".to_string()));
    }

    #[test]
    fn toma_la_primera_url() {
        let salida = "aviso cualquiera\nhttps://rr1.googlevideo.com/a?x=1\nhttps://otra\n";
        assert_eq!(primera_url(salida).unwrap(), "https://rr1.googlevideo.com/a?x=1");
        assert_eq!(primera_url("nada útil"), None);
    }

    #[test]
    fn id_invalido_no_ejecuta_nada() {
        assert!(obtener_url_audio("-malo").is_err());
    }

    /// Test real contra YouTube: necesita red y yt-dlp. No corre por defecto;
    /// se ejecuta con `cargo test -- --ignored`.
    #[test]
    #[ignore]
    fn real_obtiene_url() {
        let url = obtener_url_audio("dQw4w9WgXcQ").unwrap();
        assert!(url.starts_with("https://"));
    }
}
