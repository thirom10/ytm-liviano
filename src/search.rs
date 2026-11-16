//! Búsqueda y parseo de respuestas -> pistas.
//!
//! API NO OFICIAL: todo este módulo depende de la API interna de YouTube Music
//! (InnerTube). Si YouTube cambia el formato, solo hay que tocar este archivo.

use serde_json::Value;

/// Una canción o video que se puede reproducir.
///
/// Usamos `String` (texto que la struct posee) en vez de `&str` (texto prestado):
/// así cada `Pista` es dueña de sus datos y no hay que preocuparse por lifetimes.
#[derive(Debug, Clone, PartialEq)]
pub struct Pista {
    /// ID del video de YouTube (por ejemplo "dQw4w9WgXcQ").
    pub id: String,
    pub titulo: String,
    pub artista: String,
    /// Duración en segundos. `Option` porque a veces la búsqueda no la trae.
    pub duracion_seg: Option<u32>,
}

/// Convierte el JSON de una búsqueda en una lista de pistas.
///
/// Devuelve `Result`: `Ok(pistas)` si el JSON se pudo leer, o `Err(mensaje)` si no.
/// Los resultados que no se pueden reproducir (artistas, álbumes) se saltean.
// API NO OFICIAL: las rutas de abajo son el formato interno de YouTube Music.
pub fn parsear_busqueda(json: &str) -> Result<Vec<Pista>, String> {
    let raiz: Value = serde_json::from_str(json).map_err(|e| format!("JSON inválido: {e}"))?;

    // `pointer` navega el JSON con una ruta tipo "/a/b/0/c" y devuelve `None`
    // si algo no existe, en vez de romper el programa.
    let secciones = raiz
        .pointer("/contents/tabbedSearchResultsRenderer/tabs/0/tabRenderer/content/sectionListRenderer/contents")
        .and_then(Value::as_array)
        .ok_or("La respuesta no tiene el formato de búsqueda esperado")?;

    let mut pistas = Vec::new();
    for seccion in secciones {
        let Some(items) = seccion.pointer("/musicShelfRenderer/contents").and_then(Value::as_array) else {
            continue; // secciones sin lista de resultados (sugerencias, etc.)
        };
        for item in items {
            if let Some(p) = parsear_item(item) {
                pistas.push(p);
            }
        }
    }
    Ok(pistas)
}

/// Lee un resultado. `None` si no es reproducible (no tiene videoId).
fn parsear_item(item: &Value) -> Option<Pista> {
    let r = item.get("musicResponsiveListItemRenderer")?;
    let id = r.pointer("/playlistItemData/videoId")?.as_str()?.to_string();
    let titulo = textos_columna(r, 0).into_iter().next()?;

    // La segunda columna viene como "Canción • Artista • Álbum • 3:33".
    let mut partes = textos_columna(r, 1);
    partes.retain(|t| t.trim() != "•");
    if partes.first().is_some_and(|t| es_tipo(t)) {
        partes.remove(0);
    }
    let duracion_seg = partes.last().and_then(|t| parsear_duracion(t));
    let artista = partes.first().cloned().unwrap_or_default();

    Some(Pista { id, titulo, artista, duracion_seg })
}

/// Textos ("runs") de una columna del resultado.
fn textos_columna(r: &Value, columna: usize) -> Vec<String> {
    let ruta = format!("/flexColumns/{columna}/musicResponsiveListItemFlexColumnRenderer/text/runs");
    r.pointer(&ruta)
        .and_then(Value::as_array)
        .map(|runs| {
            runs.iter()
                .filter_map(|run| run.get("text")?.as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default()
}

/// Etiquetas de tipo que YouTube pone antes del artista.
fn es_tipo(texto: &str) -> bool {
    matches!(texto, "Canción" | "Video" | "Song" | "Episodio" | "Episode")
}

/// "3:33" -> 213, "1:02:05" -> 3725. `None` si no es una duración.
pub fn parsear_duracion(texto: &str) -> Option<u32> {
    let partes: Vec<&str> = texto.trim().split(':').collect();
    if !(2..=3).contains(&partes.len()) {
        return None;
    }
    let mut total = 0u32;
    for p in partes {
        total = total * 60 + p.parse::<u32>().ok()?;
    }
    Some(total)
}

#[cfg(test)]
mod tests {
    use super::*;

    // `include_str!` mete el archivo dentro del binario de tests al compilar.
    const EJEMPLO: &str = include_str!("fixtures/busqueda_ejemplo.json");

    #[test]
    fn parsea_las_pistas_reproducibles() {
        let pistas = parsear_busqueda(EJEMPLO).unwrap();
        assert_eq!(pistas.len(), 3); // el artista sin videoId se saltea
    }

    #[test]
    fn lee_titulo_artista_y_duracion() {
        let p = &parsear_busqueda(EJEMPLO).unwrap()[0];
        assert_eq!(p.id, "abc123XYZ01");
        assert_eq!(p.titulo, "De Música Ligera");
        assert_eq!(p.artista, "Soda Stereo");
        assert_eq!(p.duracion_seg, Some(213));
    }

    #[test]
    fn video_largo_con_horas() {
        let p = &parsear_busqueda(EJEMPLO).unwrap()[1];
        assert_eq!(p.artista, "Soda Stereo");
        assert_eq!(p.duracion_seg, Some(3725));
    }

    #[test]
    fn sin_duracion_queda_en_none() {
        let p = &parsear_busqueda(EJEMPLO).unwrap()[2];
        assert_eq!(p.artista, "Artista X");
        assert_eq!(p.duracion_seg, None);
    }

    #[test]
    fn json_invalido_da_error() {
        assert!(parsear_busqueda("esto no es json").is_err());
    }

    #[test]
    fn formato_inesperado_da_error() {
        assert!(parsear_busqueda(r#"{"otra":"cosa"}"#).is_err());
    }

    #[test]
    fn duraciones() {
        assert_eq!(parsear_duracion("0:45"), Some(45));
        assert_eq!(parsear_duracion("3:33"), Some(213));
        assert_eq!(parsear_duracion("1:02:05"), Some(3725));
        assert_eq!(parsear_duracion("Soda Stereo"), None);
        assert_eq!(parsear_duracion("12"), None);
    }
}
