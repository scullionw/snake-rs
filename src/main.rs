use ggez::audio;
use ggez::conf;
use ggez::event;
use ggez::event::{Keycode, Mod};
use ggez::graphics;
use ggez::graphics::{DrawMode, Point2};
use ggez::{Context, GameResult};
use std::collections::VecDeque;
use std::env;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

const BOARD_WIDTH: f32 = 800.0;
const BOARD_HEIGHT: f32 = 600.0;
const CELL_RADIUS: f32 = 5.0;
const CELL_DIAMETER: f32 = 2.0 * CELL_RADIUS;
const SLOW_SPEED: u64 = 250;
const FAST_SPEED: u64 = 50;

trait Locate {
    fn cartesian(&self) -> (f32, f32);
    fn dist_to<T: Locate>(&self, other: &T) -> f32 {
        let (x1, y1) = self.cartesian();
        let (x2, y2) = other.cartesian();
        (x2 - x1).hypot(y2 - y1)
    }
}

#[derive(Copy, Clone)]
struct SnakeCell {
    x: f32,
    y: f32,
    r: f32,
}

impl Locate for SnakeCell {
    fn cartesian(&self) -> (f32, f32) {
        (self.x, self.y)
    }
}

impl Locate for Apple {
    fn cartesian(&self) -> (f32, f32) {
        (self.x, self.y)
    }
}

#[derive(Copy, Clone)]
struct Apple {
    x: f32,
    y: f32,
    r: f32,
}

impl Apple {
    fn new() -> Apple {
        Apple {
            x: rand::random::<f32>() * (BOARD_WIDTH - CELL_DIAMETER) + CELL_RADIUS,
            y: rand::random::<f32>() * (BOARD_HEIGHT - CELL_DIAMETER) + CELL_RADIUS,
            r: CELL_RADIUS,
        }
    }
    fn eaten(&mut self) {
        self.x = rand::random::<f32>() * (BOARD_WIDTH - CELL_DIAMETER) + CELL_RADIUS;
        self.y = rand::random::<f32>() * (BOARD_HEIGHT - CELL_DIAMETER) + CELL_RADIUS;
    }
    fn draw(&self, ctx: &mut Context) -> GameResult<()> {
        graphics::set_color(ctx, graphics::Color::new(1.0, 0.0, 0.0, 1.0))?;
        graphics::circle(
            ctx,
            DrawMode::Fill,
            Point2::new(self.x, self.y),
            self.r,
            0.1,
        )
    }
}

impl SnakeCell {
    fn new(x: f32, y: f32) -> SnakeCell {
        SnakeCell {
            x,
            y,
            r: CELL_RADIUS,
        }
    }
    fn next_to(&self, dir: Direction) -> SnakeCell {
        match dir {
            Direction::Up => SnakeCell {
                y: self.y - CELL_DIAMETER,
                ..*self
            },
            Direction::Down => SnakeCell {
                y: self.y + CELL_DIAMETER,
                ..*self
            },
            Direction::Left => SnakeCell {
                x: self.x - CELL_DIAMETER,
                ..*self
            },
            Direction::Right => SnakeCell {
                x: self.x + CELL_DIAMETER,
                ..*self
            },
        }
    }
    fn draw(&self, ctx: &mut Context) -> GameResult<()> {
        graphics::circle(
            ctx,
            DrawMode::Fill,
            Point2::new(self.x, self.y),
            self.r,
            0.1,
        )
    }
}

struct Snake {
    body: VecDeque<SnakeCell>,
    curr_dir: Direction,
}

#[derive(Copy, Clone)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Snake {
    fn new() -> Snake {
        let head = SnakeCell::new(CELL_RADIUS, 200.0);
        let body = vec![head, head.next_to(Direction::Right)]
            .into_iter()
            .collect();

        Snake {
            body,
            curr_dir: Direction::Down,
        }
    }
    fn shorten_tail(&mut self) {
        self.body.pop_back();
    }
    fn advance(&mut self) {
        let new_head = self.head().next_to(self.curr_dir);
        self.body.push_front(new_head);
    }
    fn head(&self) -> SnakeCell {
        *self.body.front().unwrap()
    }
    fn draw(&self, ctx: &mut Context) -> GameResult<()> {
        graphics::set_color(ctx, graphics::Color::new(1.0, 1.0, 1.0, 1.0))?;
        for cell in &self.body {
            cell.draw(ctx)?;
        }
        Ok(())
    }
    fn bounds_check(&self, bounds: &Bounds) -> bool {
        self.body.iter().all(|cell| bounds.check(cell.cartesian()))
    }
    fn body_check(&self) -> bool {
        let mut collisions = 0;
        for cell in &self.body {
            if cell.dist_to(&self.head()) < CELL_RADIUS {
                collisions += 1;
            }
        }
        collisions <= 1
    }
}

struct Bounds {
    width: f32,
    height: f32,
}

impl Bounds {
    fn new() -> Bounds {
        Bounds {
            width: BOARD_WIDTH,
            height: BOARD_HEIGHT,
        }
    }
    fn check(&self, coord: (f32, f32)) -> bool {
        coord.0 < self.width && coord.0 > 0.0 && coord.1 < self.height && coord.1 > 0.0
    }
}

struct Score {
    pos: graphics::Point2,
    font: graphics::Font,
    val: u32,
}

impl Score {
    fn new(ctx: &mut Context) -> Score {
        Score {
            pos: graphics::Point2::new(600.0, 20.0),
            font: graphics::Font::new(ctx, "/DejaVuSerif.ttf", 24).unwrap(),
            val: 0,
        }
    }
    fn draw(&self, ctx: &mut Context) -> GameResult<()> {
        let score_text = format!("Score: {}", self.val);
        let text = graphics::Text::new(ctx, score_text.as_str(), &self.font)?;
        graphics::draw(ctx, &text, self.pos, 0.0)?;
        Ok(())
    }
    fn increment(&mut self) {
        self.val += 1;
    }
}

struct MainState {
    snake: Snake,
    apple: Apple,
    bounds: Bounds,
    last_move: Instant,
    last_key_moment: Instant,
    background_music: audio::Source,
    eating_sound: audio::Source,
    game_over_sound: audio::Source,
    score: Score,
    delay: u64,
    game_over: bool,
}

impl MainState {
    fn new(ctx: &mut Context) -> GameResult<MainState> {
        let mut background_music = audio::Source::new(ctx, "/crystals.ogg").unwrap();
        background_music.set_volume(0.4);
        background_music.play().unwrap();
        let s = MainState {
            snake: Snake::new(),
            apple: Apple::new(),
            bounds: Bounds::new(),
            last_move: Instant::now(),
            last_key_moment: Instant::now(),
            background_music,
            eating_sound: audio::Source::new(ctx, "/gulp.ogg").unwrap(),
            game_over_sound: audio::Source::new(ctx, "/gameover.ogg").unwrap(),
            score: Score::new(ctx),
            delay: SLOW_SPEED,
            game_over: false,
        };
        Ok(s)
    }
}

impl event::EventHandler for MainState {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        if !self.game_over {
            if self.last_key_moment.elapsed() >= Duration::from_millis(self.delay) {
                self.delay = SLOW_SPEED;
            }
            if self.last_move.elapsed() >= Duration::from_millis(self.delay) {
                self.last_move = Instant::now();
                self.snake.advance();
                if !self.snake.bounds_check(&self.bounds) || !self.snake.body_check() {
                    self.game_over = true;
                    self.background_music.stop();
                    self.game_over_sound.play().unwrap();
                    while self.game_over_sound.playing() {
                        ggez::timer::yield_now();
                    }
                    ctx.quit()?;
                }
                if self.apple.dist_to(&self.snake.head()) < CELL_DIAMETER {
                    self.eating_sound.play().unwrap();
                    self.apple.eaten();
                    self.score.increment();
                } else {
                    self.snake.shorten_tail();
                }
            }
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx);
        self.snake.draw(ctx)?;
        self.apple.draw(ctx)?;
        self.score.draw(ctx)?;
        graphics::present(ctx);
        ggez::timer::yield_now();
        Ok(())
    }
    fn key_down_event(&mut self, _ctx: &mut Context, keycode: Keycode, _keymod: Mod, repeat: bool) {
        match keycode {
            Keycode::Up => self.snake.curr_dir = Direction::Up,
            Keycode::Left => self.snake.curr_dir = Direction::Left,
            Keycode::Down => self.snake.curr_dir = Direction::Down,
            Keycode::Right => self.snake.curr_dir = Direction::Right,
            _ => (),
        }
        self.delay = if repeat { FAST_SPEED } else { SLOW_SPEED };
        self.last_key_moment = Instant::now();
    }
}

fn resource_path() -> PathBuf {
    match env::var("CARGO_MANIFEST_DIR") {
        Ok(manifest_dir) => Path::new(&manifest_dir).join("resources"),
        Err(_) => PathBuf::from("resources"),
    }
}
pub fn main() {
    let c = conf::Conf::new();
    let ctx = &mut Context::load_from_conf("snake", "ggez", c).unwrap();
    ctx.filesystem.mount(&resource_path(), true);
    let state = &mut MainState::new(ctx).unwrap();
    event::run(ctx, state).unwrap();
}
