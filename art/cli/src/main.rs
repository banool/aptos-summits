use nannou::prelude::*;
use nannou::rand::{Rng, rngs::SmallRng};
use nannou::rand::SeedableRng;

// TODO: Make it possible to use different width values. Make sure the output is the
// same with different widths. Perhaps start with a high width like 10k and then remove
// points (or add if for some reason width is greater than 10k).

const WIDTH: u32 = 1600;

// TODO: For the API try window.capture_frame with an in memory thing that is a path.

struct Model {
    /// Mountains from back to front.
    mountains: Vec<Mountain>,
}

impl Model {
    pub fn mountains(&self) -> &[Mountain] {
        &self.mountains
    }
}

fn main() {
    nannou::app(model).event(event).simple_window(view).size(WIDTH, WIDTH).run();
}

fn model(_app: &App, width: u32) -> Model {
    // TODO: Somehow make this take the txn hash as input.
    // TODO: You can't provide args when building model so we read from an env var.
    let mut rng = SmallRng::seed_from_u64(10);
    let mountains = vec![
        Mountain::new(&mut rng, WIDTH, 50., 300.),
    ];
    Model { mountains }
}

fn event(_app: &App, _model: &mut Model, _event: Event) {}

fn view(app: &App, model: &Model, frame: Frame) {
    // get canvas to draw on
    let draw = app.draw();

    // set background to blue
    draw.background().color(BLUE);

    // Draw mountains
    for mountain in model.mountains() {
        mountain.draw(&draw);
    }

    // put everything on the frame
    draw.to_frame(app, &frame).unwrap();
}

struct Mountain {
    points: Vec<f32>,
}

impl Mountain {
    pub fn new(rng: &mut SmallRng, width: u32, min_height: f64, max_height: f64) -> Mountain {
        let step_max = rng.gen_range(0.9..1.1);
        let step_change = rng.gen_range(0.15..0.35);
        let mut height = rng.gen_range(0.0..max_height);
        let mut slope = rng.gen_range(0.0..step_max) * 2.0 - step_max;
        let mut points: Vec<f32> = Vec::new();

        for _ in 0..(width * 2) {
            height = height + slope;
            slope = slope + (rng.gen_range(0.0..step_change) * 2.0 - step_change);

            if slope > step_max {
                slope = step_max;
            } else if slope < -step_max {
                slope = -step_max;
            }

            if height > max_height {
                height = max_height;
                slope = slope * -1.0;
            } else if height < min_height {
                height = min_height;
                slope = slope * -1.0;
            }
            points.push(height as f32);
        }
        Mountain { points }
    }

    pub fn draw(&self, draw: &Draw) {
        // todo how to express x as a percent of container width?
        let points = self.points.iter().enumerate().map(|(i, y)| {
            let x = (i as i32 - WIDTH as i32) as f32;
            (pt2(x, *y), RED)
        });
        draw.polygon().points_colored(points);
    }
}

/*
use rand::{Rng, ThreadRng};
use std::{fs::File, path::Path};
use image::{ImageBuffer, Rgb, ImageRgb8, PNG, RgbImage};
use imageproc::drawing::draw_filled_circle_mut;
use imageproc::pixelops::interpolate;

    /*
    fn draw(&self, img: &mut RgbImage, color: Rgb<u8>, c_fog: Rgb<u8>) {
        let mut i = 0;
        for &point in self.points.iter() {
            img.put_pixel(i, point, color);
            for j in point..480 {
                img.put_pixel(i, j, interpolate(c_fog, color, j as f32 / 480.0));
            }
            i = i + 1;
        }
    }
    */
}

fn rgb_rand(rng: &mut ThreadRng, r: (u8, u8), g: (u8, u8), b: (u8, u8)) -> Rgb<u8> {
    Rgb([rng.gen_range(r.0, r.1), rng.gen_range(g.0, g.1), rng.gen_range(b.0, b.1)])
}

fn main() {
    let mut rng = rand::thread_rng();
    let c_sky = match rng.gen_range(1, 4) {
        1 => rgb_rand(&mut rng, (1, 40), (1, 40), (1, 40)),
        2 => rgb_rand(&mut rng, (215, 225), (215, 225), (230, 255)),
        _ => rgb_rand(&mut rng, (200, 255), (200, 255), (200, 255)),
    };
    let c_fog = rgb_rand(&mut rng, (1, 255), (1, 255), (1, 255));
    let mut img = ImageBuffer::from_pixel(640, 480, c_sky);

    if rng.gen_weighted_bool(2) {
        let x = rng.gen_range(101, 520);
        let y = rng.gen_range(81, 200);
        let rad = rng.gen_range(20, 80);
        let c_planet = interpolate(rgb_rand(&mut rng, (1, 255), (1, 255), (1, 255)), c_sky, 0.1);
        draw_filled_circle_mut(&mut img, (x, y), rad, c_planet);
        if !rng.gen_weighted_bool(5) {
            draw_filled_circle_mut(&mut img, (x + rng.gen_range(-2, 4) * 10, y), rad, c_sky);
        }
    }

    for (_, y, pixel) in img.enumerate_pixels_mut() {
        *pixel = interpolate(c_fog, *pixel, y as f32 / 1000.0);
    }

    let mountain_count: u32 = rng.gen_range(4, 7);
    let c_mountain = rgb_rand(&mut rng, (1, 255), (1, 255), (1, 255));
    for i in 0..mountain_count {
        let c = interpolate(c_mountain, c_sky, (i + 1) as f32 / mountain_count as f32);
        let y_amp = ( (399 - 480 / 2 / mountain_count * (mountain_count - i)) as f64, 401.0 );
        Mountain::new(y_amp).draw(&mut img, c, c_fog);
    }

    let _ = ImageRgb8(img).save(&mut File::create(&Path::new("images/export.png")).unwrap(), PNG);
}


fn blah() {
    println!("Hello, world!");
}
*/
