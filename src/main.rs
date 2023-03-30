extern crate sdl2;

use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::time::Duration;

use sdl2::rect::Rect;
use sdl2::render::WindowCanvas;
use sdl2::EventPump;
use sdl2::rect::Point;

const MOVEMENT_SPEED: f32 = 2.0;
const PI: f32 = 3.14159265259;
const ONE_DEGREE: f32 = PI/180.0;

const SCREEN_WIDTH: u32 = 512;
const SCREEN_HEIGHT: u32 = 512;
const MAP_SIZE: usize = 8;

#[derive(Debug)]
struct Entity {
    x: f32,
    y: f32,
    ix: i32,
    iy: i32,
    angle: f32,
    color: (u8,u8,u8),
    size: u32,
}
impl Entity {
    fn draw(&self, canvas: &mut WindowCanvas) {
        canvas.set_draw_color(Color::RGB(self.color.0,self.color.1,self.color.2));
        canvas.fill_rect(Rect::new(self.ix,self.iy,self.size,self.size)).expect("lol");
    }
    fn check_collide(&mut self) {
        // todo
    }
}

const MAP: [[i16; MAP_SIZE]; MAP_SIZE] = [[1,1,1,1,1,1,1,1],
                                          [1,0,0,0,0,0,0,1],
                                          [1,0,0,0,0,0,0,1],
                                          [1,0,0,0,0,0,0,1],
                                          [1,0,0,0,0,0,0,1],
                                          [1,0,0,0,0,0,0,1],
                                          [1,0,0,0,0,0,0,1],
                                          [1,1,1,1,1,1,1,1],];
// obviously this will break if `MAP_SIZE` is changed to something other than 8.

struct Controls {
    up: bool,
    down: bool,
    left: bool,
    right: bool,
}

pub fn main() {
    for r in MAP {
        for c in r {
            print!("{c}");
        }
        println!();
    }
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem.window("top-down", SCREEN_WIDTH, SCREEN_HEIGHT)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();

    canvas.set_draw_color(Color::RGB(0, 255, 255));
    canvas.clear();
    canvas.present();
    let mut event_pump = sdl_context.event_pump().unwrap();

    // ix is an integer rounding of float x
    let mut player = Entity {
        x: 128.0,
        y: 128.0,
        ix: 128,
        iy: 128,
        angle: 0.0,
        color: (255,0,0),
        size: 8,

    };

    let mut input = Controls {
        up: false,
        down: false,
        left: false,
        right: false,
    };

    'running: loop {
        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();

        // player.check_collide();

        if check_keys(&mut event_pump, &mut input) {
            break 'running
        }
        update_player_pos(&mut player, &input);

        draw_world(&mut canvas);
        player.draw(&mut canvas);
        cast_rays(&mut canvas, &player);


        canvas.present();
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
}

fn update_player_pos(player: &mut Entity, input: &Controls) {
    let mut changed = false;

    if input.left {
        player.angle += ONE_DEGREE;
        changed = true;
    }
    if input.right {
        player.angle -= ONE_DEGREE;
        changed = true;
    }

    if input.up {
        player.y += MOVEMENT_SPEED * (player.angle + PI/2.0).cos();
        player.x += MOVEMENT_SPEED * (player.angle + PI/2.0).sin();
        changed = true;
    }
    if input.down {
        player.y -= MOVEMENT_SPEED * (player.angle + PI/2.0).cos();
        player.x -= MOVEMENT_SPEED * (player.angle + PI/2.0).sin();
        changed = true;
    }


    if player.angle > 2.0*PI {
        player.angle = 0.0;
    }
    if player.angle < 0.0 {
        player.angle = 2.0*PI;
    }

    player.ix = player.x as i32;
    player.iy = player.y as i32;

    if changed {
        println!("{:?}", player);
    }

}
fn cast_rays(canvas: &mut WindowCanvas, player: &Entity) {
    for i in 0..1 {
        let start = Point::new(player.ix,player.iy);

        let ray_x: f32; let ray_y: f32; let ray_x_off: f32; let ray_y_off: f32;

        let inver_tan: f32 = -1.0/player.angle.tan();
        if (player.angle < PI) { // looking up
            // round the y value to nearest 64
            ray_y = ((player.iy >> 6) << 6) as f32 - 0.0001;
            ray_x = (player.y - ray_y) * inver_tan + player.x;

        }
        else if (player.angle > PI) { // looking down

        }
    }
}

fn draw_world(canvas: &mut WindowCanvas) {
    let square_width = SCREEN_WIDTH / MAP_SIZE as u32;
    let mut square_pos_x: i32 = 0;
    let mut square_pos_y: i32 = 0;
    for r in MAP {
        for c in r {
            // println!("{square_pos_x}, {square_pos_y}");
            if square_pos_x >= SCREEN_WIDTH as i32 {
                square_pos_x = 0;
            }
            match c {
                // drawing the rect a bit small to show the lines
                1 => {canvas.set_draw_color(Color::RGB(127,127,127));
                    canvas.fill_rect(Rect::new(square_pos_x+1,square_pos_y+1,square_width-1,square_width-1)).expect("lol");},
                0 => {canvas.set_draw_color(Color::RGB(63,63,63));
                    canvas.fill_rect(Rect::new(square_pos_x+1,square_pos_y+1,square_width-1,square_width-1)).expect("lol");},
                _ => {},
            }
            square_pos_x += square_width as i32;
        }
        square_pos_y += square_width as i32;
    }

}


fn check_keys(event_pump: &mut EventPump, input: &mut Controls) -> bool {
    for event in event_pump.poll_iter() {
        match event {
            Event::Quit {..} |
            Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                return true;
            },

            Event::KeyDown { keycode: Some(Keycode::Up), .. } => {
                input.up = true;
            },
            Event::KeyUp { keycode: Some(Keycode::Up), .. } => {
                input.up = false;
            },

            Event::KeyDown { keycode: Some(Keycode::Down), .. } => {
                input.down = true;
            },
            Event::KeyUp { keycode: Some(Keycode::Down), .. } => {
                input.down = false;
            },

            Event::KeyDown { keycode: Some(Keycode::Left), .. } => {
                input.left = true;
            },
            Event::KeyUp { keycode: Some(Keycode::Left), .. } => {
                input.left = false;
            },

            Event::KeyDown { keycode: Some(Keycode::Right), .. } => {
                input.right = true;
            },
            Event::KeyUp { keycode: Some(Keycode::Right), .. } => {
                input.right = false;
            },
            _ => {}
        }
    }
    return false;
}