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
const MOV_SPEED: f64 = 100.0;

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

#[derive(Debug, Copy, Clone)]
struct LanePosition {
    x: usize,
    y: usize
}

impl LanePosition {
    fn prev_x(n: usize) -> usize {
        match n {
            0 => 0,
            x => x - 1
        }
    }

    fn next_x(n: usize) -> usize {
        match n {
            LAST_WIDTH_POS => LAST_WIDTH_POS,
            x => x + 1
        }
    }

    fn decr_x(&mut self) {
        self.x = LanePosition::prev_x(self.x);
    }

    fn incr_x(&mut self) {
        self.x = LanePosition::next_x(self.x);
    }

    fn incr_y(&mut self) {
        self.y += 1;
    }
}

#[derive(Debug, Copy, Clone)]
struct Square {
    pos: LanePosition,
    color: [f32; 4],
}

#[derive(Debug, Copy, Clone)]
enum TetraminoType {
    Line = 0,
    Sq = 1,
    T = 2,
    L = 3,
    J = 4,
    S = 5,
    Z = 6
}

// TODO: Generate with macro
impl From<usize> for TetraminoType {
    fn from(n: usize) -> Self {
        use TetraminoType::*;
        if n == 0 {
            return Line;
        } else if n == 1 {
            return Sq;
        } else if n == 2 {
            return T;
        } else if n == 3 {
            return L;
        } else if n == 4 {
            return J;
        } else if n == 5 {
            return S;
        } else {
            assert_eq!(n, 6);
            return Z;
        }
    }
}

impl TetraminoType {
    fn get_width(tt: TetraminoType) -> usize {
        use TetraminoType::*;
        match tt {
            Line => 4,
            Sq => 2,
            T => 3,
            L => 3,
            J => 3,
            S => 3,
            Z => 3,
        }
    }
}

struct Tetramino {
    t_type: TetraminoType,
    squares: [Square; 4],
    float_pos: f64,
    pos: LanePosition,
}

impl Square {
    fn render(&self, gl: &mut GlGraphics, args: &RenderArgs) {
        use graphics::*;
        let square = rectangle::square(
            (self.pos.x * LANE_WIDTH) as f64,
            (self.pos.y * LANE_HEIGHT) as f64,
            SQUARE_WIDTH as f64);
        gl.draw(args.viewport(), |c, gl| {
            rectangle(self.color, square, c.transform, gl);
        });
    }
}

impl Tetramino {
    fn render(&self, gl: &mut GlGraphics, args: &RenderArgs) {
        for sq in self.squares.iter() {
            sq.render(gl, args);
        }
    }

    fn get_new_type() -> TetraminoType {
        let mut rng = rand::thread_rng();
        let num: usize = rng.gen_range(0, 7);
        return TetraminoType::from(num);
    }

    fn get_new_start_pos(t_type: TetraminoType) -> LanePosition {
        let last_start_pos = TetraminoType::get_width(t_type) - 1;

        let mut rng = rand::thread_rng();
        let num: usize = rng.gen_range(0, last_start_pos + 1);
        return LanePosition {x: num, y: 0};
    }

    fn get_squares(t_type: TetraminoType, start_pos: LanePosition) -> [Square; 4] {
        use TetraminoType::*;
        match t_type {
            Line => {[Square{pos: start_pos, color: RED},
                 Square{pos: LanePosition{x: start_pos.x + 1, y: start_pos.y}, color: RED},
                 Square{pos: LanePosition{x: start_pos.x + 2, y: start_pos.y}, color: RED},
                 Square{pos: LanePosition{x: start_pos.x + 3, y: start_pos.y}, color: RED}]},
            Sq => {[Square{pos: start_pos, color: RED},
                 Square{pos: LanePosition{x: start_pos.x + 1, y: start_pos.y}, color: RED},
                 Square{pos: LanePosition{x: start_pos.x, y: start_pos.y + 1}, color: RED},
                 Square{pos: LanePosition{x: start_pos.x + 1, y: start_pos.y + 1}, color: RED}]},
            T => {[Square{pos: start_pos, color: RED},
                 Square{pos: LanePosition{x: start_pos.x + 1, y: start_pos.y}, color: RED},
                 Square{pos: LanePosition{x: start_pos.x + 2, y: start_pos.y}, color: RED},
                 Square{pos: LanePosition{x: start_pos.x + 1, y: start_pos.y + 1}, color: RED}]},
            L => {[Square{pos: start_pos, color: RED},
                 Square{pos: LanePosition{x: start_pos.x + 1, y: start_pos.y}, color: RED},
                 Square{pos: LanePosition{x: start_pos.x + 2, y: start_pos.y}, color: RED},
                 Square{pos: LanePosition{x: start_pos.x, y: start_pos.y + 1}, color: RED}]},
            J => {[Square{pos: start_pos, color: RED},
                 Square{pos: LanePosition{x: start_pos.x + 1, y: start_pos.y}, color: RED},
                 Square{pos: LanePosition{x: start_pos.x + 2, y: start_pos.y}, color: RED},
                 Square{pos: LanePosition{x: start_pos.x + 2, y: start_pos.y + 1}, color: RED}]},
            S => {[Square{pos: LanePosition{x: start_pos.x + 1, y: start_pos.y} , color: RED},
                 Square{pos: LanePosition{x: start_pos.x + 2, y: start_pos.y}, color: RED},
                 Square{pos: LanePosition{x: start_pos.x, y: start_pos.y + 1}, color: RED},
                 Square{pos: LanePosition{x: start_pos.x + 1, y: start_pos.y + 1}, color: RED}]},
            Z => {[Square{pos: start_pos, color: RED},
                 Square{pos: LanePosition{x: start_pos.x + 1, y: start_pos.y}, color: RED},
                 Square{pos: LanePosition{x: start_pos.x + 1, y: start_pos.y + 1}, color: RED},
                 Square{pos: LanePosition{x: start_pos.x + 2, y: start_pos.y + 1}, color: RED}]},
        }
    }

    fn new() -> Tetramino {
        let t_type = Tetramino::get_new_type();
        let start = Tetramino::get_new_start_pos(t_type);
        let squares = Tetramino::get_squares(t_type, start);
        return Tetramino{t_type: t_type, squares: squares, float_pos: 0.0, pos: start};
    }

    fn move_left(&mut self) {
        if self.pos.x <= 0 {
            return;
        }

        self.pos.decr_x();

        for sq in &mut self.squares {
            sq.pos.decr_x();
        }
    }

    fn move_down(&mut self) {
        for sq in &mut self.squares {
            sq.pos.incr_y();
        }
    }

    fn move_right(&mut self) {
        if self.pos.x + TetraminoType::get_width(self.t_type) - 1 >= LAST_WIDTH_POS {
            return;
        }

        self.pos.incr_x();

        for sq in &mut self.squares {
            sq.pos.incr_x();
        }
    }

}

struct App {
    gl: GlGraphics,
    square_slots: [[Option<Square>; N_HEIGHT_LANES]; N_WIDTH_LANES],
    tetramino: Tetramino
}

impl App {
    fn render(&mut self, args: &RenderArgs) {
        use graphics::*;

        self.gl.draw(args.viewport(), |_, gl| {
            clear(GRAY, gl);
        });

        self.tetramino.render(&mut self.gl, args);

        for it in self.square_slots.iter() {
            for opt in it {
                if let &Some(ssq) = opt {
                    ssq.render(&mut self.gl, args);
                }
            }
        }
    }

    fn is_square_done(&self, sq: &Square) -> bool {
        if sq.pos.y == LAST_HEIGHT_POS {
            return true;
        }

        for it in self.square_slots.iter() {
            for opt in it {
                if let &Some(ssq) = opt {
                    if sq.pos.x == ssq.pos.x && sq.pos.y + 1 == ssq.pos.y {
                        return true;
                    }
                }
            }
        }

        return false;
    }

    fn is_done(&self, t: &Tetramino) -> bool {
        return t.squares.iter().any(|sq| self.is_square_done(sq));
    }

    fn clean_filled_rows(&mut self) {
        for j in (0..N_HEIGHT_LANES).rev() {
            let mut whole_row = true;
            for i in 0..N_WIDTH_LANES {
                whole_row = whole_row && self.square_slots[i][j].is_some();
            }

            if whole_row {
                for i in 0..N_WIDTH_LANES {
                    self.square_slots[i][j] = None;
                }

                for k in (0..(j-1)).rev() {
                    for l in 0..N_WIDTH_LANES {
                        if let Some(mut sq) = self.square_slots[l][k] {
                            if self.square_slots[l][k+1].is_none() {
                                self.square_slots[l][k+1] = Some(sq);
                                self.square_slots[l][k] = None;
                                sq.pos.incr_y();
                            }
                        }
                    }
                }
            }
        }
    }

    fn decompose_tetramino(&mut self, tetra: Tetramino) {
        for sq in tetra.squares.iter() {
            assert_eq!(self.square_slots[sq.pos.x][sq.pos.y].is_none(), true);
            self.square_slots[sq.pos.x][sq.pos.y] = Some(*sq);
        }
    }

    fn update(&mut self, args: &UpdateArgs) {
        if self.is_done(&self.tetramino) {
            let old_tetra = std::mem::replace(&mut self.tetramino, Tetramino::new());
            self.decompose_tetramino(old_tetra);
            self.clean_filled_rows();
        }

        self.tetramino.float_pos += MOV_SPEED * args.dt;
        if self.tetramino.float_pos > LANE_HEIGHT as f64{
            self.tetramino.float_pos = 0.0;
            self.tetramino.move_down();
        }
    }

    fn handle_button_input(&mut self, args: &ButtonArgs) {
        if args.state == ButtonState::Press {
            if args.button == Button::Keyboard(Key::Left) {
                self.tetramino.move_left();
            } else if args.button == Button::Keyboard(Key::Right) {
                self.tetramino.move_right();
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
        square_slots: [[None; N_HEIGHT_LANES]; N_WIDTH_LANES],
        tetramino: Tetramino::new()
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

