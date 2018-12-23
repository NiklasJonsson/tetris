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
    fn clamp_width(n: i32) -> usize {
        return std::cmp::min(std::cmp::max(n, 0) as usize, LAST_WIDTH_POS);
    }
    fn clamp_height(n: i32) -> usize {
        return std::cmp::max(n, 0) as usize;
    }

    fn prev_x(self) -> LanePosition {
        LanePosition{x: LanePosition::clamp_width(self.x as i32 - 1), y: self.y}
    }
    fn next_x(self) -> LanePosition {
        LanePosition{x: LanePosition::clamp_width(self.x as i32 + 1), y: self.y}
    }
    fn next_y(self) -> LanePosition {
        LanePosition{x: self.x, y: LanePosition::clamp_height(self.y as i32 + 1)}
    }

    fn decr_x(&mut self) {
        self.x = LanePosition::clamp_width(self.x as i32 - 1);
    }
    fn incr_x(&mut self) {
        self.x = LanePosition::clamp_width(self.x as i32 + 1);
    }
    fn incr_y(&mut self) {
        self.y = LanePosition::clamp_height(self.y as i32 + 1);
    }
}

impl From<(usize, usize)> for LanePosition {
    fn from(inp: (usize, usize)) -> Self { LanePosition{x: inp.0, y: inp.1} }
}

impl std::ops::Add for LanePosition {
    type Output = LanePosition;

    fn add(self, other: LanePosition) -> LanePosition {
        LanePosition{
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl std::ops::Mul for LanePosition {
    type Output = LanePosition;

    fn mul(self, other: LanePosition) -> LanePosition {
        LanePosition{
            x: self.x * other.x,
            y: self.y * other.y,
        }
    }
}

#[derive(Debug, Copy, Clone)]
struct RelPosition {
    x: i32,
    y: i32
}

impl RelPosition {
    fn rotate_clockwise(self) -> Self { RelPosition{x: self.y, y: -self.x} } 
    fn rotate_counter_clockwise(self) -> Self { RelPosition{x: -self.y, y: self.x} } 
    fn move_left(self) -> Self { Self{x: self.x - 1, y: self.y} }
    fn move_right(self) -> Self { Self{x: self.x + 1, y: self.y} }
}

impl std::ops::Add<&RelPosition> for LanePosition {
    type Output = Option<LanePosition>;

    fn add(self, other: &RelPosition) -> Option<LanePosition> {
        let x = self.x as i32 + other.x;
        let y = self.y as i32 + other.y;
        if x < 0 || x > LAST_WIDTH_POS as i32 || y < 0 || y > LAST_HEIGHT_POS as i32 {
            return None;
        }
        Some(LanePosition{x: x as usize, y: y as usize})
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
    squares: [RelPosition; 4],
    float_pos: f64,
    pos: LanePosition,
    color: [f32; 4],
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
            let pos: LanePosition = (self.pos + rel_pos).unwrap() * LanePosition{x: LANE_WIDTH, y: LANE_HEIGHT};
            rectangle::square(
                pos.x as f64 + border_width,
                pos.y as f64 + border_width,
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

    fn get_rel_pos(t_type: TetraminoType) -> [RelPosition; 4] {
        use TetraminoType::*;
        match t_type {
            Line => [RelPosition{x: 0, y: 0},
                     RelPosition{x: 1, y: 0},
                     RelPosition{x: 2, y: 0},
                     RelPosition{x: 3, y: 0}],
            Sq => [RelPosition{x: 0, y: 0},
                     RelPosition{x: 1, y: 0},
                     RelPosition{x: 0, y: 1},
                     RelPosition{x: 1, y: 1}],
            T => [RelPosition{x: 0, y: 0},
                     RelPosition{x: 1, y: 0},
                     RelPosition{x: 2, y: 0},
                     RelPosition{x: 1, y: 1}],
            L => [RelPosition{x: 0, y: 0},
                     RelPosition{x: 1, y: 0},
                     RelPosition{x: 2, y: 0},
                     RelPosition{x: 0, y: 1}],
            J => [RelPosition{x: 0, y: 0},
                     RelPosition{x: 1, y: 0},
                     RelPosition{x: 2, y: 0},
                     RelPosition{x: 2, y: 1}],
            S => [RelPosition{x: 1, y: 0},
                     RelPosition{x: 2, y: 0},
                     RelPosition{x: 0, y: 1},
                     RelPosition{x: 1, y: 1}],
            Z => [RelPosition{x: 0, y: 0},
                     RelPosition{x: 1, y: 0},
                     RelPosition{x: 1, y: 1},
                     RelPosition{x: 2, y: 1}],
        }
    }

    fn new() -> Tetramino {
        let t_type = Tetramino::get_new_type();
        let start = Tetramino::get_new_start_pos(t_type);
        let squares = Tetramino::get_rel_pos(t_type);
        let col = TetraminoType::get_color(t_type);
        return Tetramino{t_type: t_type, squares: squares, float_pos: 0.0, pos: start, color: col};
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

    fn rotate_clockwise(&mut self) {
        for sq in &mut self.squares {
            *sq = sq.rotate_clockwise();
        }
    }

    fn rotate_counter_clockwise(&mut self) {
        for sq in &mut self.squares {
            *sq = sq.rotate_counter_clockwise();
        }
    }
}

// TODO: change to enum and associate with function
struct GameState {
    move_right: bool,
    move_left: bool,
    rotate_clockwise: bool,
    rotate_counter_clockwise: bool,
    paused: bool,
}

struct App {
    gl: GlGraphics,
    square_slots: [[Option<Square>; N_HEIGHT_LANES]; N_WIDTH_LANES],
    tetramino: Tetramino,
    mov_speed: f64,
    state: GameState,
}

 macro_rules! gen_transform {
        ($rotator: ident) => {
            fn $rotator(&mut self) {
                let can_rot = self.tetramino.squares.iter().all(|rel_pos| {
                    let new_pos = self.tetramino.pos + &rel_pos.$rotator();
                    match new_pos {
                        None => false,
                        Some(pos) => !self.has_square_at(pos),
                    }
                });

                if can_rot {
                    self.tetramino.$rotator();
                }
                self.state.$rotator = false;
            }
        }
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

    fn get_square_at(&self, pos: LanePosition) -> Option<Square> { self.square_slots[pos.x][pos.y] }
    fn has_square_at(&self, pos: LanePosition) -> bool { self.square_slots[pos.x][pos.y].is_some() }
    fn assign_square_at(&mut self, pos: LanePosition, sq: Square) {
        assert!(!self.has_square_at(pos));
        self.square_slots[pos.x][pos.y] = Some(sq);
    }


    fn is_done(&self, t: &Tetramino) -> bool {
        t.squares.iter().any(|rel_pos| {
            let pos = (t.pos + rel_pos).unwrap();
            return pos.y == LAST_HEIGHT_POS || self.has_square_at(pos.next_y());
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
			let global_sq_pos = (tetra.pos + rel_pos).unwrap();
            self.assign_square_at(global_sq_pos, Square{pos: global_sq_pos, color: tetra.color});
        }
    }

    gen_transform!(move_left);
    gen_transform!(move_right);
    gen_transform!(rotate_clockwise);
    gen_transform!(rotate_counter_clockwise);

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

        if self.state.rotate_clockwise {
            self.rotate_clockwise();
        }

        if self.state.rotate_counter_clockwise {
            self.rotate_counter_clockwise();
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
                self.state.rotate_counter_clockwise = true;
            } else if args.button == Button::Keyboard(Key::D) {
                self.state.rotate_clockwise = true;
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
            state: GameState{move_right: false, move_left: false,
                             rotate_clockwise: false, rotate_counter_clockwise: false,
                             paused: false},
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

