use ggez::conf;
use ggez::event;
use ggez::graphics;
use ggez::graphics::{DrawMode, Point2};
use ggez::{Context, GameResult};
use std::collections::VecDeque;
use std::time::{Duration, Instant};

const BOARD_WIDTH: f32 = 800.0;
const BOARD_HEIGHT: f32 = 600.0;
const CELL_RADIUS: f32 = 20.0;
const CELL_DIAMETER: f32 = 2.0 * CELL_RADIUS;

// TODO: ggez::timer::yield, tempo matching article, check out example to compare

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
            x: CELL_RADIUS, //rand::random::<f32>() * (BOARD_WIDTH - CELL_DIAMETER) + CELL_RADIUS,
            y: 400.0,       //rand::random::<f32>() * (BOARD_HEIGHT - CELL_DIAMETER) + CELL_RADIUS,
            r: CELL_RADIUS,
        }
    }
    fn eaten(&mut self) {
        self.x = rand::random::<f32>() * BOARD_WIDTH;
        self.y = rand::random::<f32>() * BOARD_HEIGHT;
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
}

struct MainState {
    frames: usize,
    snake: Snake,
    apple: Apple,
    last_move: Instant,
}

impl MainState {
    fn new(_ctx: &mut Context) -> GameResult<MainState> {
        let s = MainState {
            frames: 0,
            snake: Snake::new(),
            apple: Apple::new(),
            last_move: Instant::now(),
        };
        Ok(s)
    }
}

impl event::EventHandler for MainState {
    fn update(&mut self, _ctx: &mut Context) -> GameResult<()> {
        if self.last_move.elapsed() >= Duration::from_secs(1) {
            self.last_move = Instant::now();
            self.snake.advance();
            if self.apple.dist_to(&self.snake.head()) < CELL_DIAMETER {
                println!("COLLISION!");
                self.apple.eaten();
            } else {
                self.snake.shorten_tail();
            }
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx);
        self.snake.draw(ctx)?;
        self.apple.draw(ctx)?;
        //let dest_point = graphics::Point2::new(10.0, 10.0);
        // graphics::draw(ctx, &self.text, dest_point, 0.0)?;
        graphics::present(ctx);
        self.frames += 1;
        if (self.frames % 100) == 0 {
            println!("FPS: {}", ggez::timer::get_fps(ctx));
        }
        Ok(())
    }
}

pub fn main() {
    let c = conf::Conf::new();
    let ctx = &mut Context::load_from_conf("snake", "ggez", c).unwrap();
    let state = &mut MainState::new(ctx).unwrap();
    event::run(ctx, state).unwrap();
}
