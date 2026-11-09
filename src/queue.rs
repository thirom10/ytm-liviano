//! Cola de reproducción pura (sin I/O), fácil de testear.
//!
//! Guarda una lista de pistas y cuál es la actual. No reproduce nada: solo decide
//! "qué pista toca ahora". El audio y la UI le preguntan a la cola.

use crate::search::Pista;

#[derive(Debug, Default)]
pub struct Cola {
    pistas: Vec<Pista>,
    /// Posición de la pista actual. `None` = todavía no se eligió ninguna.
    actual: Option<usize>,
}

impl Cola {
    pub fn new() -> Self {
        Self::default()
    }

    /// Agrega una pista al final. La cola pasa a ser dueña de la pista
    /// (en Rust se dice que la pista se "mueve" adentro de la cola).
    pub fn agregar(&mut self, pista: Pista) {
        self.pistas.push(pista);
    }

    pub fn len(&self) -> usize {
        self.pistas.len()
    }

    pub fn esta_vacia(&self) -> bool {
        self.pistas.is_empty()
    }

    /// Todas las pistas, solo para leer (la UI las dibuja).
    /// `&[Pista]` es una "vista prestada": se mira pero no se puede modificar.
    pub fn pistas(&self) -> &[Pista] {
        &self.pistas
    }

    /// Posición de la pista actual, si hay.
    pub fn indice_actual(&self) -> Option<usize> {
        self.actual
    }

    /// La pista actual, si hay. Devuelve una referencia (`&Pista`): prestada, no copiada.
    pub fn actual(&self) -> Option<&Pista> {
        self.actual.and_then(|i| self.pistas.get(i))
    }

    /// Elige una pista concreta (por ejemplo cuando el usuario hace clic en ella).
    /// Si la posición no existe devuelve `None` y no cambia nada.
    pub fn elegir(&mut self, indice: usize) -> Option<&Pista> {
        if indice < self.pistas.len() {
            self.actual = Some(indice);
        }
        self.actual()
    }

    /// Avanza a la siguiente pista.
    /// - Si todavía no había actual, empieza por la primera.
    /// - Si ya era la última, devuelve `None` y se queda donde estaba.
    pub fn siguiente(&mut self) -> Option<&Pista> {
        let proximo = match self.actual {
            None => 0,
            Some(i) => i + 1,
        };
        if proximo < self.pistas.len() {
            self.actual = Some(proximo);
            self.actual()
        } else {
            None
        }
    }

    /// Vuelve a la pista anterior.
    /// - Si ya estás en la primera, devuelve esa misma (la UI puede reiniciarla).
    /// - Si no había actual, devuelve `None`.
    pub fn anterior(&mut self) -> Option<&Pista> {
        if let Some(i) = self.actual {
            self.actual = Some(i.saturating_sub(1)); // nunca baja de 0
        }
        self.actual()
    }

    /// Quita la pista de esa posición y la devuelve. `None` si no existe.
    /// Si se quita la que estaba sonando, la cola queda sin pista actual.
    pub fn quitar(&mut self, indice: usize) -> Option<Pista> {
        if indice >= self.pistas.len() {
            return None;
        }
        let quitada = self.pistas.remove(indice);
        self.actual = match self.actual {
            Some(a) if a == indice => None,
            Some(a) if a > indice => Some(a - 1), // todo se corrió un lugar
            otro => otro,
        };
        Some(quitada)
    }

    pub fn vaciar(&mut self) {
        self.pistas.clear();
        self.actual = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pista(n: u32) -> Pista {
        Pista {
            id: format!("id{n}"),
            titulo: format!("Tema {n}"),
            artista: "Artista".to_string(),
            duracion_seg: Some(180),
        }
    }

    fn cola_con(n: u32) -> Cola {
        let mut c = Cola::new();
        for i in 1..=n {
            c.agregar(pista(i));
        }
        c
    }

    #[test]
    fn cola_nueva_esta_vacia_y_sin_actual() {
        let c = Cola::new();
        assert!(c.esta_vacia());
        assert_eq!(c.actual(), None);
    }

    #[test]
    fn agregar_no_elige_actual() {
        let c = cola_con(2);
        assert_eq!(c.len(), 2);
        assert_eq!(c.actual(), None);
    }

    #[test]
    fn siguiente_empieza_por_la_primera() {
        let mut c = cola_con(3);
        assert_eq!(c.siguiente().unwrap().id, "id1");
        assert_eq!(c.siguiente().unwrap().id, "id2");
    }

    #[test]
    fn siguiente_en_la_ultima_devuelve_none_y_no_se_mueve() {
        let mut c = cola_con(2);
        c.elegir(1);
        assert_eq!(c.siguiente(), None);
        assert_eq!(c.indice_actual(), Some(1));
    }

    #[test]
    fn siguiente_en_cola_vacia_devuelve_none() {
        let mut c = Cola::new();
        assert_eq!(c.siguiente(), None);
    }

    #[test]
    fn anterior_retrocede_y_se_frena_en_la_primera() {
        let mut c = cola_con(3);
        c.elegir(2);
        assert_eq!(c.anterior().unwrap().id, "id2");
        assert_eq!(c.anterior().unwrap().id, "id1");
        assert_eq!(c.anterior().unwrap().id, "id1"); // no baja de la primera
    }

    #[test]
    fn anterior_sin_actual_devuelve_none() {
        let mut c = cola_con(2);
        assert_eq!(c.anterior(), None);
    }

    #[test]
    fn elegir_fuera_de_rango_no_cambia_nada() {
        let mut c = cola_con(2);
        c.elegir(0);
        assert_eq!(c.elegir(5).unwrap().id, "id1"); // sigue la anterior
        assert_eq!(c.indice_actual(), Some(0));
    }

    #[test]
    fn quitar_una_anterior_a_la_actual_corre_el_indice() {
        let mut c = cola_con(3);
        c.elegir(2); // suena "id3"
        c.quitar(0);
        assert_eq!(c.actual().unwrap().id, "id3");
        assert_eq!(c.indice_actual(), Some(1));
    }

    #[test]
    fn quitar_la_actual_deja_sin_actual() {
        let mut c = cola_con(3);
        c.elegir(1);
        let quitada = c.quitar(1).unwrap();
        assert_eq!(quitada.id, "id2");
        assert_eq!(c.actual(), None);
        assert_eq!(c.len(), 2);
    }

    #[test]
    fn quitar_posterior_a_la_actual_no_la_afecta() {
        let mut c = cola_con(3);
        c.elegir(0);
        c.quitar(2);
        assert_eq!(c.actual().unwrap().id, "id1");
    }

    #[test]
    fn quitar_fuera_de_rango_devuelve_none() {
        let mut c = cola_con(1);
        assert_eq!(c.quitar(3), None);
        assert_eq!(c.len(), 1);
    }

    #[test]
    fn vaciar_limpia_todo() {
        let mut c = cola_con(3);
        c.elegir(1);
        c.vaciar();
        assert!(c.esta_vacia());
        assert_eq!(c.actual(), None);
    }
}
