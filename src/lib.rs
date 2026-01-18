use rand::seq::SliceRandom;

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
    pub material : MaterialTyp
}
impl Particle {
    pub fn new(id: i32, position: [f32; 2], velocity: [f32; 2], material : MaterialTyp) -> Particle {
        println!("Erschaffe neues Partikel");
        Particle {
            id : id,
            position : position,
            velocity : velocity,
            material: material,
        }
    }
    pub fn mass(&self) -> f32{
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
                if !world.give_occupation_on_position(target_x as usize, target_y as usize) {
                    world.clear_occupation_on_position(self.position);
                    world.clear_mass_on_position(self.position);
                    self.position[0] = target_x as f32;
                    self.position[1] = target_y as f32;
                    world.update_occupation_on_position(self.position);
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
        if !world.give_occupation_on_position(x as usize, (y - 1) as usize) {
            world.clear_occupation_on_position(self.position);
            world.clear_mass_on_position(self.position);
            self.position[1] -= 1.0;
            world.update_occupation_on_position(self.position);
            world.update_mass_on_position(self.position, self.mass());
            return;
        }

        // Diagonal links unten frei?
        if x > 0 && !world.give_occupation_on_position((x - 1) as usize, (y - 1) as usize) {
            world.clear_occupation_on_position(self.position);
            world.clear_mass_on_position(self.position);
            self.position[0] -= 1.0;
            self.position[1] -= 1.0;
            world.update_occupation_on_position(self.position);
            world.update_mass_on_position(self.position, self.mass());
            return;
        }

        // Diagonal rechts unten frei?
        if x < (world.width - 1) as i32 && !world.give_occupation_on_position((x + 1) as usize, (y - 1) as usize) {
            world.clear_occupation_on_position(self.position);
            world.clear_mass_on_position(self.position);
            self.position[0] += 1.0;
            self.position[1] -= 1.0;
            world.update_occupation_on_position(self.position);
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

        world.update_occupation_on_position(self.position);
        world.update_mass_on_position(self.position, self.mass());
    }

    pub fn update_velocity(&mut self, gravity: [f32; 2], world: &World) {
        let next_y = self.position[1] + self.velocity[1] + gravity[1];
        let check_y = if next_y < 0.0 { 0.0 } else { next_y };

        if world.give_occupation_on_position(self.position[0] as usize, check_y as usize) {
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
    object_id: i32,           // Eindeutige ID für dieses Objekt
    position: [f32; 2],       // ANKER-Position (linke untere Ecke)
    velocity: [f32; 2],       // Geschwindigkeit des GESAMTEN Objekts
    total_object_mass: f32,   // Gesamtmasse
    object_h: usize,          // Höhe in Zellen (z.B. 3)
    object_w: usize,          // Breite in Zellen (z.B. 3)
    object_grid: Vec<Vec<(Particle, f32, f32)>>,  // Das Mini-Grid
}

impl Object {
    pub fn new(id: i32, position: [f32; 2], velocity: [f32; 2], material: MaterialTyp, h: usize, w: usize) -> Object {
        println!("Erschaffe neues Objekt");

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
                let particle = Particle::new(
                    id * 100 + (i * w + j) as i32,  // id
                    particle_pos,                     // position
                    [0.0, 0.0],                       // velocity (explizit)
                    material,           // mass
                );
                row.push((particle, 0.0, 0.0));
            }
            object_grid.push(row);
        }
        Object {
            object_id: id,
            position,
            velocity,
            total_object_mass: (h * w) as f32 * material.density(),
            object_h: h,
            object_w: w,
            object_grid,
        }
    }
    pub fn update_object_position(&mut self, world: &mut World) {
        // Alte Positionen clearen
        for i in 0..self.object_h {
            for j in 0..self.object_w {
                let particle = &self.object_grid[i][j].0;
                world.clear_occupation_on_position(particle.position);
                world.clear_mass_on_position(particle.position);
            }
        }

        // Object-Position updaten
        self.position[0] += self.velocity[0];
        self.position[1] += self.velocity[1];

        // Partikel-Positionen neu berechnen + World updaten
        for i in 0..self.object_h {
            for j in 0..self.object_w {
                let particle = &mut self.object_grid[i][j].0;
                particle.position[0] = self.position[0] + j as f32;
                particle.position[1] = self.position[1] + i as f32;

                world.update_occupation_on_position(particle.position);
                world.update_mass_on_position(particle.position, particle.mass());
            }
        }
    }
    pub fn get_object_elements(&self) -> Vec<&Particle> {
        let mut particles = Vec::new();
        for i in 0..self.object_h {
            for j in 0..self.object_w {
                particles.push(&self.object_grid[i][j].0);
            }
        }particles
    }
    pub fn get_object_velocity(&self) -> [f32; 2] {
        self.velocity
    }
    pub fn update_object_velocity(&mut self, gravity: [f32; 2], world: &World) {
        let next_y = self.position[1] + self.velocity[1] + gravity[1];
        let check_y = if next_y < 0.0 { 0.0 } else { next_y };

        if world.give_occupation_on_position(self.position[0] as usize, check_y as usize) {
            self.velocity[1] = 0.0;
        } else if next_y < 0.0 {
            self.velocity[1] = -self.position[1];
        } else {
            self.velocity[1] += gravity[1];
        }
    }

    pub fn calc_object_mass(&mut self) -> f32{
        let mut sum_mass:f32 = 0.0;
        for i in 0..self.object_h {
            for j in 0..self.object_w {
                let particle : &Particle = &self.object_grid[i][j].0;
                sum_mass += particle.mass();
            }
        } self.total_object_mass = sum_mass;
        sum_mass
    }
    pub fn get_object_mass(&self) -> f32{
        self.total_object_mass
    }
}

//Struktur Welt
pub struct World {
    pub height: usize,
    pub width: usize,
    pub grid: Vec<Vec<(bool, f32, f32)>>,
}

impl World {
    pub fn new(h: usize, w: usize) -> World {
        World {
            height: h,
            width: w,
            grid: vec![vec![(false, 0.0, 0.0); w]; h],
        }
    }

    pub fn give_world(&self) {
        for i in 0..self.height {
            for j in 0..self.width {
                println!(
                    "An X_{}/Y_{}:Occupation:{}/Masse:{}/Druck:{}",
                    j, i, self.grid[i][j].0, self.grid[i][j].1, self.grid[i][j].2
                );
            }
        }
    }

    pub fn give_pressure_on_position(&self, x: usize, y: usize) -> f32 {
        self.grid[y][x].2
    }

    pub fn give_occupation_on_position(&self, x: usize, y: usize) -> bool {
        self.grid[y][x].0
    }

    pub fn update_mass_on_position(&mut self, partl_koord: [f32; 2], mass: f32) {
        let x = partl_koord[0] as usize;
        let y = partl_koord[1] as usize;
        self.grid[y][x].1 = mass;
    }

    pub fn update_occupation_on_position(&mut self, partl_koord: [f32; 2]) {
        let x = partl_koord[0] as usize;
        let y = partl_koord[1] as usize;
        self.grid[y][x].0 = true;
    }

    pub fn clear_occupation_on_position(&mut self, partl_koord: [f32; 2]) {
        let x = partl_koord[0] as usize;
        let y = partl_koord[1] as usize;
        self.grid[y][x].0 = false;
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
