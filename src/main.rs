use druid::{
    widget::Padding, kurbo::Affine, AppLauncher, BoxConstraints, Color, Data, Env, Event,
    EventCtx, LayoutCtx, LifeCycle, LifeCycleCtx, PaintCtx, Point, Rect, RenderContext, Size,
    TimerToken, UpdateCtx, Widget, WindowDesc,
};
use rand::prelude::*;
use std::time::{Duration, Instant};

/// The "design-time" base dimensions of our scene.
const BASE_WIDTH: f64 = 600.0;
const BASE_HEIGHT: f64 = 600.0;

/// Minimal data type implementing `Data`.
#[derive(Clone, Data)]
struct AppData;

/// A single Christmas light on the tree.
#[derive(Clone)]
struct Light {
    position: Point,
    color: Color,
}

/// A single snowflake in the scene.
#[derive(Clone)]
struct Snowflake {
    position: Point,
    speed: f64,
}

struct ChristmasTreeWidget {
    lights: Vec<Light>,
    snowflakes: Vec<Snowflake>,
    timer_id: TimerToken,
    last_update: Instant,
}

impl ChristmasTreeWidget {
    fn new() -> Self {
        let mut widget = ChristmasTreeWidget {
            lights: Vec::new(),
            snowflakes: Vec::new(),
            timer_id: TimerToken::INVALID,
            last_update: Instant::now(),
        };

        widget.lights = widget.generate_lights();
        widget.snowflakes = widget.generate_snowflakes();
        widget
    }

    /// Create random lights within the triangle of the tree.
    fn generate_lights(&self) -> Vec<Light> {
        let mut rng = thread_rng();
        let mut lights = Vec::with_capacity(50);
        // The triangle corners in base coordinates
        let top = Point::new(BASE_WIDTH / 2.0, 50.0);
        let left = Point::new(100.0, BASE_HEIGHT - 50.0);
        let right = Point::new(BASE_WIDTH - 100.0, BASE_HEIGHT - 50.0);

        for _ in 0..50 {
            let r1: f64 = rng.gen();
            let r2: f64 = rng.gen();
            // Barycentric trick
            let (u, v) = if r1 + r2 > 1.0 {
                (1.0 - r1, 1.0 - r2)
            } else {
                (r1, r2)
            };
            let x = top.x + u * (left.x - top.x) + v * (right.x - top.x);
            let y = top.y + u * (left.y - top.y) + v * (right.y - top.y);

            let color = Color::rgb(rng.gen(), rng.gen(), rng.gen());
            lights.push(Light {
                position: Point::new(x, y),
                color,
            });
        }
        lights
    }

    /// Create snowflake positions slightly above the top edge.
    fn generate_snowflakes(&self) -> Vec<Snowflake> {
        let mut rng = thread_rng();
        let mut flakes = Vec::with_capacity(100);

        for _ in 0..100 {
            let x = rng.gen_range(0.0..BASE_WIDTH);
            let y = rng.gen_range(-100.0..0.0);
            let speed = rng.gen_range(1.0..3.0);
            flakes.push(Snowflake {
                position: Point::new(x, y),
                speed,
            });
        }
        flakes
    }
}

impl Widget<AppData> for ChristmasTreeWidget {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, _data: &mut AppData, _env: &Env) {
        match event {
            Event::WindowConnected => {
                // Start our 100ms timer for animation
                self.timer_id = ctx.request_timer(Duration::from_millis(100));
                self.last_update = Instant::now();
            }
            Event::Timer(id) if *id == self.timer_id => {
                let mut rng = thread_rng();

                // Randomly change some light colors
                for light in &mut self.lights {
                    if rng.gen::<f64>() < 0.25 {
                        light.color = Color::rgb(rng.gen(), rng.gen(), rng.gen());
                    }
                }
                // Move snowflakes downward
                for flake in &mut self.snowflakes {
                    flake.position.y += flake.speed;
                    // If below the base "floor," reset to near top
                    if flake.position.y > BASE_HEIGHT {
                        flake.position.y = rng.gen_range(-100.0..0.0);
                        flake.position.x = rng.gen_range(0.0..BASE_WIDTH);
                    }
                }

                // Request next timer
                self.timer_id = ctx.request_timer(Duration::from_millis(100));
                ctx.request_paint();
            }
            _ => (),
        }
    }

    fn lifecycle(
        &mut self,
        _ctx: &mut LifeCycleCtx,
        _event: &LifeCycle,
        _data: &AppData,
        _env: &Env,
    ) {
        // Not used here
    }

    fn update(
        &mut self,
        _ctx: &mut UpdateCtx,
        _old_data: &AppData,
        _data: &AppData,
        _env: &Env,
    ) {
        // Not used; we drive changes in the event method.
    }

    fn layout(
        &mut self,
        _ctx: &mut LayoutCtx,
        bc: &BoxConstraints,
        _data: &AppData,
        _env: &Env,
    ) -> Size {
        // Take all available space
        bc.max()
    }

    fn paint(&mut self, ctx: &mut PaintCtx, _data: &AppData, _env: &Env) {
        let bounds = ctx.size();

        // 1) Fill the entire window with a blue background.
        let sky_blue = Color::rgb8(135, 206, 235);
        ctx.fill(bounds.to_rect(), &sky_blue);

        // 2) Compute a uniform scale factor to preserve 1:1 aspect ratio
        //    of our base 600×600 scene, no matter the window size.
        let scale = f64::min(bounds.width / BASE_WIDTH, bounds.height / BASE_HEIGHT);

        // 3) Compute offsets so the scaled scene is centered.
        let offset_x = (bounds.width - (BASE_WIDTH * scale)) / 2.0;
        let offset_y = (bounds.height - (BASE_HEIGHT * scale)) / 2.0;

        // 4) Use a temporary transform to scale + center the base scene
        //    (tree, lights, snow) onto the current window.
        ctx.with_save(|ctx| {
            // First translate to the offset, then scale uniformly.
            let translation = Affine::translate((offset_x, offset_y));
            let scaling = Affine::scale(scale);
            ctx.transform(translation);
            ctx.transform(scaling);

            // --- Everything drawn below is in BASE_WIDTH×BASE_HEIGHT space ---

            // Draw snowflakes
            for flake in &self.snowflakes {
                ctx.fill(
                    Rect::from_center_size(flake.position, Size::new(4.0, 4.0)),
                    &Color::WHITE,
                );
            }

            // Draw the tree trunk (brown rectangle)
            let trunk_color = Color::rgb8(139, 69, 19); // saddlebrown
            let trunk_width = 30.0;
            let trunk_height = 60.0;
            let trunk_rect = Rect::from_center_size(
                Point::new(BASE_WIDTH / 2.0, BASE_HEIGHT - 60.0),
                Size::new(trunk_width, trunk_height),
            );
            ctx.fill(trunk_rect, &trunk_color);

            // Draw the main tree (triangle)
            let tree_color = Color::rgb8(0, 100, 0); // dark green
            let mut path = druid::kurbo::BezPath::new();
            path.move_to((BASE_WIDTH / 2.0, 50.0));           // top
            path.line_to((100.0, BASE_HEIGHT - 50.0));        // bottom-left
            path.line_to((BASE_WIDTH - 100.0, BASE_HEIGHT - 50.0)); // bottom-right
            path.close_path();
            ctx.fill(path, &tree_color);

            // Draw lights
            for light in &self.lights {
                ctx.fill(
                    Rect::from_center_size(light.position, Size::new(8.0, 8.0)),
                    &light.color,
                );
            }
        });
    }
}

fn main() {
    let window = WindowDesc::new(build_ui())
        // Start with a 600×600 window, but the user can resize freely.
        .window_size(Size::new(BASE_WIDTH, BASE_HEIGHT))
        .title("Christmas Tree");

    let data = AppData;

    AppLauncher::with_window(window)
        .log_to_console()
        .launch(data)
        .expect("Failed to launch application");
}

fn build_ui() -> impl Widget<AppData> {
    Padding::new(0.0, ChristmasTreeWidget::new())
}
