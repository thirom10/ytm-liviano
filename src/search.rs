//! Búsqueda y parseo de respuestas -> pistas.
//! API NO OFICIAL: la búsqueda (paso 3) dependerá de la API interna de YouTube Music.

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
