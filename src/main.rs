
use ggez::{
    event, graphics::{self, Color, InstanceArray, Drawable, DrawParam},
    input::keyboard::{KeyInput},
    Context, GameResult,
};

use rand::Rng; 

const DESIRED_FPS: u32 = 30;
const SCREEN_SIZE: (u32, u32) = (2000,2000);

const PARTICLE_SIZE: i32 = 10;

const COLOURS: [Color; 4] = [
    Color::GREEN,
    Color::RED,
    Color::CYAN,
    Color::BLUE,
    //Color::YELLOW,
    // Color::WHITE,
    // Color::MAGENTA,
    // Color::new(0.1,0.5,0.0,1.0),
    // Color::new(1.0,0.3,0.3,1.0),
    // Color::new(0.4, 0.9, 0.7,1.0)
];

const DENSITY: u32 = 600;

const MAX_EFFECT_DISTANCE: f32 = 800.0;
const MAX_GRAVITY: f32 = 2.0;
const FRICTION: f32 = 0.5;

const ROUND_ROBIN: usize = 2;


#[derive(Clone, Copy, Debug, PartialEq)]
struct GridPosition{
    x: f32,
    y: f32,
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

impl From<(f32, f32)> for GridPosition {
    fn from(pos: (f32, f32)) -> Self {
        GridPosition { x: pos.0, y: pos.1}
    }
}


struct Particle {
    pos: GridPosition,
    vx: f32,
    vy: f32, 
    colour: Color,
    id: u32
}

fn random_f32() -> f32{
    let mut rng = rand::thread_rng();
    rng.gen::<f32>()
}

fn random_f32_range(low: f32, high: f32) -> f32{
    let mut rng = rand::thread_rng();
    rng.gen_range(low..high)
}

fn random_i8() -> i8{
    let mut rng = rand::thread_rng();
    rng.gen::<i8>()
}

struct GameState {
    particles: Vec<Vec<Particle>>,
    draw_instances: Option<InstanceArray>,
    obj_count: u32,
    relations: [(i8, f32, f32); COLOURS.len() * COLOURS.len()],
    ticks: usize
}


impl GameState{
    fn new() -> Self{
        let mut game_state: GameState = GameState{
            particles: Vec::new(),
            draw_instances: None,
            obj_count: 0,
            relations: [(0,0.0,0.0); COLOURS.len() * COLOURS.len()],
            ticks: 1
        };
        
        game_state.init();

        game_state
    }


    fn random_relations(&mut self) -> [(i8, f32, f32); COLOURS.len() * COLOURS.len()]{
        let mut rel = [(0,0.0,0.0);COLOURS.len() * COLOURS.len()];
        for i in 0..COLOURS.len() * COLOURS.len(){
            rel[i] = (random_i8(), (random_f32() - 0.5) * MAX_GRAVITY, random_f32() * MAX_EFFECT_DISTANCE);
        }
        rel
    }

    fn init(&mut self){
        for c in COLOURS{
            let new_particles = self.create_particles(DENSITY, c);
            self.particles.push(new_particles);
        }
        self.relations = self.random_relations()
       
    }

    fn create_particles(&mut self, num: u32, color: Color) -> Vec<Particle>{
        let mut group: Vec<Particle> = Vec::new();
        for _ in 0..num{
            let pos = GridPosition{
                x: random_f32_range(0.0, SCREEN_SIZE.0 as f32),
                y: random_f32_range(0.0, SCREEN_SIZE.1 as f32)
            };
            
            let particle = Particle{
                pos: pos,
                colour: color,
                id: self.obj_count,
                vx: (random_f32() - 0.5) * 50.0,
                vy: (random_f32() - 0.5) * 50.0
            };

            group.push(
                particle
            );
            self.obj_count += 1;
           
        }
        group

    }


    fn _wrap_around_screen(x: f32, y: f32) -> GridPosition{
        let mut rx = x;
        let mut ry = y;
        if x > (SCREEN_SIZE.0 - PARTICLE_SIZE as u32) as f32 {
            rx = (x - SCREEN_SIZE.0 as f32) + PARTICLE_SIZE as f32 ; 
        }
        if x < PARTICLE_SIZE as f32 {
            rx = SCREEN_SIZE.0 as f32 - x;
        } 
    
        if y > SCREEN_SIZE.1 as f32{
            ry = y - SCREEN_SIZE.1 as f32; 

        }
        if y < PARTICLE_SIZE as f32{
            ry = (SCREEN_SIZE.1 - PARTICLE_SIZE as u32) as f32 - y;
        }
        GridPosition { x: rx, y: ry }
    }

    fn clamp_to_screen(x: f32, y: f32) -> GridPosition{
        let mut rx = x;
        let mut ry = y;
        
        if x > SCREEN_SIZE.0 as f32 {
            rx = SCREEN_SIZE.0 as f32 - 1.0; 
        }
        if x < 0.0 {
            rx = 1.0;
        } 
    
        if y > SCREEN_SIZE.1 as f32{
            ry = SCREEN_SIZE.1 as f32 - 1.0; 

        }
        if y < 0.0{
            ry = 1.0;
        }
        GridPosition { x: rx, y: ry }
    }

    fn regular_distance(x1: f32, x2: f32, y1: f32, y2: f32) -> (f32, f32, f32){
        let dx = x1 - x2;
        let dy = y1 - y2;
        (f32::sqrt((dx * dx) + (dy * dy)), dx, dy)

    }
    fn _toroidal_distance(x1: f32, x2: f32, y1: f32, y2: f32) -> (f32, f32, f32){
        let mut dx = f32::abs(x2 - x1);
        let mut dy = f32::abs(y2-y1);

        if dx > 0.5 {
            dx = 1.0 - dx;
        }
 
        if dy > 0.5 {
            dy = 1.0 - dy;
        }
        (f32::sqrt(dx*dx + dy*dy),dx, dy)

    }
    fn update_particles(&mut self){
        let mut i = 0;
        for x in 0..COLOURS.len(){
            for y in 0..COLOURS.len(){
                self.particle_relation(x, y, i as usize);
                i += 1;
            }
        
        }
        for x in 0..self.particles.len(){
                for y in 0..self.particles[x].len(){ 
                    let particle = &mut self.particles[x][y];
  
                    particle.vx = particle.vx * (1.0 - FRICTION);
                    particle.vy = particle.vy * (1.0 - FRICTION);
                    particle.pos.x += particle.vx;
                    particle.pos.y += particle.vy;

                    particle.pos = GameState::_wrap_around_screen(particle.pos.x, particle.pos.y);

                    
                };
            };
        self.ticks = self.ticks+1;
    }
    

    

    fn particle_relation(&mut self, colour_a: usize, colour_b: usize, relation_idx: usize){
        for p1 in 0..self.particles[colour_a].len(){
            if p1 % ROUND_ROBIN == (self.ticks % ROUND_ROBIN){
                continue
            }
            let mut fx = 0.0;
            let mut fy = 0.0;
            for p2 in 0..self.particles[colour_b].len(){
                let (d, dx, dy) = GameState::regular_distance(
                        self.particles[colour_a][p1].pos.x,
                        self.particles[colour_b][p2].pos.x,
                        self.particles[colour_a][p1].pos.y,
                        self.particles[colour_b][p2].pos.y
                    );
                if d > 50.0 && d< self.relations[relation_idx].2{
                    let f = self.relations[relation_idx].1 * (1.0/d);
                    fx += f * dx;
                    fy += f * dy;
                }
            }

            self.particles[colour_a][p1].vx = self.particles[colour_a][p1].vx + fx;
            self.particles[colour_a][p1].vy = self.particles[colour_a][p1].vy + fy;

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

    fn key_down_event(&mut self, _ctx: &mut Context, _input: KeyInput, _repeat: bool) -> GameResult {
       
        for x in 0..self.particles.len(){
            for y in 0..self.particles[x].len(){
                let pos = GridPosition{
                    x: random_f32_range(0.0, SCREEN_SIZE.0 as f32) ,
                    y: random_f32_range(0.0, SCREEN_SIZE.1 as f32)
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