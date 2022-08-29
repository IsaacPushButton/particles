use ggez::{
    event, graphics::{self, Text, Color, InstanceArray, Drawable, DrawParam},
    input::keyboard::{KeyCode, KeyInput},
    Context, GameResult, winit::dpi::{Size, Position},
};

use rand::Rng; 

const DESIRED_FPS: u32 = 20;
const SCREEN_SIZE: (u32, u32) = (3000,2000);

const PARTICLE_SIZE: i32 = 4;

const N: usize = 3;
const COLOURS: [Color; N] = [
    Color::GREEN,
    Color::RED,
    Color::CYAN,
 //   Color::BLUE,
];

const DENSITY: u32 = 800;



#[derive(Clone, Copy, Debug, PartialEq)]
struct GridPosition{
    x: f64,
    y: f64,
}

impl From<GridPosition> for graphics::Rect {
    fn from(grid: GridPosition) -> Self {
        graphics::Rect::new_i32(
            grid.x as i32,
            grid.y as i32,
            PARTICLE_SIZE as i32,
            PARTICLE_SIZE as i32,
        )
    }
}

impl From<(f64, f64)> for GridPosition {
    fn from(pos: (f64, f64)) -> Self {
        GridPosition { x: pos.0, y: pos.1}
    }
}


struct Particle {
    pos: GridPosition,
    vx: f64,
    vy: f64, 
    colour: Color,
    id: u32
}

struct GameState {
    particles: Vec<Vec<Particle>>,
    draw_instances: Option<InstanceArray>,
    obj_count: u32,
    relations: [(i8, f64, f64); N*N],
    rng: rand::rngs::ThreadRng,
}


impl GameState{
    fn new() -> Self{
       // let mut seed: [u8; 8] = [1; 8];
       // getrandom::getrandom(&mut seed[..]).expect("Could not create RNG seed");
       // let rng = Rand32::new(u64::from_ne_bytes(seed));

        let mut game_state: GameState = GameState{
            particles: Vec::new(),
            draw_instances: None,
            obj_count: 0,
            relations: [(0,0.0,0.0); N*N],
            rng: rand::thread_rng()
        };
        
        game_state.init();

        game_state
    }

    fn random_relations(&mut self) -> [(i8, f64, f64); N*N]{
        let mut rel = [(0,0.0,0.0);N*N];
        for i in 0..N*N{

            rel[i] = (self.rng.gen::<i8>(), (self.rng.gen::<f64>() - 0.5), (self.rng.gen::<f64>() * 600.0));
        }
        rel
    }

    fn init(&mut self){
        
        for c in COLOURS{
            self.create_particles(DENSITY, c);
        }
        self.relations = self.random_relations()
       
    }

    fn create_particles(&mut self, num: u32, color: Color){
        let mut group: Vec<Particle> = Vec::new();
        for n in 0..num{
            let pos = GridPosition{
                x: self.rng.gen_range(0..SCREEN_SIZE.0 + 1) as f64,
                y: self.rng.gen_range(0..SCREEN_SIZE.1 + 1) as f64
            };
            
            let particle = Particle{
                pos: pos,
                colour: color,
                id: self.obj_count,
                vx: self.rng.gen::<f64>() * 10.0,
                vy: self.rng.gen::<f64>() * 10.0
            };

            group.push(
                particle
            );
            self.obj_count += 1;
           
        }
        self.particles.push(group);

    }


    fn clamp_to_screen(x: f64, y: f64) -> GridPosition{
        let mut rx = x;
        let mut ry = y;
        if x > SCREEN_SIZE.0 as f64 {
            rx = x - SCREEN_SIZE.0 as f64; 
        }
        if x < 0.0 {
            rx = SCREEN_SIZE.0 as f64 - x;
        } 
    
        if y > SCREEN_SIZE.1 as f64{
            ry = y - SCREEN_SIZE.1 as f64; 

        }
        if y < 0.0{
            ry = SCREEN_SIZE.1 as f64 - y;
        }
        GridPosition { x: rx, y: ry }
    }


    fn update_particles(&mut self){
        let mut i = 0;
        for x in 0..N{
            for y in 0..N{
                self.particle_relation(x, y, i);
                i += 1;

            }
        }

    }

    fn particle_relation(&mut self, x: usize,y: usize, relation_idx: usize){
      //  if self.relations[relation_idx].0 == 0{
      //      return
      //  }
        for p1 in 0..self.particles[x].len(){
            let mut fx = 0.0;
            let mut fy = 0.0;
            for p2 in 0..self.particles[y].len(){
                let p1x = self.particles[x][p1].pos.x;
                let p1y = self.particles[x][p1].pos.y;
                let p2x = self.particles[y][p2].pos.x;
                let p2y = self.particles[y][p2].pos.y;
                let dx = p1x - p2x;
                let dy = p1y - p2y;
                let d = (((dx*dx) + (dy*dy))).sqrt();
                if d > 5.0 && d< self.relations[relation_idx].2 {
                    let F = self.relations[relation_idx].1 * (1.0/d);
                    fx += F * dx;
                    fy += F * dy ;
                }
            }

            self.particles[x][p1].vx = (self.particles[x][p1].vx + fx) * 0.7;
            self.particles[x][p1].vy = (self.particles[x][p1].vy + fy) * 0.7;

            


            // if self.particles[x][p1].pos.x <= 0.0 || self.particles[x][p1].pos.x >= SCREEN_SIZE.0 as f64 { 
            //     self.particles[x][p1].vx = self.particles[x][p1].vx * -1.0;
            // }
            // if self.particles[x][p1].pos.y <= 0.0 || self.particles[x][p1].pos.y >= SCREEN_SIZE.1 as f64 { 
            //     self.particles[x][p1].vy = self.particles[x][p1].vy * -1.0;
            // }
            
            self.particles[x][p1].pos.x += self.particles[x][p1].vx;
            self.particles[x][p1].pos.y += self.particles[x][p1].vy;

            self.particles[x][p1].pos = GameState::clamp_to_screen(self.particles[x][p1].pos.x, self.particles[x][p1].pos.y);

        }
    }

}





impl event::EventHandler<ggez::GameError> for GameState {

    fn update(&mut self, ctx: &mut Context) -> GameResult {

        while ctx.time.check_update_time(DESIRED_FPS) {
            self.update_particles();
        }
        
        Ok(())
    
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        if self.draw_instances.is_none(){
            let mut instance_array = InstanceArray::new(ctx, None, 99999, false);
            
            for particle_group in &self.particles{
                for particle in particle_group{
                    instance_array.push(graphics::DrawParam::new()
                    .dest_rect(particle.pos.into())
                    .color(particle.colour))
                }
            }
            self.draw_instances = Some(instance_array);
        }

        let mut canvas = graphics::Canvas::from_frame(
            ctx,
            graphics::CanvasLoadOp::Clear([0.0, 0.0, 0.0, 1.0].into()),
        );
        let inst = self.draw_instances.as_mut().unwrap();
        for particle_group in &self.particles{
            for particle in particle_group{
                inst.update(particle.id,graphics::DrawParam::new()
                .dest_rect(particle.pos.into())
                .color(particle.colour))
            }
        }
        inst.draw(&mut canvas, DrawParam::new());
        canvas.finish(ctx)?;
    
        ggez::timer::yield_now();
        Ok(())
    }

    fn key_down_event(&mut self, _ctx: &mut Context, input: KeyInput, _repeat: bool) -> GameResult {
        let max_x: u32 = SCREEN_SIZE.0 + 1;
        let max_y: u32 = SCREEN_SIZE.1 + 1;

        for x in 0..self.particles.len(){
            for y in 0..self.particles[x].len(){
                let pos = GridPosition{
                    x: self.rng.gen_range(0..max_x) as f64,
                    y: self.rng.gen_range(0..max_y) as f64
                };
                self.particles[x][y].pos = pos;
            }
        }
        self.relations = self.random_relations();

        Ok(())
    }


}



fn main() -> GameResult {
    // Here we use a ContextBuilder to setup metadata about our game. First the title and author
    let (ctx, events_loop) = ggez::ContextBuilder::new("Cellular Automata", "Isaac")
        // Next we set up the window. This title will be displayed in the title bar of the window.
        .window_setup(ggez::conf::WindowSetup::default().title("Cells"))
        // Now we get to set the size of the window, which we use our SCREEN_SIZE constant from earlier to help with
        .window_mode(ggez::conf::WindowMode::default().dimensions(SCREEN_SIZE.0 as f32, SCREEN_SIZE.1 as f32))
        // And finally we attempt to build the context and create the window. If it fails, we panic with the message
        // "Failed to build ggez context"
        .build()?;

    // Next we create a new instance of our GameState struct, which implements EventHandler
    let state = GameState::new();

    // And finally we actually run our game, passing in our context and state.
    event::run(ctx, events_loop, state)
}