use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::rect::{Rect, Point};
use std::time::{Duration, Instant};
use rand::Rng;

// -- SIMULATION CONSTANTS --
const WINDOW_WIDTH: u32 = 800;
const WINDOW_HEIGHT: u32 = 800;
const ROAD_WIDTH: u32 = 150;
const LANE_WIDTH: u32 = ROAD_WIDTH / 2; // 75 pixels per lane

const VEHICLE_WIDTH: u32 = 25;
const VEHICLE_HEIGHT: u32 = 25;
const VEHICLE_SPEED: f32 = 2.5;
const SAFETY_GAP: f32 = 20.0; // Minimum distance between vehicles

// -- ENUMS AND STRUCTS --

#[derive(Debug, Clone, Copy, PartialEq)]
enum Route { Straight, Left, Right }

#[derive(Debug, Clone, Copy, PartialEq)]
enum Origin { North, South, East, West }

struct Vehicle {
    rect: Rect,
    vx: f32,
    vy: f32,
    color: Color,
    origin: Origin,
    is_stopped: bool,
    is_outbound: bool,
}

#[derive(PartialEq, Clone, Copy)]
enum LightState { Red, Green }

enum Phase { NorthSouth, EastWest }

struct TrafficLight {
    rect: Rect,
    state: LightState,
}

struct LightController {
    n_light: TrafficLight,
    e_light: TrafficLight,
    s_light: TrafficLight,
    w_light: TrafficLight,
    phase: Phase,
    phase_timer: Instant,
    base_duration: Duration,
}

impl LightController {
    fn new() -> Self {
        LightController {
            n_light: TrafficLight {
                rect: Rect::new((WINDOW_WIDTH / 2 - ROAD_WIDTH / 2 - 15) as i32, (WINDOW_HEIGHT / 2 + ROAD_WIDTH / 2) as i32, 15, 15),
                state: LightState::Green,
            },
            e_light: TrafficLight {
                rect: Rect::new((WINDOW_WIDTH / 2 + ROAD_WIDTH / 2) as i32, (WINDOW_HEIGHT / 2 + ROAD_WIDTH / 2) as i32, 15, 15),
                state: LightState::Red,
            },
            s_light: TrafficLight {
                rect: Rect::new((WINDOW_WIDTH / 2 - ROAD_WIDTH / 2 - 15) as i32, (WINDOW_HEIGHT / 2 - ROAD_WIDTH / 2 - 15) as i32, 15, 15),
                state: LightState::Green,
            },
            w_light: TrafficLight {
                rect: Rect::new((WINDOW_WIDTH / 2 + ROAD_WIDTH / 2) as i32, (WINDOW_HEIGHT / 2 - ROAD_WIDTH / 2 - 15) as i32, 15, 15),
                state: LightState::Red,
            },
            phase: Phase::NorthSouth,
            phase_timer: Instant::now(),
            base_duration: Duration::from_secs(8),
        }
    }

    fn update(&mut self, vehicles: &Vec<Vehicle>) {
        let mut extend_green = false;
        let lane_capacity = (LANE_WIDTH as f32 / (VEHICLE_HEIGHT as f32 + SAFETY_GAP)).floor() as usize;

        if self.phase_timer.elapsed() >= self.base_duration {
            let (current_green_origins, _current_red_origins) = match self.phase {
                Phase::NorthSouth => ([Origin::North, Origin::South], [Origin::East, Origin::West]),
                Phase::EastWest => ([Origin::East, Origin::West], [Origin::North, Origin::South]),
            };

            let mut congested_lanes = 0;
            for &origin in &current_green_origins {
                let queue_size = vehicles.iter().filter(|v| v.origin == origin && v.is_stopped).count();
                if queue_size > lane_capacity / 2 {
                    congested_lanes += 1;
                }
            }

            if congested_lanes > 0 {
                extend_green = true;
            }
        }

        if self.phase_timer.elapsed() >= self.base_duration && !extend_green {
            self.phase_timer = Instant::now();
            match self.phase {
                Phase::NorthSouth => {
                    self.phase = Phase::EastWest;
                    self.n_light.state = LightState::Red;
                    self.s_light.state = LightState::Red;
                    self.e_light.state = LightState::Green;
                    self.w_light.state = LightState::Green;
                }
                Phase::EastWest => {
                    self.phase = Phase::NorthSouth;
                    self.e_light.state = LightState::Red;
                    self.w_light.state = LightState::Red;
                    self.n_light.state = LightState::Green;
                    self.s_light.state = LightState::Green;
                }
            }
        }
    }

    fn get_light_state_for(&self, origin: Origin) -> LightState {
        match origin {
            Origin::North => self.n_light.state,
            Origin::South => self.s_light.state,
            Origin::East => self.e_light.state,
            Origin::West => self.w_light.state,
        }
    }
}

// -- MAIN FUNCTION --

pub fn main() -> Result<(), String> {
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;

    let window = video_subsystem.window("Traffic Simulation", WINDOW_WIDTH, WINDOW_HEIGHT)
        .position_centered()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window.into_canvas().present_vsync().build().map_err(|e| e.to_string())?;
    let mut event_pump = sdl_context.event_pump()?;
    let mut rng = rand::thread_rng();

    let mut vehicles: Vec<Vehicle> = Vec::new();
    let mut light_controller = LightController::new();

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => break 'running,
                Event::KeyDown { keycode: Some(keycode), .. } => {
                    let origin = match keycode {
                        Keycode::Up => Some(Origin::South),
                        Keycode::Down => Some(Origin::North),
                        Keycode::Left => Some(Origin::East),
                        Keycode::Right => Some(Origin::West),
                        Keycode::R => {
                            match rng.gen_range(0..4) {
                                0 => Some(Origin::North),
                                1 => Some(Origin::South),
                                2 => Some(Origin::East),
                                _ => Some(Origin::West),
                            }
                        }
                        _ => None
                    };

                    if let Some(o) = origin {
                        let can_spawn = !vehicles.iter().any(|v| {
                            if v.origin != o { return false; }
                            let dist = match o {
                                Origin::North => v.rect.y(),
                                Origin::South => (WINDOW_HEIGHT as i32) - v.rect.bottom(),
                                Origin::East => v.rect.x(),
                                Origin::West => (WINDOW_WIDTH as i32) - v.rect.right(),
                            };
                            dist < (VEHICLE_HEIGHT as i32 + SAFETY_GAP as i32)
                        });

                        if can_spawn {
                            vehicles.push(spawn_vehicle(o));
                        }
                    }
                }
                _ => {}
            }
        }

        // -- UPDATE STATE --
        light_controller.update(&vehicles);

        // Define intersection center with explicit casts
        let intersection_center_x: i32 = (WINDOW_WIDTH / 2) as i32; // 400
        let intersection_center_y: i32 = (WINDOW_HEIGHT / 2) as i32; // 400

        for i in 0..vehicles.len() {
            let mut is_stopped_by_car = false;
            let current_vehicle_rect = vehicles[i].rect;
            let current_vehicle_origin = vehicles[i].origin;
            let current_vehicle_is_outbound = vehicles[i].is_outbound;

            // Collect other vehicles' data before mutable borrow
            let other_vehicles: Vec<(usize, Rect, Origin, bool)> = vehicles
                .iter()
                .enumerate()
                .filter(|(j, _)| *j != i)
                .map(|(j, v)| (j, v.rect, v.origin, v.is_outbound))
                .collect();

            // Split vehicles to avoid borrow conflicts
            let (left, right) = vehicles.split_at_mut(i);
            let vehicle = &mut right[0]; // vehicles[i]

            // Check if vehicle has passed the intersection
            if !vehicle.is_outbound {
                match vehicle.origin {
                    Origin::North => {
                        if vehicle.rect.y() > intersection_center_y {
                            vehicle.is_outbound = true;
                            vehicle.rect.set_x((intersection_center_x + 10) as i32); // Move to right lane (x > 400)
                        }
                    }
                    Origin::South => {
                        if vehicle.rect.bottom() < intersection_center_y {
                            vehicle.is_outbound = true;
                            vehicle.rect.set_x((intersection_center_x + 10) as i32); // Move to right lane (x > 400)
                        }
                    }
                    Origin::East => {
                        if vehicle.rect.right() > intersection_center_x {
                            vehicle.is_outbound = true;
                            vehicle.rect.set_y((intersection_center_y + 10) as i32); // Move to right lane (y > 400)
                        }
                    }
                    Origin::West => {
                        if vehicle.rect.x() < intersection_center_x {
                            vehicle.is_outbound = true;
                            vehicle.rect.set_y((intersection_center_y - 25 - 10) as i32); // Move to right lane (y < 400)
                        }
                    }
                }
            }

            // Vehicle-Vehicle collision avoidance
            for (j, other_rect, other_origin, other_is_outbound) in other_vehicles {
                if other_origin == current_vehicle_origin && other_is_outbound == current_vehicle_is_outbound {
                    let dist = match current_vehicle_origin {
                        Origin::North => other_rect.y() - current_vehicle_rect.y(),
                        Origin::South => current_vehicle_rect.y() - other_rect.y(),
                        Origin::East => other_rect.x() - current_vehicle_rect.x(),
                        Origin::West => current_vehicle_rect.x() - other_rect.x(),
                    };
                    if dist > 0 && (dist as f32) < (VEHICLE_HEIGHT as f32 + SAFETY_GAP) {
                        is_stopped_by_car = true;
                        break;
                    }
                }
            }

            // Vehicle-Light interaction
            let light_state = light_controller.get_light_state_for(current_vehicle_origin);
            let stop_line: i32 = (WINDOW_HEIGHT / 2 - ROAD_WIDTH / 2) as i32;
            let is_at_red_light = match current_vehicle_origin {
                Origin::South => light_state == LightState::Red && current_vehicle_rect.bottom() > stop_line && current_vehicle_rect.y() < stop_line + ROAD_WIDTH as i32,
                Origin::North => light_state == LightState::Red && current_vehicle_rect.y() < stop_line && current_vehicle_rect.bottom() > stop_line - ROAD_WIDTH as i32,
                Origin::West => light_state == LightState::Red && current_vehicle_rect.right() > stop_line && current_vehicle_rect.x() < stop_line + ROAD_WIDTH as i32,
                Origin::East => light_state == LightState::Red && current_vehicle_rect.x() < stop_line && current_vehicle_rect.right() > stop_line - ROAD_WIDTH as i32,
            };

            if is_stopped_by_car || is_at_red_light {
                vehicle.is_stopped = true;
            } else {
                vehicle.is_stopped = false;
                vehicle.rect.set_x(vehicle.rect.x() + vehicle.vx as i32);
                vehicle.rect.set_y(vehicle.rect.y() + vehicle.vy as i32);
            }
        }

        vehicles.retain(|v| {
            v.rect.right() > 0 && v.rect.x() < WINDOW_WIDTH as i32 &&
            v.rect.bottom() > 0 && v.rect.y() < WINDOW_HEIGHT as i32
        });

        // -- RENDER --
        canvas.set_draw_color(Color::RGB(34, 139, 34)); // Green grass
        canvas.clear();

        let h_road = Rect::new(0, (WINDOW_HEIGHT / 2 - ROAD_WIDTH / 2) as i32, WINDOW_WIDTH, ROAD_WIDTH);
        let v_road = Rect::new((WINDOW_WIDTH / 2 - ROAD_WIDTH / 2) as i32, 0, ROAD_WIDTH, WINDOW_HEIGHT);
        canvas.set_draw_color(Color::RGB(105, 105, 105)); // Grey road
        canvas.fill_rects(&[h_road, v_road])?;

        // Draw dashed center lines
        canvas.set_draw_color(Color::RGB(255, 255, 255));
        let dash_length = 10;
        let gap_length = 10;
        let mut h_points = Vec::new();
        let mut v_points = Vec::new();

        let mut x = 0;
        while x < WINDOW_WIDTH as i32 {
            h_points.push(Point::new(x, (WINDOW_HEIGHT / 2) as i32));
            h_points.push(Point::new((x + dash_length).min(WINDOW_WIDTH as i32), (WINDOW_HEIGHT / 2) as i32));
            x += dash_length + gap_length;
        }

        let mut y = 0;
        while y < WINDOW_HEIGHT as i32 {
            v_points.push(Point::new((WINDOW_WIDTH / 2) as i32, y));
            v_points.push(Point::new((WINDOW_WIDTH / 2) as i32, (y + dash_length).min(WINDOW_HEIGHT as i32)));
            y += dash_length + gap_length;
        }

        canvas.draw_lines(h_points.as_slice())?;
        canvas.draw_lines(v_points.as_slice())?;

        for vehicle in &vehicles {
            canvas.set_draw_color(vehicle.color);
            canvas.fill_rect(vehicle.rect)?;
        }

        let ns_color = if light_controller.n_light.state == LightState::Green { Color::GREEN } else { Color::RED };
        let ew_color = if light_controller.e_light.state == LightState::Green { Color::GREEN } else { Color::RED };
        canvas.set_draw_color(ns_color);
        canvas.fill_rect(light_controller.n_light.rect)?;
        canvas.fill_rect(light_controller.s_light.rect)?;
        canvas.set_draw_color(ew_color);
        canvas.fill_rect(light_controller.e_light.rect)?;
        canvas.fill_rect(light_controller.w_light.rect)?;

        canvas.present();
    }

    Ok(())
}

// -- HELPER FUNCTIONS --

fn spawn_vehicle(origin: Origin) -> Vehicle {
    let (x, y, vx, vy) = match origin {
        Origin::North => ((WINDOW_WIDTH / 2 - VEHICLE_WIDTH - 10) as i32, 0, 0.0, VEHICLE_SPEED), // Left lane (x < 400)
        Origin::South => ((WINDOW_WIDTH / 2 - VEHICLE_WIDTH - 10) as i32, (WINDOW_HEIGHT - VEHICLE_HEIGHT) as i32, 0.0, -VEHICLE_SPEED), // Left lane (x < 400)
        Origin::East => (0, (WINDOW_HEIGHT / 2 - VEHICLE_HEIGHT - 10) as i32, VEHICLE_SPEED, 0.0), // Left lane (y < 400)
        Origin::West => ((WINDOW_WIDTH - VEHICLE_WIDTH) as i32, (WINDOW_HEIGHT / 2 + 10) as i32, -VEHICLE_SPEED, 0.0), // Left lane (y > 400)
    };

    let _route = match rand::thread_rng().gen_range(0..3) {
        0 => Route::Straight,
        1 => Route::Left,
        _ => Route::Right,
    };

    let color = match origin {
        Origin::North => Color::RGB(255, 0, 0),    // Red
        Origin::South => Color::RGB(0, 255, 0),    // Green
        Origin::East => Color::RGB(0, 0, 255),     // Blue
        Origin::West => Color::RGB(255, 255, 0),   // Yellow
    };

    Vehicle {
        rect: Rect::new(x, y, VEHICLE_WIDTH, VEHICLE_HEIGHT),
        vx, vy, color, origin, is_stopped: false, is_outbound: false,
    }
}