use rand::seq::SliceRandom;

/// Referenz auf ein Partikel im World-Grid.
/// Ermöglicht Rückverfolgung: Wer sitzt an dieser Position?
#[derive(Debug, Clone, Copy)]
pub enum ParticleRef {
    Free(usize),                      // Index in sim.particles
    InObject(usize, usize, usize),    // object_idx, i, j im object_grid
    Static,                           // Boden, Hindernisse (nicht trackbar)
}

/// Materialtypen für Partikel.
///
/// Jede Zelle im Grid hat Volumen = 1 (architektonische Konstante).
/// Daher gilt: Masse = Dichte × 1 = Dichte.
/// Die density()-Werte SIND die Massen pro Partikel.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MaterialTyp {
    Sand,
    Stein,
    Metall,
    Luft,
    Wasser,
    Holz,
}

impl MaterialTyp {
    /// Bindungsstärke zwischen benachbarten Partikeln.
    /// Höher = schwerer zu brechen.
    pub fn binding_strength(&self) -> f32 {
        match self {
            MaterialTyp::Sand => 2.0,      // Lose Körner, kaum Bindung
            MaterialTyp::Stein => 80.0,    // Starke kristalline Bindung
            MaterialTyp::Metall => 200.0,  // Sehr starke metallische Bindung
            MaterialTyp::Luft => 0.0,      // Keine Bindung, Gas
            MaterialTyp::Wasser => 0.0,    // Keine Bindung, Flüssigkeit
            MaterialTyp::Holz => 40.0,     // Mittlere Faserbindung
        }
    }

    /// Dichte des Materials.
    /// Da Zellvolumen = 1, entspricht dieser Wert direkt der Masse.
    /// Werte relativ zu Wasser (Wasser = 1.0).
    pub fn density(&self) -> f32 {
        match self {
            MaterialTyp::Sand => 1.5,      // Schwerer als Wasser, sinkt
            MaterialTyp::Stein => 2.5,     // Granit-ähnlich
            MaterialTyp::Metall => 8.0,    // Eisen-ähnlich
            MaterialTyp::Luft => 0.001,    // Fast masselos, aber nicht 0
            MaterialTyp::Wasser => 1.0,    // Referenzwert
            MaterialTyp::Holz => 0.6,      // Leichter als Wasser, schwimmt
        }
    }

    /// Ist das Material fest?
    /// false = fließt/strömt (Flüssigkeit, Gas)
    pub fn is_solid(&self) -> bool {
        match self {
            MaterialTyp::Sand => true,     // Fest, aber granular
            MaterialTyp::Stein => true,    // Fest
            MaterialTyp::Metall => true,   // Fest
            MaterialTyp::Luft => false,    // Gas
            MaterialTyp::Wasser => false,  // Flüssig
            MaterialTyp::Holz => true,     // Fest
        }
    }

    /// Dämpfungsfaktor bei Aufprall.
    /// Wie viel der Impact-Kraft wird weitergegeben?
    /// 1.0 = volle Kraft (hart), 0.1 = stark gedämpft (weich)
    pub fn impact_dampening(&self) -> f32 {
        match self {
            MaterialTyp::Sand => 0.3,      // Weich, dämpft stark
            MaterialTyp::Stein => 1.0,     // Hart, keine Dämpfung
            MaterialTyp::Metall => 0.9,    // Fast hart
            MaterialTyp::Luft => 0.0,      // Kein Widerstand
            MaterialTyp::Wasser => 0.2,    // Stark dämpfend
            MaterialTyp::Holz => 0.6,      // Mittel
        }
    }

    /// Farbe des Materials als (R, G, B) Werte zwischen 0.0 und 1.0.
    /// Wird vom Renderer zu seiner nativen Farbdarstellung konvertiert.
    /// lib.rs bleibt damit unabhängig von Bevy.
    pub fn color(&self) -> (f32, f32, f32) {
        match self {
            MaterialTyp::Sand => (0.9, 0.75, 0.4),      // Sandgelb/Beige
            MaterialTyp::Stein => (0.5, 0.5, 0.5),     // Neutrales Grau
            MaterialTyp::Metall => (0.7, 0.75, 0.8),   // Kühles Silber
            MaterialTyp::Luft => (0.9, 0.95, 1.0),     // Fast weiß, leicht bläulich
            MaterialTyp::Wasser => (0.2, 0.5, 0.8),    // Klares Blau
            MaterialTyp::Holz => (0.55, 0.35, 0.15),   // Warmes Braun
        }
    }
}

//Struktur Partikel
#[derive(Debug, Clone)]
pub struct Particle {
    pub id: i32,
    pub position: [f32; 2],
    pub velocity: [f32; 2],
    pub material: MaterialTyp,
    pub particle_ref: ParticleRef,  // NEU: Weiß, wer es ist
}

impl Particle {
    pub fn new(id: i32, position: [f32; 2], velocity: [f32; 2], material: MaterialTyp, particle_ref: ParticleRef) -> Particle {
        Particle {
            id,
            position,
            velocity,
            material,
            particle_ref,
        }
    }

    pub fn mass(&self) -> f32 {
        self.material.density()
    }

    pub fn check_way(&self, world: &World) -> Option<(f32, i32, i32)> {
        let own_x_pos = self.position[0] as i32;
        let own_y_pos = self.position[1] as i32;

        let max_x = (world.width - 1) as i32;
        let max_y = (world.height - 1) as i32;

        let can_go_left = own_x_pos > 0;
        let can_go_right = own_x_pos < max_x;
        let can_go_down = own_y_pos > 0;
        let can_go_up = own_y_pos < max_y;

        let mut values: Vec<(f32, i32, i32)> = Vec::new();

        if can_go_up && can_go_right {
            let pressure = world.grid[(own_y_pos + 1) as usize][(own_x_pos + 1) as usize].2;
            values.push((pressure, own_x_pos + 1, own_y_pos + 1));
        }
        if can_go_right {
            let pressure = world.grid[(own_y_pos) as usize][(own_x_pos + 1) as usize].2;
            values.push((pressure, own_x_pos + 1, own_y_pos));
        }
        if can_go_down && can_go_right {
            let pressure = world.grid[(own_y_pos - 1) as usize][(own_x_pos + 1) as usize].2;
            values.push((pressure, own_x_pos + 1, own_y_pos - 1));
        }
        if can_go_down {
            let pressure = world.grid[(own_y_pos - 1) as usize][(own_x_pos) as usize].2;
            values.push((pressure, own_x_pos, own_y_pos - 1));
        }
        if can_go_down && can_go_left {
            let pressure = world.grid[(own_y_pos - 1) as usize][(own_x_pos - 1) as usize].2;
            values.push((pressure, own_x_pos - 1, own_y_pos - 1));
        }
        if can_go_left {
            let pressure = world.grid[(own_y_pos) as usize][(own_x_pos - 1) as usize].2;
            values.push((pressure, own_x_pos - 1, own_y_pos));
        }
        if can_go_up && can_go_left {
            let pressure = world.grid[(own_y_pos + 1) as usize][(own_x_pos - 1) as usize].2;
            values.push((pressure, own_x_pos - 1, own_y_pos + 1));
        }

        let min_pressure = values.iter().map(|v| v.0).fold(f32::INFINITY, |a, b| a.min(b));

        let min_options: Vec<_> = values.iter()
            .filter(|v| v.0 == min_pressure)
            .collect();

        match min_options.choose(&mut rand::thread_rng()) {
            Some(&&(pressure, x, y)) => {
                Some((pressure, x, y))
            }
            None => {
                None
            }
        }
    }

    pub fn resolve_pressure(&mut self, world: &mut World) {
        let own_x = self.position[0] as usize;
        let own_y = self.position[1] as usize;
        let own_pressure = world.give_pressure_on_position(own_x, own_y);

        if own_pressure <= self.mass() {
            return;
        }

        if let Some((min_pressure, target_x, target_y)) = self.check_way(world) {
            if min_pressure < own_pressure && target_y <= own_y as i32 {
                if world.give_occupation_on_position(target_x as usize, target_y as usize).is_none() {
                    world.clear_occupation_on_position(self.position);
                    world.clear_mass_on_position(self.position);
                    self.position[0] = target_x as f32;
                    self.position[1] = target_y as f32;
                    world.update_occupation_on_position(self.position, self.particle_ref);
                    world.update_mass_on_position(self.position, self.mass());
                }
            }
        }
    }

    pub fn fall_down(&mut self, world: &mut World) {
        let x = self.position[0] as i32;
        let y = self.position[1] as i32;

        if y <= 0 {
            return;
        }

        // Gerade runter frei?
        if world.give_occupation_on_position(x as usize, (y - 1) as usize).is_none() {
            world.clear_occupation_on_position(self.position);
            world.clear_mass_on_position(self.position);
            self.position[1] -= 1.0;
            world.update_occupation_on_position(self.position, self.particle_ref);
            world.update_mass_on_position(self.position, self.mass());
            return;
        }

        // Diagonal links unten frei?
        if x > 0 && world.give_occupation_on_position((x - 1) as usize, (y - 1) as usize).is_none() {
            world.clear_occupation_on_position(self.position);
            world.clear_mass_on_position(self.position);
            self.position[0] -= 1.0;
            self.position[1] -= 1.0;
            world.update_occupation_on_position(self.position, self.particle_ref);
            world.update_mass_on_position(self.position, self.mass());
            return;
        }

        // Diagonal rechts unten frei?
        if x < (world.width - 1) as i32 && world.give_occupation_on_position((x + 1) as usize, (y - 1) as usize).is_none() {
            world.clear_occupation_on_position(self.position);
            world.clear_mass_on_position(self.position);
            self.position[0] += 1.0;
            self.position[1] -= 1.0;
            world.update_occupation_on_position(self.position, self.particle_ref);
            world.update_mass_on_position(self.position, self.mass());
        }
    }

    pub fn get_position(&self) -> [f32; 2] {
        self.position
    }

    pub fn get_velocity(&self) -> [f32; 2] {
        self.velocity
    }

    pub fn get_impuls(&self) -> [f32; 2] {
        [self.velocity[0] * self.mass(), self.velocity[1] * self.mass()]
    }

    pub fn update_position(&mut self, world: &mut World) {
        world.clear_occupation_on_position(self.position);
        world.clear_mass_on_position(self.position);

        for i in 0..2 {
            self.position[i] += self.velocity[i];
        }

        world.update_occupation_on_position(self.position, self.particle_ref);
        world.update_mass_on_position(self.position, self.mass());
    }

    pub fn update_velocity(&mut self, gravity: [f32; 2], world: &World) {
        let next_y = self.position[1] + self.velocity[1] + gravity[1];
        let check_y = if next_y < 0.0 { 0.0 } else { next_y };

        if world.give_occupation_on_position(self.position[0] as usize, check_y as usize).is_some() {
            self.velocity[1] = 0.0;
        } else if next_y < 0.0 {
            self.velocity[1] = -self.position[1];
        } else {
            self.velocity[1] += gravity[1];
        }
    }
}

//Struktur Objekt
pub struct Object {
    pub object_id: i32,           // Eindeutige ID für dieses Objekt (pub für ParticleRef)
    pub is_destroyed: bool,       // NEU: Markiert ob Object zerstört wurde
    position: [f32; 2],           // ANKER-Position (linke untere Ecke)
    velocity: [f32; 2],           // Geschwindigkeit des GESAMTEN Objekts
    total_object_mass: f32,       // Gesamtmasse
    object_h: usize,              // Höhe in Zellen (z.B. 3)
    object_w: usize,              // Breite in Zellen (z.B. 3)
    object_grid: Vec<Vec<(Particle, f32, f32)>>,  // Das Mini-Grid
}

impl Object {
    pub fn new(id: i32, object_idx: usize, position: [f32; 2], velocity: [f32; 2], material: MaterialTyp, h: usize, w: usize) -> Object {
        // Grid aufbauen
        let mut object_grid: Vec<Vec<(Particle, f32, f32)>> = Vec::new();

        for i in 0..h {
            let mut row: Vec<(Particle, f32, f32)> = Vec::new();
            for j in 0..w {
                // Jedes Partikel bekommt eigene Position (Anker + Offset)
                let particle_pos = [
                    position[0] + j as f32,
                    position[1] + i as f32,
                ];
                // ParticleRef zeigt auf dieses Object und Position im Grid
                let particle_ref = ParticleRef::InObject(object_idx, i, j);
                let particle = Particle::new(
                    id * 100 + (i * w + j) as i32,
                    particle_pos,
                    [0.0, 0.0],
                    material,
                    particle_ref,
                );
                row.push((particle, 0.0, 0.0));
            }
            object_grid.push(row);
        }
        Object {
            object_id: id,
            is_destroyed: false,  // NEU: Startet als nicht zerstört
            position,
            velocity,
            total_object_mass: (h * w) as f32 * material.density(),
            object_h: h,
            object_w: w,
            object_grid,
        }
    }

    /// Erstellt ein neues Object aus Fragment-Daten.
    ///
    /// fragment_data: Liste von (Welt-Position, Material) für jedes Partikel
    ///
    /// Algorithmus:
    /// 1. Finde Bounding Box (min/max x und y)
    /// 2. Erstelle Grid mit diesen Dimensionen
    /// 3. Platziere Partikel relativ zur Anker-Position (min_x, min_y)
    ///
    /// HINWEIS: Löcher im Fragment werden mit Luft gefüllt (später ignoriert)
    pub fn new_from_fragment(
        id: i32,
        object_idx: usize,
        fragment_data: &[([f32; 2], MaterialTyp)],
        velocity: [f32; 2],
    ) -> Object {
        // Bounding Box berechnen
        let min_x = fragment_data.iter().map(|(pos, _)| pos[0] as usize).min().unwrap();
        let max_x = fragment_data.iter().map(|(pos, _)| pos[0] as usize).max().unwrap();
        let min_y = fragment_data.iter().map(|(pos, _)| pos[1] as usize).min().unwrap();
        let max_y = fragment_data.iter().map(|(pos, _)| pos[1] as usize).max().unwrap();

        let w = max_x - min_x + 1;
        let h = max_y - min_y + 1;
        let anchor_pos = [min_x as f32, min_y as f32];

        // Grid mit Luft initialisieren
        let mut object_grid: Vec<Vec<(Particle, f32, f32)>> = Vec::new();
        let mut total_mass: f32 = 0.0;

        for i in 0..h {
            let mut row: Vec<(Particle, f32, f32)> = Vec::new();
            for j in 0..w {
                let world_x = min_x + j;
                let world_y = min_y + i;

                // Suche ob dieses Partikel im Fragment ist
                let maybe_particle = fragment_data.iter()
                    .find(|(pos, _)| pos[0] as usize == world_x && pos[1] as usize == world_y);

                let material = match maybe_particle {
                    Some((_, mat)) => *mat,
                    None => MaterialTyp::Luft,  // Loch → Luft als Platzhalter
                };

                let particle_pos = [world_x as f32, world_y as f32];
                let particle_ref = ParticleRef::InObject(object_idx, i, j);
                let particle = Particle::new(
                    id * 100 + (i * w + j) as i32,
                    particle_pos,
                    velocity,
                    material,
                    particle_ref,
                );

                if material != MaterialTyp::Luft {
                    total_mass += particle.mass();
                }
                row.push((particle, 0.0, 0.0));
            }
            object_grid.push(row);
        }

        Object {
            object_id: id,
            is_destroyed: false,
            position: anchor_pos,
            velocity,
            total_object_mass: total_mass,
            object_h: h,
            object_w: w,
            object_grid,
        }
    }

    /// Erstellt ein Test-Object mit 2x2 Quadranten aus verschiedenen Materialien.
    ///
    /// Aufbau (4x4):
    /// [Stein]  [Stein]  [Holz]   [Holz]    Reihe 3 (oben)
    /// [Stein]  [Stein]  [Holz]   [Holz]    Reihe 2
    /// [Holz]   [Holz]   [Metall] [Metall]  Reihe 1
    /// [Holz]   [Holz]   [Metall] [Metall]  Reihe 0 (unten)
    ///
    /// Bindungsstärken:
    /// - Holz↔Holz = 40 (normal)
    /// - Metall↔Metall = 200 (sehr stark)
    /// - Stein↔Stein = 150 (stark)
    /// - Holz↔Metall = 40 × 0.5 = 20 (SCHWACH - Übergang!)
    /// - Holz↔Stein = 40 × 0.5 = 20 (SCHWACH - Übergang!)
    ///
    /// Bei Impact sollten zuerst die Übergänge brechen!
    pub fn new_quadrant(id: i32, object_idx: usize, position: [f32; 2], velocity: [f32; 2]) -> Object {
        let h = 4;
        let w = 4;
        let mut object_grid: Vec<Vec<(Particle, f32, f32)>> = Vec::new();
        let mut total_mass: f32 = 0.0;

        for i in 0..h {
            let mut row: Vec<(Particle, f32, f32)> = Vec::new();

            for j in 0..w {
                // Material je nach Quadrant (2x2 Bereiche)
                let material = if i < 2 && j < 2 {
                    MaterialTyp::Holz    // Unten links: Holz
                } else if i < 2 && j >= 2 {
                    MaterialTyp::Metall  // Unten rechts: Metall
                } else if i >= 2 && j < 2 {
                    MaterialTyp::Stein   // Oben links: Stein
                } else {
                    MaterialTyp::Holz    // Oben rechts: Holz
                };

                let particle_pos = [
                    position[0] + j as f32,
                    position[1] + i as f32,
                ];
                let particle_ref = ParticleRef::InObject(object_idx, i, j);
                let particle = Particle::new(
                    id * 100 + (i * w + j) as i32,
                    particle_pos,
                    [0.0, 0.0],
                    material,
                    particle_ref,
                );
                total_mass += particle.mass();
                row.push((particle, 0.0, 0.0));
            }
            object_grid.push(row);
        }

        Object {
            object_id: id,
            is_destroyed: false,
            position,
            velocity,
            total_object_mass: total_mass,
            object_h: h,
            object_w: w,
            object_grid,
        }
    }

    pub fn update_object_position(&mut self, world: &mut World) {
        // Alte Positionen clearen (nur echte Partikel, nicht Luft)
        for i in 0..self.object_h {
            for j in 0..self.object_w {
                let particle = &self.object_grid[i][j].0;
                if particle.material != MaterialTyp::Luft {
                    world.clear_occupation_on_position(particle.position);
                    world.clear_mass_on_position(particle.position);
                }
            }
        }

        // Object-Position updaten
        self.position[0] += self.velocity[0];
        self.position[1] += self.velocity[1];

        // Partikel-Positionen neu berechnen + World updaten (nur echte Partikel)
        for i in 0..self.object_h {
            for j in 0..self.object_w {
                let particle = &mut self.object_grid[i][j].0;
                particle.position[0] = self.position[0] + j as f32;
                particle.position[1] = self.position[1] + i as f32;

                if particle.material != MaterialTyp::Luft {
                    world.update_occupation_on_position(particle.position, particle.particle_ref);
                    world.update_mass_on_position(particle.position, particle.mass());
                }
            }
        }
    }

    pub fn get_object_elements(&self) -> Vec<&Particle> {
        let mut particles = Vec::new();
        for i in 0..self.object_h {
            for j in 0..self.object_w {
                particles.push(&self.object_grid[i][j].0);
            }
        }
        particles
    }

    pub fn get_object_velocity(&self) -> [f32; 2] {
        self.velocity
    }

    /// Berechnet die Impact-Kraft bei Aufprall.
    ///
    /// Physik: Impuls = Masse × Geschwindigkeit
    /// Bei Aufprall ändert sich die Geschwindigkeit von v auf 0.
    /// Die Kraft die dabei wirkt ist proportional zu: Masse × |Geschwindigkeit|
    ///
    /// Vereinfachung: Wir ignorieren die Zeitkomponente (F = dp/dt),
    /// da wir diskrete Zeitschritte haben. Der "impact_force" Wert ist
    /// eigentlich ein Impuls, aber für den Vergleich mit binding_strength reicht das.
    pub fn calc_impact_force(&self, velocity_before_impact: f32) -> f32 {
        // Impact-Kraft = Gesamtmasse × Betrag der Geschwindigkeit
        self.total_object_mass * velocity_before_impact.abs()
    }

    /// Berechnet den Dämpfungsfaktor basierend auf den Kollisions-Materialien.
    ///
    /// Logik:
    /// - Static (Boden/Wand): Hart, keine Dämpfung → 1.0
    /// - Free (Partikel): Weich, dämpft → 0.4
    /// - InObject (anderes Object): Mittel → 0.6
    ///
    /// Bei mehreren Kollisionen: Durchschnitt
    fn calc_dampening_factor(collisions: &[ParticleRef]) -> f32 {
        if collisions.is_empty() {
            return 1.0;
        }

        let sum: f32 = collisions.iter().map(|c| {
            match c {
                ParticleRef::Static => 1.0,       // Boden ist hart
                ParticleRef::Free(_) => 0.4,      // Partikel dämpfen stark
                ParticleRef::InObject(_, _, _) => 0.6,  // Objects dämpfen mittel
            }
        }).sum();

        sum / collisions.len() as f32
    }

    /// Prüft welche Bindungen bei gegebener Impact-Kraft brechen würden.
    ///
    /// Logik:
    /// 1. Iteriere durch alle NACHBARPAARE im Grid (horizontal + vertikal)
    /// 2. Bindungsstärke berechnen:
    ///    - GLEICHES Material: volle Stärke des Materials
    ///    - VERSCHIEDENE Materialien: Übergang = Schwachstelle! 
    ///      Nur 50% der schwächeren Bindung
    /// 3. Die Kraft wird nach Reihe verteilt:
    ///    - Reihe 0 (unten): volle Kraft
    ///    - Reihe 1: halbe Kraft
    ///    - Reihe 2: drittel Kraft
    /// 4. Wenn Kraft an dieser Stelle > Bindungsstärke → Bindung bricht
    ///
    /// dampening_factor: Dämpfung durch das Material auf das aufgeprallt wurde
    ///
    /// Rückgabe: Liste von gebrochenen Bindungen als ((i1,j1), (i2,j2)) Paare
    pub fn check_fracture(&self, impact_force: f32, dampening_factor: f32) -> Vec<((usize, usize), (usize, usize))> {
        let mut broken_bonds: Vec<((usize, usize), (usize, usize))> = Vec::new();

        // Gedämpfte Basis-Kraft
        let base_force = impact_force * dampening_factor;

        // Durch alle Zellen im Grid iterieren
        for i in 0..self.object_h {
            for j in 0..self.object_w {
                let particle_a = &self.object_grid[i][j].0;
                let mat_a = particle_a.material;

                // Luft-Partikel haben keine Bindungen - überspringen
                if mat_a == MaterialTyp::Luft {
                    continue;
                }

                // Kraft an dieser Reihe: Je höher (größeres i), desto weniger Kraft
                // Reihe 0 → 1.0, Reihe 1 → 0.5, Reihe 2 → 0.33, etc.
                let row_factor = 1.0 / (i as f32 + 1.0);
                let force_at_row = base_force * row_factor;

                // Nachbar RECHTS prüfen (wenn vorhanden und nicht Luft)
                if j + 1 < self.object_w {
                    let particle_b = &self.object_grid[i][j + 1].0;
                    let mat_b = particle_b.material;

                    // Keine Bindung zu Luft
                    if mat_b == MaterialTyp::Luft {
                        continue;
                    }

                    // Bindungsstärke berechnen
                    let bond_strength = Self::calc_bond_strength(mat_a, mat_b);

                    if force_at_row > bond_strength {
                        broken_bonds.push(((i, j), (i, j + 1)));
                    }
                }

                // Nachbar OBEN prüfen (wenn vorhanden und nicht Luft)
                if i + 1 < self.object_h {
                    let particle_b = &self.object_grid[i + 1][j].0;
                    let mat_b = particle_b.material;

                    // Keine Bindung zu Luft
                    if mat_b == MaterialTyp::Luft {
                        continue;
                    }

                    // Bindungsstärke berechnen
                    let bond_strength = Self::calc_bond_strength(mat_a, mat_b);

                    if force_at_row > bond_strength {
                        broken_bonds.push(((i, j), (i + 1, j)));
                    }
                }
            }
        }

        broken_bonds
    }

    /// Berechnet die Bindungsstärke zwischen zwei Materialien.
    ///
    /// Regel:
    /// - Gleiches Material → volle binding_strength
    /// - Verschiedene Materialien → Übergang ist SCHWACHSTELLE
    ///   Nur 50% des schwächeren Materials
    ///
    /// Beispiele:
    /// - Holz↔Holz = 40 (stark für Holz)
    /// - Metall↔Metall = 200 (stark für Metall)
    /// - Holz↔Metall = 40 × 0.5 = 20 (schwach! Übergang)
    /// - Stein↔Holz = 40 × 0.5 = 20 (schwach! Übergang)
    fn calc_bond_strength(mat_a: MaterialTyp, mat_b: MaterialTyp) -> f32 {
        let strength_a = mat_a.binding_strength();
        let strength_b = mat_b.binding_strength();

        if mat_a == mat_b {
            // Gleiches Material: volle Stärke
            strength_a
        } else {
            // Übergang: Schwachstelle! 50% des schwächeren
            strength_a.min(strength_b) * 0.5
        }
    }

    /// Berechnet den Druck der auf jeder Spalte von oben lastet.
    ///
    /// Für jede Spalte j:
    /// 1. Finde die oberste Position des Objects
    /// 2. Schaue was direkt darüber im World-Grid liegt
    /// 3. Summiere die Masse aller Zellen darüber
    ///
    /// Rückgabe: Vec<f32> mit Druck pro Spalte
    fn calc_pressure_per_column(&self, world: &World) -> Vec<f32> {
        let mut pressure_per_col = vec![0.0; self.object_w];

        for j in 0..self.object_w {
            // Oberste Position dieser Spalte im World-Grid
            let top_row = self.object_h - 1;
            let world_x = self.position[0] as usize + j;
            let world_y = self.position[1] as usize + top_row;

            // Bounds check
            if world_x >= world.width || world_y >= world.height {
                continue;
            }

            // Alles darüber summieren
            let mut total_pressure = 0.0;
            for y in (world_y + 1)..world.height {
                let mass_at_pos = world.grid[y][world_x].1;
                if mass_at_pos > 0.0 {
                    total_pressure += mass_at_pos;
                }
            }

            pressure_per_col[j] = total_pressure;
        }

        // DEBUG: Wenn hoher Druck, zeige Info
        let max_pressure: f32 = pressure_per_col.iter().cloned().fold(0.0, f32::max);
        if max_pressure > 50.0 {
            println!("Object {} bei Y={}: Druck {:?}", self.object_id, self.position[1], pressure_per_col);
        }

        pressure_per_col
    }

    /// Prüft welche Bindungen durch kontinuierlichen Druck brechen.
    ///
    /// Anders als Impact-Bruch:
    /// - Druck kommt von OBEN (nicht von unten beim Aufprall)
    /// - Druck akkumuliert sich nach unten (obere Reihen tragen weniger)
    /// - Nur VERTIKALE Bindungen werden geprüft (Druck wirkt vertikal)
    ///
    /// Algorithmus:
    /// 1. Berechne externen Druck pro Spalte (was liegt drauf?)
    /// 2. Für jede Reihe von oben nach unten:
    ///    - Akkumuliere Druck (externer + Masse der Reihen darüber)
    ///    - Prüfe vertikale Bindung zur Reihe darunter
    ///
    /// Rückgabe: Liste von gebrochenen Bindungen
    pub fn check_pressure_fracture(&self, world: &World) -> Vec<((usize, usize), (usize, usize))> {
        let mut broken_bonds: Vec<((usize, usize), (usize, usize))> = Vec::new();

        // Externer Druck pro Spalte
        let external_pressure = self.calc_pressure_per_column(world);

        // Für jede Spalte: Druck von oben nach unten propagieren
        for j in 0..self.object_w {
            // Starte mit externem Druck
            let mut accumulated_pressure = external_pressure[j];

            // Von oben nach unten durch die Reihen
            for i in (0..self.object_h).rev() {
                let particle = &self.object_grid[i][j].0;

                // Luft ignorieren
                if particle.material == MaterialTyp::Luft {
                    continue;
                }

                // Prüfe VERTIKALE Bindung zur Reihe DARUNTER (wenn vorhanden)
                if i > 0 {
                    let particle_below = &self.object_grid[i - 1][j].0;

                    if particle_below.material != MaterialTyp::Luft {
                        let bond_strength = Self::calc_bond_strength(
                            particle.material,
                            particle_below.material
                        );

                        if accumulated_pressure > bond_strength {
                            broken_bonds.push(((i - 1, j), (i, j)));
                        }
                    }
                }

                // Prüfe HORIZONTALE Bindung nach RECHTS (wenn vorhanden)
                // Der gleiche Druck wirkt auch auf horizontale Bindungen!
                if j + 1 < self.object_w {
                    let particle_right = &self.object_grid[i][j + 1].0;

                    if particle_right.material != MaterialTyp::Luft {
                        let bond_strength = Self::calc_bond_strength(
                            particle.material,
                            particle_right.material
                        );

                        if accumulated_pressure > bond_strength {
                            broken_bonds.push(((i, j), (i, j + 1)));
                        }
                    }
                }

                // Masse dieser Zelle zum akkumulierten Druck addieren
                accumulated_pressure += particle.mass();
            }
        }

        broken_bonds
    }

    /// Findet zusammenhängende Fragmente nach Bruch.
    ///
    /// Algorithmus (Union-Find):
    /// 1. Jedes Partikel startet als eigene Gruppe
    /// 2. Sammle ALLE möglichen Bindungen im Grid
    /// 3. Entferne die gebrochenen Bindungen
    /// 4. Für jede übrige Bindung: Verbinde die beiden Partikel zu einer Gruppe
    /// 5. Sammle alle Partikel nach ihrer Gruppe
    ///
    /// Rückgabe: Liste von Fragmenten, jedes Fragment ist Liste von (i,j) Koordinaten
    ///
    /// Beispiel: 3x3 Grid, alle horizontalen Bindungen gebrochen
    /// → 3 Fragmente: [(0,0),(1,0),(2,0)], [(0,1),(1,1),(2,1)], [(0,2),(1,2),(2,2)]
    pub fn find_fragments(&self, broken_bonds: &[((usize, usize), (usize, usize))]) -> Vec<Vec<(usize, usize)>> {
        // Hilfsfunktion: (i,j) → eindeutiger Index
        let to_index = |i: usize, j: usize| -> usize { i * self.object_w + j };

        // Union-Find Datenstruktur: parent[i] = Eltern-Index von i
        // Wenn parent[i] == i, dann ist i eine Wurzel
        let total = self.object_h * self.object_w;
        let mut parent: Vec<usize> = (0..total).collect();

        // Find mit Pfadkompression: Finde die Wurzel einer Gruppe
        fn find(parent: &mut Vec<usize>, mut x: usize) -> usize {
            while parent[x] != x {
                parent[x] = parent[parent[x]]; // Pfadkompression
                x = parent[x];
            }
            x
        }

        // Union: Verbinde zwei Gruppen
        fn union(parent: &mut Vec<usize>, a: usize, b: usize) {
            let root_a = find(parent, a);
            let root_b = find(parent, b);
            if root_a != root_b {
                parent[root_a] = root_b;
            }
        }

        // Sammle ALLE möglichen Bindungen (aber nur zwischen echten Partikeln, nicht Luft)
        let mut all_bonds: Vec<((usize, usize), (usize, usize))> = Vec::new();
        for i in 0..self.object_h {
            for j in 0..self.object_w {
                let mat_a = self.object_grid[i][j].0.material;
                if mat_a == MaterialTyp::Luft {
                    continue; // Luft hat keine Bindungen
                }

                // Nachbar rechts
                if j + 1 < self.object_w {
                    let mat_b = self.object_grid[i][j + 1].0.material;
                    if mat_b != MaterialTyp::Luft {
                        all_bonds.push(((i, j), (i, j + 1)));
                    }
                }
                // Nachbar oben
                if i + 1 < self.object_h {
                    let mat_b = self.object_grid[i + 1][j].0.material;
                    if mat_b != MaterialTyp::Luft {
                        all_bonds.push(((i, j), (i + 1, j)));
                    }
                }
            }
        }

        // Verbinde alle NICHT gebrochenen Bindungen
        for bond in &all_bonds {
            if !broken_bonds.contains(bond) {
                let idx_a = to_index(bond.0.0, bond.0.1);
                let idx_b = to_index(bond.1.0, bond.1.1);
                union(&mut parent, idx_a, idx_b);
            }
        }

        // Sammle Partikel nach ihrer Wurzel (= Fragment) - NUR echte Partikel, nicht Luft
        use std::collections::HashMap;
        let mut fragments_map: HashMap<usize, Vec<(usize, usize)>> = HashMap::new();

        for i in 0..self.object_h {
            for j in 0..self.object_w {
                let mat = self.object_grid[i][j].0.material;
                if mat == MaterialTyp::Luft {
                    continue; // Luft gehört zu keinem Fragment
                }
                let idx = to_index(i, j);
                let root = find(&mut parent, idx);
                fragments_map.entry(root).or_insert_with(Vec::new).push((i, j));
            }
        }

        // HashMap zu Vec konvertieren
        fragments_map.into_values().collect()
    }

    /// Aktualisiert die Velocity des Objects basierend auf Gravitation.
    /// Prüft die GESAMTE untere Kante auf Kollision, nicht nur den Anker.
    /// Bei Impact: Berechnet Kraft, prüft Bruch, findet Fragmente.
    ///
    /// Rückgabe: 
    /// - None: Kein Impact oder keine Brüche
    /// - Some(fragments): Liste der Fragmente (jedes Fragment = Liste von (i,j) Koordinaten)
    pub fn update_object_velocity(&mut self, gravity: [f32; 2], world: &World) -> Option<Vec<Vec<(usize, usize)>>> {
        // Berechne wo die Unterseite im nächsten Frame wäre
        let next_y = self.position[1] + self.velocity[1] + gravity[1];
        let check_y = if next_y < 0.0 { 0.0 } else { next_y };

        // Sammle alle Kollisionen der Unterkante
        let mut collisions: Vec<ParticleRef> = Vec::new();

        for j in 0..self.object_w {
            let check_x = (self.position[0] + j as f32) as usize;

            if let Some(particle_ref) = world.give_occupation_on_position(check_x, check_y as usize) {
                collisions.push(particle_ref);
            }
        }

        // Bei Kollision: Impact-Kraft berechnen und Bruch prüfen
        if !collisions.is_empty() {
            // Velocity VOR dem Nullsetzen speichern
            let velocity_before = self.velocity[1];

            // Velocity auf 0 setzen (Object stoppt)
            self.velocity[1] = 0.0;

            // Nur bei ECHTEM Impact (Object war in Bewegung)
            if velocity_before != 0.0 {
                // Impact-Kraft berechnen
                let impact_force = self.calc_impact_force(velocity_before);

                // Dämpfungsfaktor berechnen basierend auf Kollisions-Materialien
                // Static (Boden) = hart (1.0), Partikel/Objects = weicher (0.4)
                // Bei gemischten Kollisionen: Durchschnitt
                let dampening = Self::calc_dampening_factor(&collisions);

                // Prüfen welche Bindungen brechen (mit Dämpfung)
                let broken_bonds = self.check_fracture(impact_force, dampening);

                if broken_bonds.is_empty() {
                    return None;
                }

                // Fragmente finden
                let fragments = self.find_fragments(&broken_bonds);

                return Some(fragments);
            }

        } else if next_y < 0.0 {
            self.velocity[1] = -self.position[1];
        } else {
            self.velocity[1] += gravity[1];
        }

        None
    }

    pub fn calc_object_mass(&mut self) -> f32 {
        let mut sum_mass: f32 = 0.0;
        for i in 0..self.object_h {
            for j in 0..self.object_w {
                let particle: &Particle = &self.object_grid[i][j].0;
                sum_mass += particle.mass();
            }
        }
        self.total_object_mass = sum_mass;
        sum_mass
    }

    pub fn get_object_mass(&self) -> f32 {
        self.total_object_mass
    }

    /// Gibt Höhe des Object-Grids zurück
    pub fn get_height(&self) -> usize {
        self.object_h
    }

    /// Gibt Breite des Object-Grids zurück
    pub fn get_width(&self) -> usize {
        self.object_w
    }

    /// Gibt Anker-Position zurück
    pub fn get_position(&self) -> [f32; 2] {
        self.position
    }

    /// Gibt Partikel an Grid-Position (i,j) zurück
    pub fn get_particle_at(&self, i: usize, j: usize) -> &Particle {
        &self.object_grid[i][j].0
    }

    /// Gibt Object-ID zurück
    pub fn get_id(&self) -> i32 {
        self.object_id
    }

    /// Entfernt alle Partikel des Objects aus dem World-Grid.
    /// Wird bei Zerstörung aufgerufen.
    pub fn clear_from_world(&self, world: &mut World) {
        for i in 0..self.object_h {
            for j in 0..self.object_w {
                let particle = &self.object_grid[i][j].0;
                world.clear_occupation_on_position(particle.position);
                world.clear_mass_on_position(particle.position);
            }
        }
    }

    /// Extrahiert Partikel-Daten für ein Fragment.
    /// Gibt für jede Grid-Position im Fragment zurück: (Welt-Position, Material)
    pub fn extract_fragment_data(&self, fragment: &[(usize, usize)]) -> Vec<([f32; 2], MaterialTyp)> {
        fragment.iter().map(|(i, j)| {
            let particle = &self.object_grid[*i][*j].0;
            (particle.position, particle.material)
        }).collect()
    }
}

//Struktur Welt
pub struct World {
    pub height: usize,
    pub width: usize,
    pub grid: Vec<Vec<(Option<ParticleRef>, f32, f32)>>,
}

impl World {
    pub fn new(h: usize, w: usize) -> World {
        World {
            height: h,
            width: w,
            grid: vec![vec![(None, 0.0, 0.0); w]; h],
        }
    }

    pub fn give_world(&self) {
        for i in 0..self.height {
            for j in 0..self.width {
                println!(
                    "An X_{}/Y_{}:Occupation:{:?}/Masse:{}/Druck:{}",
                    j, i, self.grid[i][j].0, self.grid[i][j].1, self.grid[i][j].2
                );
            }
        }
    }

    pub fn give_pressure_on_position(&self, x: usize, y: usize) -> f32 {
        self.grid[y][x].2
    }

    pub fn give_occupation_on_position(&self, x: usize, y: usize) -> Option<ParticleRef> {
        self.grid[y][x].0
    }

    pub fn update_mass_on_position(&mut self, partl_koord: [f32; 2], mass: f32) {
        let x = partl_koord[0] as usize;
        let y = partl_koord[1] as usize;
        self.grid[y][x].1 = mass;
    }

    pub fn update_occupation_on_position(&mut self, partl_koord: [f32; 2], particle_ref: ParticleRef) {
        let x = partl_koord[0] as usize;
        let y = partl_koord[1] as usize;
        self.grid[y][x].0 = Some(particle_ref);
    }

    pub fn clear_occupation_on_position(&mut self, partl_koord: [f32; 2]) {
        let x = partl_koord[0] as usize;
        let y = partl_koord[1] as usize;
        self.grid[y][x].0 = None;
    }

    pub fn clear_mass_on_position(&mut self, partl_koord: [f32; 2]) {
        let x = partl_koord[0] as usize;
        let y = partl_koord[1] as usize;
        self.grid[y][x].1 = 0.0;
    }

    pub fn calc_pressure_on_all_position(&mut self) {
        for j in 0..self.width {
            let mut sum_pressure: f32 = 0.0;
            for i in (0..self.height).rev() {
                sum_pressure += self.grid[i][j].1;
                self.grid[i][j].2 = sum_pressure;
            }
        }
    }
}