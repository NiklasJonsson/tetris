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

const BLUE: [f32; 4] = [0.0, 0.0, 1.0, 1.0];
const RED: [f32; 4] = [1.0, 0.0, 0.0, 1.0];
const GREEN: [f32; 4] = [0.0, 1.0, 0.0, 1.0];
const CYAN: [f32; 4] = [0.0, 1.0, 1.0, 1.0];
const YELLOW: [f32; 4] = [1.0, 1.0, 0.0, 1.0];
const PURPLE: [f32; 4] = [1.0, 0.0, 1.0, 1.0];
const ORANGE: [f32; 4] = [1.0, 0.5, 0.0, 1.0];

const GRAY: [f32; 4] = [0.5, 0.5, 0.5, 1.0];
const BLACK: [f32; 4] = [0.0, 0.0, 0.0, 1.0];
const WHITE: [f32; 4] = [1.0, 1.0, 0.0, 1.0];

const BASE_MOVE_SPEED: f64 = 100.0;
const MAX_MOVE_SPEED: f64 = 200.0;

const SQUARE_WIDTH: usize = 30;
const LANE_WIDTH: usize = SQUARE_WIDTH;
const LANE_HEIGHT: usize = SQUARE_WIDTH;
const N_WIDTH_LANES: usize = 10;
const N_HEIGHT_LANES: usize = 20;
const GAME_AREA_WIDTH: usize = LANE_WIDTH * N_WIDTH_LANES;
const GAME_AREA_HEIGHT: usize = LANE_HEIGHT * N_HEIGHT_LANES;
const LAST_HEIGHT_POS: usize = N_HEIGHT_LANES - 1;
const LAST_WIDTH_POS: usize = N_WIDTH_LANES - 1;

const BORDER_WIDTH: usize = 2;
const WINDOW_HEIGHT: usize = GAME_AREA_HEIGHT + BORDER_WIDTH * 2;
const WINDOW_WIDTH: usize = GAME_AREA_WIDTH + BORDER_WIDTH * 2;

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

impl std::ops::Add for LanePosition {
    type Output = LanePosition;

    fn add(self, other: LanePosition) -> LanePosition {
        LanePosition {
            x: self.x + other.x,
            y: self.y + other.y,
        }
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

    fn get_color(_tt: TetraminoType) -> [f32; 4] {
        return BLACK;
    }
}

struct Tetramino {
    t_type: TetraminoType,
    squares: [LanePosition; 4],
    float_pos: f64,
    pos: LanePosition,
    color: [f32; 4],
    angle: usize
}

impl Square {
    fn render(&self, gl: &mut GlGraphics, args: &RenderArgs, border_width: f64) {
        use graphics::*;
        let square = rectangle::square(
            (self.pos.x * LANE_WIDTH) as f64 + border_width,
            (self.pos.y * LANE_HEIGHT) as f64 + border_width,
            SQUARE_WIDTH as f64);
        gl.draw(args.viewport(), |c, gl| {
            rectangle(self.color, square, c.transform, gl);
        });
    }
}

impl Tetramino {
    fn render(&self, gl: &mut GlGraphics, args: &RenderArgs, border_width: f64) {
        use graphics::*;
        let render_squares = self.squares.iter().map(|rel_pos| {
            rectangle::square(
            ((rel_pos.x + self.pos.x) * LANE_WIDTH) as f64 + border_width,
            ((rel_pos.y + self.pos.y) * LANE_HEIGHT) as f64 + border_width,
            SQUARE_WIDTH as f64)});
        gl.draw(args.viewport(), |c, gl| {
            for rsq in render_squares {
                rectangle(self.color, rsq, c.transform, gl);
            }
        });
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

    fn get_rel_pos(t_type: TetraminoType) -> [LanePosition; 4] {
        use TetraminoType::*;
        match t_type {
            Line => [LanePosition{x: 0, y: 0},
                     LanePosition{x: 1, y: 0},
                     LanePosition{x: 2, y: 0},
                     LanePosition{x: 3, y: 0}],
            Sq => [LanePosition{x: 0, y: 0},
                     LanePosition{x: 1, y: 0},
                     LanePosition{x: 0, y: 1},
                     LanePosition{x: 1, y: 1}],
            T => [LanePosition{x: 0, y: 0},
                     LanePosition{x: 1, y: 0},
                     LanePosition{x: 2, y: 0},
                     LanePosition{x: 1, y: 1}],
            L => [LanePosition{x: 0, y: 0},
                     LanePosition{x: 1, y: 0},
                     LanePosition{x: 2, y: 0},
                     LanePosition{x: 0, y: 1}],
            J => [LanePosition{x: 0, y: 0},
                     LanePosition{x: 1, y: 0},
                     LanePosition{x: 2, y: 0},
                     LanePosition{x: 2, y: 1}],
            S => [LanePosition{x: 1, y: 0},
                     LanePosition{x: 2, y: 0},
                     LanePosition{x: 0, y: 1},
                     LanePosition{x: 1, y: 1}],
            Z => [LanePosition{x: 0, y: 0},
                     LanePosition{x: 1, y: 0},
                     LanePosition{x: 1, y: 1},
                     LanePosition{x: 2, y: 1}],
        }
    }

    fn new() -> Tetramino {
        let t_type = Tetramino::get_new_type();
        let start = Tetramino::get_new_start_pos(t_type);
        let squares = Tetramino::get_rel_pos(t_type);
        let col = TetraminoType::get_color(t_type);
        return Tetramino{t_type: t_type, squares: squares, float_pos: 0.0, pos: start, color: col, rot: 0.0};
    }

    fn move_left(&mut self) {
        self.pos.decr_x();
    }

    fn move_down(&mut self) {
        self.pos.incr_y();
    }

    fn move_right(&mut self) {
        self.pos.incr_x();
    }

    fn rot_right(&mut self) {
    }

    fn rot_left(&mut self) {
    }
}

struct GameState {
    move_right: bool,
    move_left: bool,
    rot_right: bool,
    rot_left: bool,
    paused: bool,
}

struct App {
    gl: GlGraphics,
    square_slots: [[Option<Square>; N_HEIGHT_LANES]; N_WIDTH_LANES],
    tetramino: Tetramino,
    mov_speed: f64,
    state: GameState,
}

impl App {
    fn render(&mut self, args: &RenderArgs) {
        use graphics::*;

        self.gl.draw(args.viewport(), |_, gl| {
            clear(GRAY, gl);
        });

        self.tetramino.render(&mut self.gl, args, BORDER_WIDTH as f64);

        for it in self.square_slots.iter() {
            for opt in it.iter() {
                if let &Some(ssq) = opt {
                    ssq.render(&mut self.gl, args, BORDER_WIDTH  as f64);
                }
            }
        }
    }

    fn is_done(&self, t: &Tetramino) -> bool {
        t.squares.iter().any(|rel_pos| {
            return t.pos.y + rel_pos.y == LAST_HEIGHT_POS
                || self.square_slots[t.pos.x + rel_pos.x][t.pos.y + rel_pos.y+1].is_some();
        })
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

                for r in (0..j).rev() {
                    for c in 0..N_WIDTH_LANES {
                        if let Some(mut sq) = self.square_slots[c][r] {
                            if self.square_slots[c][r+1].is_none() {
                                // TODO: Use references instead
                                sq.pos.incr_y();
                                self.square_slots[c][r+1] = Some(sq);
                                self.square_slots[c][r] = None;
                            }
                        }
                    }
                }
            }
        }
    }

    fn decompose_tetramino(&mut self, tetra: Tetramino) {
        for rel_pos in tetra.squares.iter() {
			let global_sq_pos = tetra.pos + *rel_pos;
            assert_eq!(self.square_slots[global_sq_pos.x][global_sq_pos.y].is_none(), true);
            self.square_slots[global_sq_pos.x][global_sq_pos.y] = Some(Square{pos: global_sq_pos, color: tetra.color});
        }
    }

    fn move_left(&mut self) {
        let can_move_left = self.tetramino.pos.x != 0 && !self.tetramino.squares.iter().any(|rel_pos| {
            let pos = self.tetramino.pos + *rel_pos;
            return self.square_slots[pos.x - 1][pos.y].is_some();
        });

        if can_move_left {
            self.tetramino.move_left();
        }

        self.state.move_left = false;
    }

    fn is_at_right_edge(&self) -> bool {
        return self.tetramino.pos.x + TetraminoType::get_width(self.tetramino.t_type) - 1 == LAST_WIDTH_POS;
    }

    fn move_right(&mut self) {
        let can_move_right = !self.is_at_right_edge() && !self.tetramino.squares.iter().any(|rel_pos| {
            let pos = self.tetramino.pos + *rel_pos;
            return self.square_slots[pos.x + 1][pos.y].is_some();
        });

        if can_move_right {
            self.tetramino.move_right();
        }

        self.state.move_right = false;
    }

    fn update(&mut self, args: &UpdateArgs) {
        if self.state.paused {
            return;
        }

        if self.state.move_right {
            self.move_right()
        }

        if self.state.move_left {
            self.move_left()
        }

        if self.state.rot_right {
            self.rot_right();
        }

        if self.state.rot_left {
            self.rot_left();
        }

        if self.is_done(&self.tetramino) {
            let old_tetra = std::mem::replace(&mut self.tetramino, Tetramino::new());
            self.decompose_tetramino(old_tetra);
            self.clean_filled_rows();
        }

        self.tetramino.float_pos += self.mov_speed * args.dt;
        if self.tetramino.float_pos > LANE_HEIGHT as f64{
            self.tetramino.float_pos = 0.0;
            self.tetramino.move_down();
        }
    }

    fn handle_button_input(&mut self, args: &ButtonArgs) {
        if args.state == ButtonState::Press {
            if args.button == Button::Keyboard(Key::Left) {
                self.state.move_left = true;
            } else if args.button == Button::Keyboard(Key::Right) {
                self.state.move_right = true;
            } else if args.button == Button::Keyboard(Key::Down) {
                self.mov_speed = MAX_MOVE_SPEED;
            } else if args.button == Button::Keyboard(Key::Space) {
                self.state.paused = !self.state.paused;
            } else if args.button == Button::Keyboard(Key::A) {
                self.state.rot_left = true;
            } else if args.button == Button::Keyboard(Key::D) {
                self.state.rot_right = true;
            }
        } else if args.state == ButtonState::Release {
            if args.button == Button::Keyboard(Key::Down) {
                self.mov_speed = BASE_MOVE_SPEED;
            }
        }
    }

    fn new(ggl: GlGraphics) -> App {
        App {
            gl: ggl,
            square_slots: [[None; N_HEIGHT_LANES]; N_WIDTH_LANES],
            tetramino: Tetramino::new(),
            mov_speed: BASE_MOVE_SPEED,
            state: GameState{move_right: false, move_left: false, paused: false},
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
    let mut app = App::new(GlGraphics::new(opengl)); {
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

