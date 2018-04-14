extern crate piston;
extern crate graphics;
extern crate glutin_window;
extern crate opengl_graphics;
extern crate rand;

use piston::window::WindowSettings;
use piston::event_loop::*;
use piston::input::*;
use glutin_window::GlutinWindow as Window;
use opengl_graphics::{ GlGraphics, OpenGL };

use rand::Rng;


const GREEN: [f32; 4] = [0.0, 1.0, 0.0, 1.0];
const RED: [f32; 4] = [1.0, 0.0, 0.0, 1.0];
const GRAY: [f32; 4] = [0.5, 0.5, 0.5, 1.0];

const SQUARE_WIDTH: usize = 50;
const MOV_SPEED: f64 = 1000.0;

const WINDOW_HEIGHT: usize = 800;
const WINDOW_WIDTH: usize = 400;

const GAME_AREA_HEIGHT: usize = WINDOW_HEIGHT;
const GAME_AREA_WIDTH: usize = WINDOW_WIDTH;

const LANE_WIDTH: usize = SQUARE_WIDTH;
const LANE_HEIGHT: usize = SQUARE_WIDTH;
const N_WIDTH_LANES: usize = (GAME_AREA_WIDTH / LANE_WIDTH);
const N_HEIGHT_LANES: usize = (GAME_AREA_HEIGHT / LANE_WIDTH);
const LAST_HEIGHT_POS: usize = N_HEIGHT_LANES - 1;
const LAST_WIDTH_POS: usize = N_WIDTH_LANES - 1;

#[derive(Debug)]
struct LanePosition {
    x: usize,
    y: usize
}

#[derive(Debug)]
struct Square {
    pos: LanePosition,
    float_pos: f64,
    color: [f32; 4],
}

impl Square {
    fn render(&self, gl: &mut GlGraphics, args: &RenderArgs) {
        use graphics::*;
        let square = rectangle::square(
            (self.pos.x * LANE_WIDTH) as f64,
            (self.pos.y * LANE_HEIGHT) as f64,
            SQUARE_WIDTH as f64);
        gl.draw(args.viewport(), |c, gl| {
            let transform = c.transform;
            rectangle(self.color, square, transform, gl);
        });
    }
}

struct App {
    gl: GlGraphics, // OpenGL drawing backend.
    still_squares: Vec<Square>,
    falling_square: Square
}
impl App {
    fn render(&mut self, args: &RenderArgs) {
        use graphics::*;

        self.gl.draw(args.viewport(), |_, gl| {
            clear(GRAY, gl);
        });

        self.falling_square.render(&mut self.gl, args);

        for sq in &self.still_squares {
            sq.render(&mut self.gl, args);
        }
    }

    fn get_new_start_pos() -> LanePosition {
        let mut rng = rand::thread_rng();
        let num: usize = rng.gen_range(0, N_WIDTH_LANES); // random number in range (0, 1)
        return LanePosition {x: num, y: 0};
    }

    pub fn get_new_square() -> Square {
        let start = App::get_new_start_pos();
        return Square{pos: start, float_pos: 0.0, color: RED};
    }

    fn is_done(&self, sq: &Square) -> bool {
        if sq.pos.y == LAST_HEIGHT_POS {
            return true;
        }

        for ssq in &self.still_squares {
            if sq.pos.x == ssq.pos.x && sq.pos.y + 1 == ssq.pos.y {
                return true;
            }
        }

        return false;
    }

    fn clean_filled_rows(&mut self) {
        let mut occupied: [[bool; N_HEIGHT_LANES]; N_WIDTH_LANES] = [[false; N_HEIGHT_LANES]; N_WIDTH_LANES];
        for sq in &self.still_squares {
            occupied[sq.pos.x][sq.pos.y] = true;
        }

        for j in 0..N_HEIGHT_LANES {
            let mut whole_row = true;
            for i in 0..N_WIDTH_LANES {
                whole_row = whole_row && occupied[i][j];
            }

            if whole_row {
                self.still_squares.retain(|ref e| e.pos.y != j);

                for mut sq in &mut self.still_squares {
                    if sq.pos.y < j {
                        sq.pos.y += 1;
                    }
                }
            }
        }
    }

    fn get_lane_pos_from(pos: f64) -> usize { (pos / LANE_HEIGHT as f64).floor() as usize}

    fn decr_lane_pos(n: usize) -> usize {
        match n {
            0 => 0,
            x => x - 1
        }
    }

    fn incr_lane_pos(n: usize) -> usize {
        match n {
            LAST_WIDTH_POS => LAST_WIDTH_POS,
            x => x + 1
        }
    }

    fn update(&mut self, args: &UpdateArgs) {
        if self.is_done(&self.falling_square) {
            self.still_squares.push(std::mem::replace(&mut self.falling_square, App::get_new_square()));
            self.clean_filled_rows();
        }

        self.falling_square.float_pos += MOV_SPEED * args.dt;
        self.falling_square.pos.y = App::get_lane_pos_from(self.falling_square.float_pos);
    }

    fn handle_button_input(&mut self, args: &ButtonArgs) {
        if args.state == ButtonState::Press {
            if args.button == Button::Keyboard(Key::Left) {
                self.falling_square.pos.x = App::decr_lane_pos(self.falling_square.pos.x);
            } else if args.button == Button::Keyboard(Key::Right) {
                self.falling_square.pos.x = App::incr_lane_pos(self.falling_square.pos.x);
            }
        }
    }
}

fn main() {
    let opengl = OpenGL::V3_2;

    let mut window: Window = WindowSettings::new(
            "tetris",
            [WINDOW_WIDTH as u32, WINDOW_HEIGHT as u32]
        )
        .opengl(opengl)
        .exit_on_esc(true)
        .build()
        .unwrap();

    // Create a new game and run it.
    let mut app = App {
        gl: GlGraphics::new(opengl),
        still_squares: Vec::new(),
        falling_square: App::get_new_square()
    };

    let mut events = Events::new(EventSettings::new());
    while let Some(e) = events.next(&mut window) {
        if let Some(r) = e.render_args() {
            app.render(&r);
        }

        if let Some(u) = e.update_args() {
            app.update(&u);
        }

        if let Some(u) = e.button_args() {
            app.handle_button_input(&u);
        }
    }
}

