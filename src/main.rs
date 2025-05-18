extern crate piston_window;
extern crate rand;

use piston_window::*;
use rand::Rng;
use std::collections::LinkedList;
use ::image::io::Reader as ImageReader;
use ::image::ImageFormat;

const GRID_SIZE: (i32, i32) = (20, 20); // 20x20 grid
const CELL_SIZE: i32 = 32; // Each cell is 32x32 pixels

// Make the window big enough to show the border outside the playfield
const BORDER_THICKNESS: f64 = 16.0;
const BORDER_FULL: f64 = BORDER_THICKNESS * 2.0;
const WINDOW_SIZE: [u32; 2] = [
    (GRID_SIZE.0 * CELL_SIZE) as u32 + (BORDER_FULL as u32) * 2,
    (GRID_SIZE.1 * CELL_SIZE) as u32 + (BORDER_FULL as u32) * 2,
];

#[derive(Clone, PartialEq)]
enum Direction {
    Left, Right, Up, Down
}

#[derive(PartialEq)]
enum GameState {
    Start,
    Running,
    GameOver,
}

struct Game {
    snake: Snake,
    food: (i32, i32),
    score: u32,
    high_score: u32,
    state: GameState,
}

struct Snake {
    body: LinkedList<(i32, i32)>,
    dir: Direction,
    grow_on_next: bool,
}

impl Game {
    fn render<G: Graphics<Texture = piston_window::G2dTexture>>(&self, c: Context, g: &mut G, glyphs: &mut Glyphs) {
        // Use a brighter, more saturated copper for the background to increase vibrancy
        let copper_bg: [f32; 4] = [0.65, 0.40, 0.18, 1.0];
        let food_color: [f32; 4] = [0.95, 0.64, 0.37, 1.0];
        let border_color: [f32; 4] = [0.25, 0.13, 0.05, 1.0]; // darker border
        clear(copper_bg, g);

        // Draw dark border AROUND the playfield (outside the grid, not overlapping any cell)
        let w = (GRID_SIZE.0 * CELL_SIZE) as f64;
        let h = (GRID_SIZE.1 * CELL_SIZE) as f64;
        let thickness = BORDER_THICKNESS;

        // Top (thick for score text)
        let border_height = thickness * 2.0;

        // All borders now use border_height
        rectangle(border_color, [0.0, 0.0, w + border_height * 2.0, border_height], c.transform.trans(0.0, 0.0), g); // Top
        rectangle(border_color, [0.0, 0.0, w + border_height * 2.0, border_height], c.transform.trans(0.0, h + border_height), g); // Bottom
        rectangle(border_color, [0.0, 0.0, border_height, h + border_height * 2.0], c.transform.trans(0.0, 0.0), g); // Left
        rectangle(border_color, [0.0, 0.0, border_height, h + border_height * 2.0], c.transform.trans(w + border_height, 0.0), g); // Right

        // Shift playfield drawing to border_height so grid is inside border
        let playfield_transform = c.transform.trans(border_height, border_height);
        if self.state == GameState::Running {
            let food_square = [
                (self.food.0 * CELL_SIZE) as f64,
                (self.food.1 * CELL_SIZE) as f64,
                CELL_SIZE as f64,
                CELL_SIZE as f64,
            ];
            rectangle(food_color, food_square, playfield_transform, g);
            self.snake.render(Context { transform: playfield_transform, ..c }, g);
        }

        // Draw overlays
        let text_color: [f32; 4] = [0.95, 0.85, 0.65, 1.0];
        let win_w = WINDOW_SIZE[0] as f64;
        let win_h = WINDOW_SIZE[1] as f64;

        // Helper for true centering: measure text width
        use piston_window::CharacterCache;
        match self.state {
            GameState::Start => {
                let title = "COPPERHEAD";
                let prompt = "Press space to start";
                let title_width = glyphs.width(48, title).unwrap_or(0.0);
                let prompt_width = glyphs.width(24, prompt).unwrap_or(0.0);
                let win_center_x = win_w / 2.0;
                let win_center_y = win_h / 2.0;
                text(text_color, 48, title, glyphs, c.transform.trans(win_center_x - title_width / 2.0, win_center_y - 60.0), g).ok();

                // Draw a preview of the snake under the title
                draw_snake_preview(c, g);

                // Move the prompt further down, under the snake preview
                let prompt_y = win_center_y + (CELL_SIZE as f64) + 50.0;
                text(text_color, 24, prompt, glyphs, c.transform.trans(win_center_x - prompt_width / 2.0, prompt_y), g).ok();
            },
            GameState::Running => {
                let score_str = format!("{}", self.score);
                let score_width = glyphs.width(24, &score_str).unwrap_or(0.0);
                text(text_color, 24, &score_str, glyphs, c.transform.trans(win_w / 2.0 - score_width / 2.0, border_height * 0.75), g).ok();
            },
            GameState::GameOver => {
                // Red-tinted background for game over
                let red_overlay: [f32; 4] = [0.6, 0.1, 0.1, 1.0];
                clear(red_overlay, g);

                // Draw playfield and snake in final position (no food)
                let w = (GRID_SIZE.0 * CELL_SIZE) as f64;
                let h = (GRID_SIZE.1 * CELL_SIZE) as f64;
                let border_height = BORDER_THICKNESS * 2.0;
                let border_color: [f32; 4] = [0.25, 0.13, 0.05, 1.0];

                // Borders
                rectangle(border_color, [0.0, 0.0, w + border_height * 2.0, border_height], c.transform.trans(0.0, 0.0), g); // Top
                rectangle(border_color, [0.0, 0.0, w + border_height * 2.0, border_height], c.transform.trans(0.0, h + border_height), g); // Bottom
                rectangle(border_color, [0.0, 0.0, border_height, h + border_height * 2.0], c.transform.trans(0.0, 0.0), g); // Left
                rectangle(border_color, [0.0, 0.0, border_height, h + border_height * 2.0], c.transform.trans(w + border_height, 0.0), g); // Right
                let playfield_transform = c.transform.trans(border_height, border_height);
                self.snake.render(Context { transform: playfield_transform, ..c }, g);

                // Overlay text
                let text_color: [f32; 4] = [0.95, 0.85, 0.65, 1.0];
                let win_w = WINDOW_SIZE[0] as f64;
                let win_h = WINDOW_SIZE[1] as f64;
                let over = "COILED!";
                let score_str = format!("Score: {}", self.score);
                let high_str = format!("Highest: {}", self.high_score);
                let prompt = "Press space to restart";
                let over_width = glyphs.width(48, over).unwrap_or(0.0);
                let score_width = glyphs.width(24, &score_str).unwrap_or(0.0);
                let high_width = glyphs.width(24, &high_str).unwrap_or(0.0);
                let prompt_width = glyphs.width(20, prompt).unwrap_or(0.0);
                text(text_color, 48, over, glyphs, c.transform.trans(win_w / 2.0 - over_width / 2.0, win_h / 2.0 - 40.0), g).ok();
                text(text_color, 24, &score_str, glyphs, c.transform.trans(win_w / 2.0 - score_width / 2.0, win_h / 2.0 + 20.0), g).ok();
                text(text_color, 24, &high_str, glyphs, c.transform.trans(win_w / 2.0 - high_width / 2.0, win_h / 2.0 + 60.0), g).ok();
                text(text_color, 20, prompt, glyphs, c.transform.trans(win_w / 2.0 - prompt_width / 2.0, win_h / 2.0 + 110.0), g).ok();
            }
        }
    }

    fn update(&mut self) {
        // Don't update if game is not running
        if self.state != GameState::Running {
            return;
        }

        // Food
        let ate = self.snake.update(self.food);
        if ate {
            self.score += 1;
            self.snake.grow();
            self.spawn_food();
        }

        // Check wall collision (now with border thickness)
        let (x, y) = self.snake.head();
        if x < 0 || x >= GRID_SIZE.0 || y < 0 || y >= GRID_SIZE.1 || self.snake.self_collision() {
            self.state = GameState::GameOver;
            if self.score > self.high_score {
                self.high_score = self.score;
            }
        }
    }

    // Handle key presses
    fn pressed(&mut self, btn: &Button) {
        match self.state {
            GameState::Start => {
                if let &Button::Keyboard(Key::Space) = btn {
                    self.state = GameState::Running;
                }
            },
            GameState::GameOver => {
                if let &Button::Keyboard(Key::Space) = btn {
                    self.reset();
                }
            },
            GameState::Running => {
                let last_direction = self.snake.dir.clone();
                self.snake.dir = match btn {
                    &Button::Keyboard(Key::Up)
                        if last_direction != Direction::Down => Direction::Up,
                    &Button::Keyboard(Key::Down)
                        if last_direction != Direction::Up => Direction::Down,
                    &Button::Keyboard(Key::Left)
                        if last_direction != Direction::Right => Direction::Left,
                    &Button::Keyboard(Key::Right)
                        if last_direction != Direction::Left => Direction::Right,
                    _ => last_direction,
                };
            }
        }
    }

    fn spawn_food(&mut self) {
        let mut rng = rand::thread_rng();
        loop {
            let pos = (
                rng.gen_range(0..GRID_SIZE.0),
                rng.gen_range(0..GRID_SIZE.1),
            );
            if !self.snake.body.contains(&pos) {
                self.food = pos;
                break;
            }
        }
    }

    fn reset(&mut self) {
        self.snake = Snake::new();
        self.score = 0;
        self.state = GameState::Start;
        self.spawn_food();
    }
}

impl Snake {
    fn new() -> Self {
        let mut body = LinkedList::new();
        let y = GRID_SIZE.1 / 2;
        let x = GRID_SIZE.0 / 2;
        body.push_back((x, y));
        body.push_back((x - 1, y));
        body.push_back((x - 2, y));
        Snake {
            body,
            dir: Direction::Right,
            grow_on_next: false,
        }
    }
    fn render<G: Graphics>(&self, c: Context, g: &mut G) {
        let head_color: [f32; 4] = [0.90, 0.60, 0.25, 1.0]; // More coppery head
        let eye_color: [f32; 4] = [0.1, 0.1, 0.1, 1.0];
        let mut iter = self.body.iter();
        if let Some(&(x, y)) = iter.next() {
            let head_square = [
                (x * CELL_SIZE) as f64,
                (y * CELL_SIZE) as f64,
                CELL_SIZE as f64,
                CELL_SIZE as f64,
            ];

            // Always draw the head, even if it overlaps the body (game over)
            rectangle(head_color, head_square, c.transform, g);

            // Fake reflection: draw a lighter, semi-transparent rectangle on the upper left of the head
            let reflection_color: [f32; 4] = [1.0, 0.95, 0.80, 0.35];
            let refl_w = CELL_SIZE as f64 * 0.45;
            let refl_h = CELL_SIZE as f64 * 0.18;
            let refl_x = (x * CELL_SIZE) as f64 + CELL_SIZE as f64 * 0.10;
            let refl_y = (y * CELL_SIZE) as f64 + CELL_SIZE as f64 * 0.10;
            rectangle(reflection_color, [refl_x, refl_y, refl_w, refl_h], c.transform, g);

            // Eyes (move slightly to the front of the head)
            let cx = (x * CELL_SIZE) as f64 + CELL_SIZE as f64 / 2.0;
            let cy = (y * CELL_SIZE) as f64 + CELL_SIZE as f64 / 2.0;
            let eye_r = CELL_SIZE as f64 * 0.1;
            let eye_offset_x = CELL_SIZE as f64 * 0.20;
            let eye_offset_y = CELL_SIZE as f64 * 0.20;
            let front_offset = CELL_SIZE as f64 * 0.18;

            let (eye1, eye2) = match self.dir {
                Direction::Up => (
                    [cx - eye_offset_x, cy - front_offset],
                    [cx + eye_offset_x, cy - front_offset],
                ),
                Direction::Down => (
                    [cx - eye_offset_x, cy + front_offset],
                    [cx + eye_offset_x, cy + front_offset],
                ),
                Direction::Left => (
                    [cx - front_offset, cy - eye_offset_y],
                    [cx - front_offset, cy + eye_offset_y],
                ),
                Direction::Right => (
                    [cx + front_offset, cy - eye_offset_y],
                    [cx + front_offset, cy + eye_offset_y],
                ),
            };

            // Draw eyes as little squares instead of ellipses
            let eye_size = eye_r * 2.0;
            rectangle(eye_color, [eye1[0] - eye_r, eye1[1] - eye_r, eye_size, eye_size], c.transform, g);
            rectangle(eye_color, [eye2[0] - eye_r, eye2[1] - eye_r, eye_size, eye_size], c.transform, g);

            // Draw the rest of the body, skipping any segment at the head's position
            for (i, &(bx, by)) in iter.enumerate() {
                if bx == x && by == y {
                    continue; // skip body segment that overlaps the head
                }
                // Alternate color: even index = dark, odd index = light
                let body_color = if i % 2 == 0 {
                    [0.60, 0.30, 0.10, 1.0] // darker copper
                } else {
                    [0.85, 0.55, 0.22, 1.0] // lighter copper
                };
                let square = [
                    (bx * CELL_SIZE) as f64,
                    (by * CELL_SIZE) as f64,
                    CELL_SIZE as f64,
                    CELL_SIZE as f64,
                ];
                rectangle(body_color, square, c.transform, g);
            }
        }
    }
    fn update(&mut self, food: (i32, i32)) -> bool {
        let mut new_head = *self.body.front().expect("Snake has no body");
        match self.dir {
            Direction::Left => new_head.0 -= 1,
            Direction::Right => new_head.0 += 1,
            Direction::Up => new_head.1 -= 1,
            Direction::Down => new_head.1 += 1,
        }
        self.body.push_front(new_head);
        let ate = new_head == food;
        if !ate && !self.grow_on_next {
            self.body.pop_back();
        } else if self.grow_on_next {
            self.grow_on_next = false;
        }
        ate
    }
    fn grow(&mut self) {
        self.grow_on_next = true;
    }
    fn head(&self) -> (i32, i32) {
        *self.body.front().unwrap()
    }
    fn self_collision(&self) -> bool {
        let head = self.head();
        self.body.iter().skip(1).any(|&pos| pos == head)
    }
}

// Center the window
fn center_window(window: &mut PistonWindow) {
    use winit::dpi::{PhysicalPosition, PhysicalSize};
    use winit::monitor::MonitorHandle;

    let primary_monitor: MonitorHandle = window.window.ctx.window().current_monitor().unwrap();
    let monitor_size: PhysicalSize<u32> = primary_monitor.size();
    let win_size = window.size();
    let x = (monitor_size.width.saturating_sub(win_size.width as u32)) / 2;
    let y = (monitor_size.height.saturating_sub(win_size.height as u32)) / 2;
    window.window.ctx.window().set_outer_position(PhysicalPosition::new(x as u32, y as u32));
}

fn draw_snake_preview<G: Graphics>(c: Context, g: &mut G) {
    // Compute the center of the playfield in pixels (relative to window)
    let border_height = BORDER_THICKNESS * 2.0;
    let playfield_x = border_height;
    let playfield_y = border_height;
    let center_cell_x = GRID_SIZE.0 / 2;
    let center_cell_y = GRID_SIZE.1 / 2;
    let preview_cell = CELL_SIZE as f64;

    // The snake is 3 long, horizontal, head to the right
    // The leftmost segment is at (center_cell_x - 2, center_cell_y)
    let preview_x = playfield_x + ((center_cell_x - 2) as f64) * preview_cell;
    let preview_y = playfield_y + (center_cell_y as f64) * preview_cell;

    for i in 0..3 {
        // Draw from right to left so the head is at the right, body extends to the left
        let bx = preview_x + (2 - i) as f64 * preview_cell;
        let by = preview_y;
        let is_head = i == 0;
        let color = if is_head {
            [0.90, 0.60, 0.25, 1.0]
        } else if i == 1 {
            // Neck (directly behind head) is always darker copper
            [0.60, 0.30, 0.10, 1.0]
        } else {
            // Tail (alternates, but for 3-length snake, this is lighter copper)
            [0.85, 0.55, 0.22, 1.0]
        };
        rectangle(color, [bx, by, preview_cell, preview_cell], c.transform, g);
        if is_head {
            // Reflection
            let reflection_color: [f32; 4] = [1.0, 0.95, 0.80, 0.35];
            let refl_w = preview_cell * 0.45;
            let refl_h = preview_cell * 0.18;
            let refl_x = bx + preview_cell * 0.10;
            let refl_y = by + preview_cell * 0.10;
            rectangle(reflection_color, [refl_x, refl_y, refl_w, refl_h], c.transform, g);

            // Eyes (facing right)
            let cx = bx + preview_cell / 2.0;
            let cy = by + preview_cell / 2.0;
            let eye_r = preview_cell * 0.1;
            let eye_offset_y = preview_cell * 0.20;
            let front_offset = preview_cell * 0.18;
            let eye1 = [cx + front_offset, cy - eye_offset_y];
            let eye2 = [cx + front_offset, cy + eye_offset_y];
            let eye_color = [0.1, 0.1, 0.1, 1.0];
            let eye_size = eye_r * 2.0;
            rectangle(eye_color, [eye1[0] - eye_r, eye1[1] - eye_r, eye_size, eye_size], c.transform, g);
            rectangle(eye_color, [eye2[0] - eye_r, eye2[1] - eye_r, eye_size, eye_size], c.transform, g);
        }
    }
}

fn main() {
    let mut window: PistonWindow = WindowSettings::new("Copperhead", WINDOW_SIZE)
        .exit_on_esc(true)
        .build()
        .unwrap();

    window.set_ups(100); // Set updates per second to 100Hz
    window.set_max_fps(120); // Set max frames per second to 120Hz
    window.set_title("Copperhead".to_string());
    center_window(&mut window);

    // Set window icon using winit (requires image crate)
    {
        use winit::window::Icon;
        use std::fs::File;
        use std::io::BufReader;
        let icon_path = "assets/icon.png";
        if let Ok(file) = File::open(icon_path) {
            let reader = BufReader::new(file);
            if let Ok(img) = ImageReader::with_format(reader, ImageFormat::Png).decode() {
                let img = img.into_rgba8();
                let (width, height) = img.dimensions();
                if let Ok(icon) = Icon::from_rgba(img.into_raw(), width, height) {
                    window.window.ctx.window().set_window_icon(Some(icon));
                }
            }
        }
    }

    let assets = std::path::Path::new("assets/JetBrainsMono-Regular.ttf");
    let mut glyphs = window.load_font(assets).expect("Could not load font");

    let mut game = Game {
        snake: Snake::new(),
        food: (5, 5),
        score: 0,
        high_score: 0,
        state: GameState::Start,
    };
    game.spawn_food();

    let mut events = window.events;
    let mut pending_direction: Option<Direction> = None;
    let mut last_update = std::time::Instant::now();
    let mut snake_move_timer = 0.0f64;
    let snake_move_interval = 0.10; // Snake moves every 100ms (10Hz)
    while let Some(e) = events.next(&mut window) {
        if let Some(Button::Keyboard(key)) = e.press_args() {
            // Only queue direction change if not already queued
            if pending_direction.is_none() {
                let dir = match key {
                    Key::Up => Some(Direction::Up),
                    Key::Down => Some(Direction::Down),
                    Key::Left => Some(Direction::Left),
                    Key::Right => Some(Direction::Right),
                    _ => None,
                };
                if let Some(d) = dir {
                    pending_direction = Some(d);
                } else {
                    // For non-direction keys, still call pressed (e.g. Space)
                    game.pressed(&Button::Keyboard(key));
                }
            }
        }
        // Game logic update at fixed interval (100Hz)
        if let Some(_u) = e.update_args() {
            let now = std::time::Instant::now();
            let dt = last_update.elapsed().as_secs_f64();
            last_update = now;
            snake_move_timer += dt;
            // Only move the snake at the slower interval
            if snake_move_timer >= snake_move_interval {
                // Apply pending direction if any
                if let Some(dir) = pending_direction.take() {
                    let last_direction = game.snake.dir.clone();
                    game.snake.dir = match dir {
                        Direction::Up if last_direction != Direction::Down => Direction::Up,
                        Direction::Down if last_direction != Direction::Up => Direction::Down,
                        Direction::Left if last_direction != Direction::Right => Direction::Left,
                        Direction::Right if last_direction != Direction::Left => Direction::Right,
                        _ => last_direction,
                    };
                }
                game.update();
                snake_move_timer -= snake_move_interval;
            }
        }
        // Render as fast as possible
        if let Some(_r) = e.render_args() {
            window.draw_2d(&e, |c, g, device| {
                game.render(c, g, &mut glyphs);
                glyphs.factory.encoder.flush(device);
            });
        }
    }
}
