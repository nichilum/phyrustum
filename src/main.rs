use fast_poisson::Poisson2D;
use image::{open, GenericImage, GenericImageView, ImageBuffer};
use minifb::{Key, KeyRepeat, ScaleMode, Window, WindowOptions};
use rand::Rng;
use std::{path::Path, thread, time};

use svg::node::element::path::Data;
use svg::Document;

const WIDTH: usize = 1024;
const HEIGHT: usize = 1024;

fn main() {
    let mut rng = rand::thread_rng();

    let mut current_frame: u32 = 0;

    let mut buffer = vec![0u32; WIDTH * HEIGHT];

    // init environment
    let mut env = vec![0f32; WIDTH * HEIGHT];
    // for (x, y, pixel) in open("assets/env_map.png")
    //     .unwrap()
    //     .into_luma8()
    //     .enumerate_pixels()
    // {
    //     env[two_d_one_d(x as usize, y as usize)] = *pixel.0.get(0).unwrap() as f32 / 50.;
    // }
    let (mut agents, mut collision_map) = setup_agents();

    // init window
    let mut window = Window::new(
        "Phyrustum",
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
        // Reset env
        if window.is_key_pressed(Key::R, KeyRepeat::No) {
            for value in env.iter_mut() {
                *value = 0.;
            }
        }
        // Draw
        if let Some((x, y)) = window.get_mouse_pos(minifb::MouseMode::Clamp) {
            if window.get_mouse_down(minifb::MouseButton::Left) {
                env[two_d_one_d(x as usize, y as usize)] = 100.;
            }
        }

        // agent motor stage
        for agent in agents.iter_mut() {
            let (n_x, n_y) = (
                agent.x + agent.rotation.sin() * agent.step_size,
                agent.y + agent.rotation.cos() * agent.step_size,
            );

            if n_x > 0. && n_y > 0. && (n_x as usize) < WIDTH && (n_y as usize) < HEIGHT {
                // if collision_map[n_x as usize][n_y as usize] == 2 {
                //     agent.deposition_size = 0.;
                // } else
                if collision_map[n_x as usize][n_y as usize] == 0 {
                    // doesn't work correctly -> random agent movements
                    collision_map[n_x as usize][n_y as usize] = 1;
                    collision_map[agent.x as usize][agent.y as usize] = 0;

                    agent.x = n_x;
                    agent.y = n_y;
                    // agent deposit
                    env[two_d_one_d(agent.x as usize, agent.y as usize)] = agent.deposition_size;
                } else {
                    agent.rotation = (rng.gen_range(0..360) as f32).to_radians();
                }
            }

            // add to svg path
            agent.pos_to_svg();
        }

        // agent sensory stage
        for agent in agents.iter_mut() {
            let (front_x, front_y) = (
                (agent.x + agent.rotation.sin() * agent.sensor_offset_distance)
                    .clamp(0., (WIDTH - 1) as f32),
                (agent.y + agent.rotation.cos() * agent.sensor_offset_distance)
                    .clamp(0., (HEIGHT - 1) as f32),
            );
            let (left_x, left_y) = (
                (agent.x
                    + (agent.rotation - agent.sensor_angle).sin() * agent.sensor_offset_distance)
                    .clamp(0., (WIDTH - 1) as f32),
                (agent.y
                    + (agent.rotation - agent.sensor_angle).cos() * agent.sensor_offset_distance)
                    .clamp(0., (HEIGHT - 1) as f32),
            );
            let (right_x, right_y) = (
                (agent.x
                    + (agent.rotation + agent.sensor_angle).sin() * agent.sensor_offset_distance)
                    .clamp(0., (WIDTH - 1) as f32),
                (agent.y
                    + (agent.rotation + agent.sensor_angle).cos() * agent.sensor_offset_distance)
                    .clamp(0., (HEIGHT - 1) as f32),
            );

            let trail_value_front: f32 = env[two_d_one_d(front_x as usize, front_y as usize)];
            let trail_value_left: f32 = env[two_d_one_d(left_x as usize, left_y as usize)];
            let trail_value_right: f32 = env[two_d_one_d(right_x as usize, right_y as usize)];

            if trail_value_front > trail_value_left && trail_value_front > trail_value_right {
            } else if trail_value_front < trail_value_left && trail_value_front < trail_value_right
            {
                agent.rotation += (rng.gen_range(0..=1) * 2 - 1) as f32 * agent.rotation_angle;
            } else if trail_value_left < trail_value_right {
                agent.rotation += agent.rotation_angle;
            } else if trail_value_right < trail_value_left {
                agent.rotation -= agent.rotation_angle;
            }
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
                (value * 255.) as u8,
                (value * 255.) as u8,
                (value * 255.) as u8,
            );
        }

        if window.is_key_down(Key::Space) {
            // agent pos to buffer
            for agent in agents.iter() {
                buffer[two_d_one_d(agent.x as usize, agent.y as usize)] = rgb_to_color(0, 0, 255);
            }

            // collision to buffer
            for x in 0..WIDTH {
                for y in 0..HEIGHT {
                    if collision_map[x][y] != 0 {
                        buffer[two_d_one_d(x, y)] = rgb_to_color(0, 255, 0);
                    }
                }
            }
        }

        // update window
        window.update_with_buffer(&buffer, WIDTH, HEIGHT).unwrap();

        // slow image export down if sth happens
        // thread::sleep(time::Duration::from_millis(100));
        // let mut luminance_buffer: Vec<u8> = Vec::new();
        // for value in env.iter() {
        //     luminance_buffer.push((value * 255.) as u8);
        // }
        // image::save_buffer(
        //     &Path::new(&format!("image-{}.jpg", current_frame)),
        //     luminance_buffer.as_slice(),
        //     WIDTH as u32,
        //     HEIGHT as u32,
        //     image::ColorType::L8,
        // );
        // current_frame += 1;

        if window.is_key_pressed(Key::S, KeyRepeat::No) {}
    }
}

/// (agents, collision_map)
fn setup_agents() -> (Vec<Agent>, Vec<Vec<u8>>) {
    let mut rng = rand::thread_rng();
    let mut agents: Vec<Agent> = Vec::new();
    let mut collision_map: Vec<Vec<u8>> = vec![vec![0; HEIGHT]; WIDTH];
    // let points = Poisson2D::new().with_dimensions([WIDTH as f64, HEIGHT as f64], 10.0);

    // for point in points.iter() {
    //     if point[0] > 15.
    //         && point[1] > 15.
    //         && point[0] < (WIDTH - 15) as f64
    //         && point[1] < (HEIGHT - 15) as f64
    //     {
    //         agents.push(Agent {
    //             x: point[0] as f32,
    //             y: point[1] as f32,
    //             rotation: (rng.gen_range(0..360) as f32).to_radians(),
    //             sensor_angle: (rng.gen_range(20..45) as f32).to_radians(),
    //             rotation_angle: (rng.gen_range(20..45) as f32).to_radians(),
    //             sensor_offset_distance: 9.,
    //             sensor_width: 1.,
    //             step_size: 1.,
    //             deposition_size: 1.,
    //             random_dir_change_prob: 0.,
    //             sensitivity_thresh: 0.,
    //         });
    //         collision_map[point[0] as usize][point[1] as usize] = 1;
    //     }
    // }

    for (x, y, pixel) in open("assets/Nichilum.png")
        .unwrap()
        .into_luma8()
        .enumerate_pixels()
    {
        if *pixel.0.get(0).unwrap() != 0 && x % 5 == 0 && y % 5 == 0 {
            agents.push(Agent {
                x: x as f32,
                y: y as f32,
                rotation: (rng.gen_range(0..360) as f32).to_radians(),
                sensor_angle: (rng.gen_range(20..45) as f32).to_radians(),
                rotation_angle: (rng.gen_range(20..45) as f32).to_radians(),
                sensor_offset_distance: 20., // 9
                sensor_width: 1.,
                step_size: 1.5,      // 1
                deposition_size: 1., // 5
                random_dir_change_prob: 0.,
                sensitivity_thresh: 0.,
                path_data: Data::new().move_to((x as f32, y as f32)),
            });
            collision_map[x as usize][y as usize] = 1;
        }
    }

    // init rect collisions
    for x in 0..WIDTH {
        for y in 0..HEIGHT {
            if x == 0 || y == 0 || x == WIDTH - 1 || y == HEIGHT - 1 {
                collision_map[x][y] = 2;
            }
        }
    }

    // init petri collisions
    let center = WIDTH / 2;
    let radius = 400 * 400;
    for x in 0..WIDTH {
        for y in 0..HEIGHT {
            if (center - x).pow(2) + (center - y).pow(2) >= radius {
                collision_map[x][y] = 2;
            }
        }
    }

    (agents, collision_map)
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

fn rgb_to_color(r: u8, g: u8, b: u8) -> u32 {
    (r as u32) << 16 | (g as u32) << 8 | b as u32
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
    path_data: svg::node::element::path::Data,
}

impl Agent {
    fn pos_to_svg(self) {
        self.path_data.line_to((self.x, self.y));
    }
}
