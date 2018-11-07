use ggez::conf;
use ggez::event;
use ggez::graphics;
use ggez::graphics::{DrawMode, Point2};
use ggez::{Context, GameResult};
use std::collections::VecDeque;
use std::time::{Instant, Duration};

const BOARD_HEIGHT: f32 = 600.0;
const BOARD_WIDTH: f32 = 600.0;
const CELL_RADIUS: f32 = 100.0;
const CELL_DIAMETER: f32 = 2.0 * CELL_RADIUS;

#[derive(Copy, Clone)]
struct SnakeCell {
    x: f32,
    y: f32,
    r: f32,
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
            x: rand::random::<f32>() * BOARD_WIDTH,
            y: rand::random::<f32>() * BOARD_HEIGHT,
            r: CELL_RADIUS,
        }
    }
    fn eaten(&mut self) {
        self.x = rand::random::<f32>() * BOARD_WIDTH;
        self.y = rand::random::<f32>() * BOARD_HEIGHT;
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
}

struct Snake {
    body: VecDeque<SnakeCell>,
    curr_dir: Direction,
}

enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Snake {
    fn new() -> Snake {
        Snake {
            body: vec![SnakeCell::new(CELL_RADIUS, 380.0), SnakeCell::new(100.0 + CELL_DIAMETER, 380.0)].into_iter().collect(),
            curr_dir: Direction::Right,
        }
    }
    fn advance(&mut self, food: bool, dir: Direction) {
        if !food { self.body.pop_back(); };
        let head = *self.body.front().unwrap();
        let new_head = match dir {
            Direction::Up => SnakeCell { y: head.y + CELL_DIAMETER, ..head },
            Direction::Down => SnakeCell { y: head.y - CELL_DIAMETER, ..head },
            Direction::Left => SnakeCell { x: head.x - CELL_DIAMETER, ..head },
            Direction::Right => SnakeCell { x: head.x + CELL_DIAMETER, ..head },
        };
        self.body.push_front(new_head);
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
        let s = MainState { frames: 0, snake: Snake::new(), apple: Apple::new(), last_move: Instant::now() };
        Ok(s)
    }
}

impl event::EventHandler for MainState {
    fn update(&mut self, _ctx: &mut Context) -> GameResult<()> {
        if self.last_move.elapsed() > Duration::from_secs(5) {
            self.last_move = Instant::now();
            self.snake.advance(false, Direction::Up);
        }
        
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx);

        for cell in &self.snake.body {
            graphics::circle(
                ctx,
                DrawMode::Fill,
                Point2::new(cell.x, cell.y),
                cell.r,
                0.1
            )?;
        }
        
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