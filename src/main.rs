use minifb::{Key, ScaleMode, Window, WindowOptions};
use rand::Rng;

const WIDTH: usize = 1280;
const HEIGHT: usize = 720;

fn main() {
    let mut rng = rand::thread_rng();

    let mut buffer = vec![0u32; WIDTH * HEIGHT];

    // init environment
    let mut env = vec![0f32; WIDTH * HEIGHT];
    let mut agents: Vec<Agent> = Vec::new();
    for x in 0..WIDTH {
        for y in 0..HEIGHT {
            if x % 15 == 0 && y % 15 == 0 {
                agents.push(Agent {
                    x: x as f32,
                    y: y as f32,
                    rotation: ((x % 360) as f32).to_radians(),
                    sensor_angle: 45f32.to_radians(),
                    rotation_angle: 45f32.to_radians(),
                    sensor_offset_distance: 9.,
                    sensor_width: 1.,
                    step_size: 1.,
                    deposition_size: 5.,
                    random_dir_change_prob: 0.,
                    sensitivity_thresh: 0.,
                });
            }
        }
    }

    // init window
    let mut window = Window::new(
        "Noise Test - Press ESC to exit",
        WIDTH,
        HEIGHT,
        WindowOptions {
            resize: true,
            scale_mode: ScaleMode::UpperLeft,
            ..WindowOptions::default()
        },
    )
    .expect("Unable to create the window");

    while window.is_open() && !window.is_key_down(Key::Escape) {
        if let Some((x, y)) = window.get_mouse_pos(minifb::MouseMode::Clamp) {
            if window.get_mouse_down(minifb::MouseButton::Left) {
                env[two_d_one_d(x as usize, y as usize)] = 4.;
            }
        }

        // agent move
        for agent in agents.iter_mut() {
            let (n_x, n_y) = (
                agent.x + agent.rotation.sin() * agent.step_size,
                agent.y + agent.rotation.cos() * agent.step_size,
            );

            if n_x > 0. && n_y > 0. && (n_x as usize) < WIDTH && (n_y as usize) < HEIGHT {
                agent.x = n_x;
                agent.y = n_y;
            } else {
                // should be random, later
                agent.rotation = (rng.gen_range(0..360) as f32).to_radians();
            }
        }

        // agent deposit
        for agent in agents.iter() {
            env[two_d_one_d(agent.x as usize, agent.y as usize)] = agent.deposition_size;
        }

        // Diffuse
        env = box_blur(env, 3);
        // Decay
        for value in env.iter_mut() {
            *value *= 0.99;
        }

        // env to buffer
        for (index, value) in env.iter().enumerate() {
            buffer[index] = rgb_to_color(
                (value * 255.) as u32,
                (0. * 255.) as u32,
                (0. * 255.) as u32,
            );
        }

        // agent pos to buffer
        for agent in agents.iter() {
            buffer[two_d_one_d(agent.x as usize, agent.y as usize)] = rgb_to_color(
                (agent.rotation * 50.) as u32,
                (agent.rotation * 50.) as u32,
                255,
            );
        }

        // update window
        window.update_with_buffer(&buffer, WIDTH, HEIGHT).unwrap();
    }
}

/// r x r filter cernel
/// r should always be uneven
/// very slow -> look into fast blur algorithms
fn box_blur(data: Vec<f32>, r: u32) -> Vec<f32> {
    if r % 2 == 0 {
        panic!("AHHH");
    }

    let r_range = (r / 2) as i32;
    let r_square = r * r;
    let mut tmp: Vec<f32> = Vec::new();
    for y in 0..HEIGHT as i32 {
        for x in 0..WIDTH as i32 {
            let mut sum: f32 = 0.;
            for x_r in -r_range..=r_range {
                for y_r in -r_range..=r_range {
                    sum += data[two_d_one_d(
                        ((x as isize + x_r as isize) + WIDTH as isize) as usize % WIDTH,
                        ((y as isize + y_r as isize) + HEIGHT as isize) as usize % HEIGHT,
                    )];
                }
            }
            tmp.push(sum / (r_square as f32));
        }
    }

    tmp
}

fn one_d_two_d(i: usize) -> (usize, usize) {
    (i % WIDTH, i / WIDTH)
}

fn two_d_one_d(x: usize, y: usize) -> usize {
    x + WIDTH * y
}

fn color_to_rgb(color: u32) -> (u32, u32, u32) {
    (color >> 16 & 0xff, color >> 8 & 0xff, color & 0xff)
}

fn rgb_to_color(r: u32, g: u32, b: u32) -> u32 {
    r << 16 | g << 8 | b
}

struct Agent {
    x: f32,
    y: f32,
    rotation: f32,
    sensor_angle: f32,
    rotation_angle: f32,
    sensor_offset_distance: f32,
    sensor_width: f32,
    step_size: f32,
    deposition_size: f32,
    random_dir_change_prob: f32,
    sensitivity_thresh: f32,
}
