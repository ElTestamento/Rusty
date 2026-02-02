use rand::seq::SliceRandom;

/// Referenz auf ein Partikel im World-Grid.
#[derive(Debug, Clone, Copy)]
pub enum ParticleRef {
    Free(usize),
    InObject(usize, usize, usize),
    Static,
}

/// Materialtypen für Partikel.
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
    pub fn binding_strength(&self) -> f32 {
        match self {
            MaterialTyp::Sand => 2.0,
            MaterialTyp::Stein => 80.0,
            MaterialTyp::Metall => 200.0,
            MaterialTyp::Luft => 0.0,
            MaterialTyp::Wasser => 0.0,
            MaterialTyp::Holz => 40.0,
        }
    }

    pub fn density(&self) -> f32 {
        match self {
            MaterialTyp::Sand => 1.5,
            MaterialTyp::Stein => 2.5,
            MaterialTyp::Metall => 8.0,
            MaterialTyp::Luft => 0.001,
            MaterialTyp::Wasser => 1.0,
            MaterialTyp::Holz => 0.6,
        }
    }

    pub fn is_solid(&self) -> bool {
        match self {
            MaterialTyp::Luft | MaterialTyp::Wasser => false,
            _ => true,
        }
    }

    pub fn impact_dampening(&self) -> f32 {
        match self {
            MaterialTyp::Sand => 0.3,
            MaterialTyp::Stein => 1.0,
            MaterialTyp::Metall => 0.9,
            MaterialTyp::Luft => 0.0,
            MaterialTyp::Wasser => 0.2,
            MaterialTyp::Holz => 0.6,
        }
    }

    pub fn color(&self) -> (f32, f32, f32) {
        match self {
            MaterialTyp::Sand => (0.9, 0.75, 0.4),
            MaterialTyp::Stein => (0.5, 0.5, 0.5),
            MaterialTyp::Metall => (0.7, 0.75, 0.8),
            MaterialTyp::Luft => (0.9, 0.95, 1.0),
            MaterialTyp::Wasser => (0.2, 0.5, 0.8),
            MaterialTyp::Holz => (0.55, 0.35, 0.15),
        }
    }
}

// ============== PARTICLE ==============

#[derive(Debug, Clone)]
pub struct Particle {
    pub id: i32,
    pub position: [f32; 2],
    pub velocity: [f32; 2],
    pub material: MaterialTyp,
    pub particle_ref: ParticleRef,
}

impl Particle {
    pub fn new(id: i32, position: [f32; 2], velocity: [f32; 2], material: MaterialTyp, particle_ref: ParticleRef) -> Particle {
        Particle { id, position, velocity, material, particle_ref }
    }

    pub fn mass(&self) -> f32 {
        self.material.density()
    }

    fn check_way(&self, world: &World) -> Option<(f32, i32, i32)> {
        let own_x_pos = self.position[0] as i32;
        let own_y_pos = self.position[1] as i32;

        let can_go_down = own_y_pos > 0;
        let can_go_up = own_y_pos < (world.height - 1) as i32;
        let can_go_left = own_x_pos > 0;
        let can_go_right = own_x_pos < (world.width - 1) as i32;

        let mut values: Vec<(f32, i32, i32)> = vec![];

        if can_go_right && can_go_down {
            let pressure = world.grid[(own_y_pos - 1) as usize][(own_x_pos + 1) as usize].2;
            values.push((pressure, own_x_pos + 1, own_y_pos - 1));
        }
        if can_go_right {
            let pressure = world.grid[(own_y_pos) as usize][(own_x_pos + 1) as usize].2;
            values.push((pressure, own_x_pos + 1, own_y_pos));
        }
        if can_go_up && can_go_right {
            let pressure = world.grid[(own_y_pos + 1) as usize][(own_x_pos + 1) as usize].2;
            values.push((pressure, own_x_pos + 1, own_y_pos + 1));
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
        let min_options: Vec<_> = values.iter().filter(|v| v.0 == min_pressure).collect();

        match min_options.choose(&mut rand::thread_rng()) {
            Some(&&(pressure, x, y)) => Some((pressure, x, y)),
            None => None,
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

        if world.give_occupation_on_position(x as usize, (y - 1) as usize).is_none() {
            world.clear_occupation_on_position(self.position);
            world.clear_mass_on_position(self.position);
            self.position[1] -= 1.0;
            world.update_occupation_on_position(self.position, self.particle_ref);
            world.update_mass_on_position(self.position, self.mass());
            return;
        }

        if x > 0 && world.give_occupation_on_position((x - 1) as usize, (y - 1) as usize).is_none() {
            world.clear_occupation_on_position(self.position);
            world.clear_mass_on_position(self.position);
            self.position[0] -= 1.0;
            self.position[1] -= 1.0;
            world.update_occupation_on_position(self.position, self.particle_ref);
            world.update_mass_on_position(self.position, self.mass());
            return;
        }

        if x < (world.width - 1) as i32 && world.give_occupation_on_position((x + 1) as usize, (y - 1) as usize).is_none() {
            world.clear_occupation_on_position(self.position);
            world.clear_mass_on_position(self.position);
            self.position[0] += 1.0;
            self.position[1] -= 1.0;
            world.update_occupation_on_position(self.position, self.particle_ref);
            world.update_mass_on_position(self.position, self.mass());
        }
    }

    /// Flüssigkeiten breiten sich seitlich aus wenn sie nicht fallen können
    pub fn flow_sideways(&mut self, world: &mut World) {
        // Nur für Flüssigkeiten (Wasser)
        if self.material.is_solid() {
            return;
        }

        let x = self.position[0] as i32;
        let y = self.position[1] as i32;
        let w = world.width as i32;

        // Nur fließen wenn unten blockiert ist
        if y > 0 && world.give_occupation_on_position(x as usize, (y - 1) as usize).is_none() {
            return; // Kann fallen, also nicht seitlich fließen
        }

        let can_left = x > 0 && world.give_occupation_on_position((x - 1) as usize, y as usize).is_none();
        let can_right = x < w - 1 && world.give_occupation_on_position((x + 1) as usize, y as usize).is_none();

        if !can_left && !can_right {
            return;
        }

        // Bevorzuge Seite mit niedrigerem Druck
        let pressure_left = if can_left { world.give_pressure_on_position((x - 1) as usize, y as usize) } else { f32::MAX };
        let pressure_right = if can_right { world.give_pressure_on_position((x + 1) as usize, y as usize) } else { f32::MAX };

        let go_left = if can_left && can_right {
            if pressure_left < pressure_right {
                true
            } else if pressure_right < pressure_left {
                false
            } else {
                rand::random::<bool>() // Zufällig wenn gleich
            }
        } else {
            can_left
        };

        world.clear_occupation_on_position(self.position);
        world.clear_mass_on_position(self.position);

        if go_left {
            self.position[0] -= 1.0;
        } else {
            self.position[0] += 1.0;
        }

        world.update_occupation_on_position(self.position, self.particle_ref);
        world.update_mass_on_position(self.position, self.mass());
    }

    pub fn get_position(&self) -> [f32; 2] {
        self.position
    }

    pub fn get_velocity(&self) -> [f32; 2] {
        self.velocity
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

// ============== OBJECT ==============

pub struct Object {
    pub object_id: i32,
    pub is_destroyed: bool,
    position: [f32; 2],
    velocity: [f32; 2],
    total_object_mass: f32,
    object_h: usize,
    object_w: usize,
    object_grid: Vec<Vec<(Particle, f32, f32)>>,
}

impl Object {
    pub fn new(id: i32, object_idx: usize, position: [f32; 2], velocity: [f32; 2], material: MaterialTyp, h: usize, w: usize) -> Object {
        let mut object_grid: Vec<Vec<(Particle, f32, f32)>> = Vec::new();

        for i in 0..h {
            let mut row: Vec<(Particle, f32, f32)> = Vec::new();
            for j in 0..w {
                let particle_pos = [position[0] + j as f32, position[1] + i as f32];
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
            is_destroyed: false,
            position,
            velocity,
            total_object_mass: (h * w) as f32 * material.density(),
            object_h: h,
            object_w: w,
            object_grid,
        }
    }

    pub fn new_from_fragment(id: i32, object_idx: usize, fragment_data: &[([f32; 2], MaterialTyp)], velocity: [f32; 2]) -> Object {
        let min_x = fragment_data.iter().map(|(pos, _)| pos[0] as usize).min().unwrap();
        let max_x = fragment_data.iter().map(|(pos, _)| pos[0] as usize).max().unwrap();
        let min_y = fragment_data.iter().map(|(pos, _)| pos[1] as usize).min().unwrap();
        let max_y = fragment_data.iter().map(|(pos, _)| pos[1] as usize).max().unwrap();

        let h = max_y - min_y + 1;
        let w = max_x - min_x + 1;
        let anchor = [min_x as f32, min_y as f32];

        let mut object_grid: Vec<Vec<(Particle, f32, f32)>> = Vec::new();
        for i in 0..h {
            let mut row: Vec<(Particle, f32, f32)> = Vec::new();
            for j in 0..w {
                let particle_pos = [anchor[0] + j as f32, anchor[1] + i as f32];
                let particle_ref = ParticleRef::InObject(object_idx, i, j);
                let particle = Particle::new(id * 100 + (i * w + j) as i32, particle_pos, [0.0, 0.0], MaterialTyp::Luft, particle_ref);
                row.push((particle, 0.0, 0.0));
            }
            object_grid.push(row);
        }

        let mut total_mass = 0.0;
        for (world_pos, material) in fragment_data {
            let i = (world_pos[1] as usize) - min_y;
            let j = (world_pos[0] as usize) - min_x;
            let particle_ref = ParticleRef::InObject(object_idx, i, j);
            object_grid[i][j].0 = Particle::new(id * 100 + (i * w + j) as i32, *world_pos, [0.0, 0.0], *material, particle_ref);
            total_mass += material.density();
        }

        Object {
            object_id: id,
            is_destroyed: false,
            position: anchor,
            velocity,
            total_object_mass: total_mass,
            object_h: h,
            object_w: w,
            object_grid,
        }
    }

    pub fn new_quadrant(id: i32, object_idx: usize, position: [f32; 2], velocity: [f32; 2]) -> Object {
        let materials = [
            [MaterialTyp::Holz, MaterialTyp::Holz, MaterialTyp::Stein, MaterialTyp::Stein],
            [MaterialTyp::Holz, MaterialTyp::Holz, MaterialTyp::Stein, MaterialTyp::Stein],
            [MaterialTyp::Metall, MaterialTyp::Metall, MaterialTyp::Sand, MaterialTyp::Sand],
            [MaterialTyp::Metall, MaterialTyp::Metall, MaterialTyp::Sand, MaterialTyp::Sand],
        ];

        let mut object_grid: Vec<Vec<(Particle, f32, f32)>> = Vec::new();
        let mut total_mass = 0.0;

        for i in 0..4 {
            let mut row: Vec<(Particle, f32, f32)> = Vec::new();
            for j in 0..4 {
                let material = materials[i][j];
                let particle_pos = [position[0] + j as f32, position[1] + i as f32];
                let particle_ref = ParticleRef::InObject(object_idx, i, j);
                let particle = Particle::new(id * 100 + (i * 4 + j) as i32, particle_pos, [0.0, 0.0], material, particle_ref);
                total_mass += material.density();
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
            object_h: 4,
            object_w: 4,
            object_grid,
        }
    }

    pub fn get_object_elements(&self) -> Vec<&Particle> {
        self.object_grid.iter().flatten().map(|(p, _, _)| p).collect()
    }

    pub fn get_object_velocity(&self) -> [f32; 2] {
        self.velocity
    }

    pub fn get_particle_at(&self, i: usize, j: usize) -> &Particle {
        &self.object_grid[i][j].0
    }

    pub fn get_height(&self) -> usize {
        self.object_h
    }

    pub fn get_width(&self) -> usize {
        self.object_w
    }

    pub fn calc_impact_force(&self, velocity_before_impact: f32) -> f32 {
        self.total_object_mass * velocity_before_impact.abs()
    }

    fn calc_dampening_factor(collisions: &[ParticleRef]) -> f32 {
        if collisions.is_empty() { return 1.0; }
        let sum: f32 = collisions.iter().map(|c| match c {
            ParticleRef::Static => 1.0,
            ParticleRef::Free(_) => 0.4,
            ParticleRef::InObject(_, _, _) => 0.6,
        }).sum();
        sum / collisions.len() as f32
    }

    fn calc_bond_strength(mat_a: MaterialTyp, mat_b: MaterialTyp) -> f32 {
        if mat_a == mat_b {
            mat_a.binding_strength()
        } else {
            mat_a.binding_strength().min(mat_b.binding_strength()) * 0.5
        }
    }

    pub fn check_fracture(&self, impact_force: f32, dampening_factor: f32) -> Vec<((usize, usize), (usize, usize))> {
        let mut broken_bonds = Vec::new();
        let base_force = impact_force * dampening_factor;

        for i in 0..self.object_h {
            for j in 0..self.object_w {
                let mat_a = self.object_grid[i][j].0.material;
                if mat_a == MaterialTyp::Luft { continue; }

                let row_factor = 1.0 / (i as f32 + 1.0);
                let force_at_row = base_force * row_factor;

                if j + 1 < self.object_w {
                    let mat_b = self.object_grid[i][j + 1].0.material;
                    if mat_b != MaterialTyp::Luft && force_at_row > Self::calc_bond_strength(mat_a, mat_b) {
                        broken_bonds.push(((i, j), (i, j + 1)));
                    }
                }

                if i + 1 < self.object_h {
                    let mat_b = self.object_grid[i + 1][j].0.material;
                    if mat_b != MaterialTyp::Luft && force_at_row > Self::calc_bond_strength(mat_a, mat_b) {
                        broken_bonds.push(((i, j), (i + 1, j)));
                    }
                }
            }
        }
        broken_bonds
    }

    fn calc_pressure_per_column(&self, world: &World) -> Vec<f32> {
        let mut pressure_per_col = vec![0.0; self.object_w];

        for j in 0..self.object_w {
            let top_row = self.object_h - 1;
            let world_x = self.position[0] as usize + j;
            let world_y = self.position[1] as usize + top_row;

            if world_x >= world.width || world_y >= world.height { continue; }

            for y in (world_y + 1)..world.height {
                let mass_at_pos = world.grid[y][world_x].1;
                if mass_at_pos > 0.0 {
                    pressure_per_col[j] += mass_at_pos;
                }
            }
        }
        pressure_per_col
    }

    pub fn check_pressure_fracture(&self, world: &World) -> Vec<((usize, usize), (usize, usize))> {
        let mut broken_bonds = Vec::new();
        let external_pressure = self.calc_pressure_per_column(world);

        for j in 0..self.object_w {
            let mut accumulated_pressure = external_pressure[j];

            for i in (0..self.object_h).rev() {
                let particle = &self.object_grid[i][j].0;
                if particle.material == MaterialTyp::Luft { continue; }

                if i > 0 {
                    let particle_below = &self.object_grid[i - 1][j].0;
                    if particle_below.material != MaterialTyp::Luft {
                        let bond_strength = Self::calc_bond_strength(particle.material, particle_below.material);
                        if accumulated_pressure > bond_strength {
                            broken_bonds.push(((i - 1, j), (i, j)));
                        }
                    }
                }

                if j + 1 < self.object_w {
                    let particle_right = &self.object_grid[i][j + 1].0;
                    if particle_right.material != MaterialTyp::Luft {
                        let bond_strength = Self::calc_bond_strength(particle.material, particle_right.material);
                        if accumulated_pressure > bond_strength {
                            broken_bonds.push(((i, j), (i, j + 1)));
                        }
                    }
                }

                accumulated_pressure += particle.mass();
            }
        }
        broken_bonds
    }

    pub fn find_fragments(&self, broken_bonds: &[((usize, usize), (usize, usize))]) -> Vec<Vec<(usize, usize)>> {
        let mut parent: Vec<usize> = (0..self.object_h * self.object_w).collect();

        let to_index = |i: usize, j: usize| i * self.object_w + j;

        fn find(parent: &mut [usize], i: usize) -> usize {
            if parent[i] != i {
                parent[i] = find(parent, parent[i]);
            }
            parent[i]
        }

        fn union(parent: &mut [usize], a: usize, b: usize) {
            let ra = find(parent, a);
            let rb = find(parent, b);
            if ra != rb { parent[ra] = rb; }
        }

        let mut all_bonds = Vec::new();
        for i in 0..self.object_h {
            for j in 0..self.object_w {
                if self.object_grid[i][j].0.material == MaterialTyp::Luft { continue; }
                if j + 1 < self.object_w && self.object_grid[i][j + 1].0.material != MaterialTyp::Luft {
                    all_bonds.push(((i, j), (i, j + 1)));
                }
                if i + 1 < self.object_h && self.object_grid[i + 1][j].0.material != MaterialTyp::Luft {
                    all_bonds.push(((i, j), (i + 1, j)));
                }
            }
        }

        for bond in &all_bonds {
            if !broken_bonds.contains(bond) {
                let idx_a = to_index(bond.0.0, bond.0.1);
                let idx_b = to_index(bond.1.0, bond.1.1);
                union(&mut parent, idx_a, idx_b);
            }
        }

        use std::collections::HashMap;
        let mut fragments_map: HashMap<usize, Vec<(usize, usize)>> = HashMap::new();
        for i in 0..self.object_h {
            for j in 0..self.object_w {
                if self.object_grid[i][j].0.material == MaterialTyp::Luft { continue; }
                let root = find(&mut parent, to_index(i, j));
                fragments_map.entry(root).or_default().push((i, j));
            }
        }
        fragments_map.into_values().collect()
    }

    pub fn update_object_velocity(&mut self, gravity: [f32; 2], world: &World) -> Option<Vec<Vec<(usize, usize)>>> {
        let next_y = self.position[1] + self.velocity[1] + gravity[1];
        let check_y = if next_y < 0.0 { 0.0 } else { next_y };

        let mut collisions: Vec<ParticleRef> = Vec::new();
        for j in 0..self.object_w {
            let check_x = (self.position[0] + j as f32) as usize;
            if let Some(particle_ref) = world.give_occupation_on_position(check_x, check_y as usize) {
                collisions.push(particle_ref);
            }
        }

        if !collisions.is_empty() {
            let velocity_before = self.velocity[1];
            self.velocity[1] = 0.0;

            if velocity_before != 0.0 {
                let impact_force = self.calc_impact_force(velocity_before);
                let dampening = Self::calc_dampening_factor(&collisions);
                let broken_bonds = self.check_fracture(impact_force, dampening);

                if !broken_bonds.is_empty() {
                    return Some(self.find_fragments(&broken_bonds));
                }
            }
        } else if next_y < 0.0 {
            self.velocity[1] = -self.position[1];
        } else {
            self.velocity[1] += gravity[1];
        }
        None
    }

    pub fn update_object_position(&mut self, world: &mut World) {
        if self.velocity[0] == 0.0 && self.velocity[1] == 0.0 {
            return;
        }

        for i in 0..self.object_h {
            for j in 0..self.object_w {
                if self.object_grid[i][j].0.material != MaterialTyp::Luft {
                    world.clear_occupation_on_position(self.object_grid[i][j].0.position);
                    world.clear_mass_on_position(self.object_grid[i][j].0.position);
                }
            }
        }

        self.position[0] += self.velocity[0];
        self.position[1] += self.velocity[1];

        for i in 0..self.object_h {
            for j in 0..self.object_w {
                self.object_grid[i][j].0.position = [self.position[0] + j as f32, self.position[1] + i as f32];
                if self.object_grid[i][j].0.material != MaterialTyp::Luft {
                    let p = &self.object_grid[i][j].0;
                    world.update_occupation_on_position(p.position, p.particle_ref);
                    world.update_mass_on_position(p.position, p.mass());
                }
            }
        }
    }

    pub fn clear_from_world(&self, world: &mut World) {
        for i in 0..self.object_h {
            for j in 0..self.object_w {
                if self.object_grid[i][j].0.material != MaterialTyp::Luft {
                    world.clear_occupation_on_position(self.object_grid[i][j].0.position);
                    world.clear_mass_on_position(self.object_grid[i][j].0.position);
                }
            }
        }
    }

    pub fn extract_fragment_data(&self, fragment: &[(usize, usize)]) -> Vec<([f32; 2], MaterialTyp)> {
        fragment.iter().map(|(i, j)| {
            let particle = &self.object_grid[*i][*j].0;
            (particle.position, particle.material)
        }).collect()
    }
}

// ============== WORLD ==============

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

    pub fn give_pressure_on_position(&self, x: usize, y: usize) -> f32 {
        self.grid[y][x].2
    }

    pub fn give_occupation_on_position(&self, x: usize, y: usize) -> Option<ParticleRef> {
        self.grid[y][x].0
    }

    pub fn update_mass_on_position(&mut self, pos: [f32; 2], mass: f32) {
        let x = pos[0] as usize;
        let y = pos[1] as usize;
        if x < self.width && y < self.height {
            self.grid[y][x].1 = mass;
        }
    }

    pub fn update_occupation_on_position(&mut self, pos: [f32; 2], particle_ref: ParticleRef) {
        let x = pos[0] as usize;
        let y = pos[1] as usize;
        if x < self.width && y < self.height {
            self.grid[y][x].0 = Some(particle_ref);
        }
    }

    pub fn clear_occupation_on_position(&mut self, pos: [f32; 2]) {
        let x = pos[0] as usize;
        let y = pos[1] as usize;
        if x < self.width && y < self.height {
            self.grid[y][x].0 = None;
        }
    }

    pub fn clear_mass_on_position(&mut self, pos: [f32; 2]) {
        let x = pos[0] as usize;
        let y = pos[1] as usize;
        if x < self.width && y < self.height {
            self.grid[y][x].1 = 0.0;
        }
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