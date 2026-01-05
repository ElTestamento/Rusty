use rand::seq::SliceRandom;

#[derive(Debug)]
pub struct Particle {
    pub id: i32,
    pub position: [f32; 2],
    pub velocity: [f32; 2],
    pub mass: f32,
    pub is_solid: bool,
    pub block_id: Option<i32>,
}

impl Particle {
    pub fn new(id: i32, position: [f32; 2], velocity: [f32; 2], mass: f32) -> Particle {
        println!("Erschaffe neues Partikel");
        Particle {
            id,
            position,
            velocity,
            mass,
            is_solid: false,
            block_id: None,
        }
    }

    pub fn new_solid(id: i32, position: [f32; 2], mass: f32, block_id: i32) -> Particle {
        println!("Erschaffe festen Block");
        Particle {
            id,
            position,
            velocity: [0.0, 0.0],
            mass,
            is_solid: true,
            block_id: Some(block_id),
        }
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
        if self.is_solid {
            return;
        }

        let own_x = self.position[0] as usize;
        let own_y = self.position[1] as usize;
        let own_pressure = world.give_pressure_on_position(own_x, own_y);

        if own_pressure <= self.mass {
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
                    world.update_mass_on_position(self.position, self.mass);
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
            world.update_mass_on_position(self.position, self.mass);
            return;
        }

        // Solids fallen NICHT diagonal
        if self.is_solid {
            return;
        }

        // Diagonal links unten frei?
        if x > 0 && !world.give_occupation_on_position((x - 1) as usize, (y - 1) as usize) {
            world.clear_occupation_on_position(self.position);
            world.clear_mass_on_position(self.position);
            self.position[0] -= 1.0;
            self.position[1] -= 1.0;
            world.update_occupation_on_position(self.position);
            world.update_mass_on_position(self.position, self.mass);
            return;
        }

        // Diagonal rechts unten frei?
        if x < (world.width - 1) as i32 && !world.give_occupation_on_position((x + 1) as usize, (y - 1) as usize) {
            world.clear_occupation_on_position(self.position);
            world.clear_mass_on_position(self.position);
            self.position[0] += 1.0;
            self.position[1] -= 1.0;
            world.update_occupation_on_position(self.position);
            world.update_mass_on_position(self.position, self.mass);
        }
    }

    pub fn get_position(&self) -> [f32; 2] {
        self.position
    }

    pub fn get_velocity(&self) -> [f32; 2] {
        self.velocity
    }

    pub fn get_impuls(&self) -> [f32; 2] {
        [self.velocity[0] * self.mass, self.velocity[1] * self.mass]
    }

    pub fn update_position(&mut self, world: &mut World) {
        world.clear_occupation_on_position(self.position);
        world.clear_mass_on_position(self.position);

        for i in 0..2 {
            self.position[i] += self.velocity[i];
        }

        world.update_occupation_on_position(self.position);
        world.update_mass_on_position(self.position, self.mass);
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

pub struct Object {
    object_id : i32,
    position : [f32;2],
    velocity : [f32;2],
    mass : f32,
    is_solid: bool,
    h : i32,
    w : i32,
    grid : Vec<Vec<(bool, f32, f32)>>,

    }
impl Object {
    pub fn new(id: i32, position: [f32; 2], velocity: [f32; 2], mass: f32, is_solid: bool, h: i32, w: i32) -> Object {
        println!("Erschaffe neues Partikel");
        Object {
            object_id: id,
            position: position,
            velocity: velocity,
            mass: mass,
            is_solid: is_solid,
            h: h,
            w: w,
            grid: vec![vec![(false, 0.0, 0.0); w]; h],
        }
    }
}

//HIER GEHTS WEITER: FUNKTIONALITÖT DES BOCK-OBJEKTS EINBAUEN!''''''''''''''''''''''''''''''''''''''''''''''''''''''''''''''''''''''''''''''''''''''''''''''
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
