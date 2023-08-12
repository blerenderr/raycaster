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
const PI: f32 = 3.14159265;
const HALF_PI: f32 = PI/2.0;
const TWO_PI: f32 = 2.0*PI;
const THREEHALFS_PI: f32 = 3.0*PI/2.0;
const ONE_DEGREE: f32 = PI/180.0;

const SCREEN_WIDTH: u32 = 512;
const SCREEN_HEIGHT: u32 = 512;
const MAP_SIZE: usize = 8;
const FOV: u32 = 60; // 60 works

const FP_SCREEN_WIDTH: u32 = 960;
const FP_SCREEN_HEIGHT: u32 = 720;

struct Ray {
    x: f32,
    y: f32,
    r: f32,
}

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
        canvas.fill_rect(Rect::new(self.ix-(self.size/2) as i32,self.iy-(self.size/2) as i32,self.size,
                                   self.size)).expect("lol");
    }
    // fn check_collide(&mut self) {
    //     // todo
    // }
}

const MAP: [[i16; MAP_SIZE]; MAP_SIZE] = [[1,1,1,1,1,1,1,1],
                                          [1,0,0,0,0,0,0,1],
                                          [1,0,0,0,0,0,0,1],
                                          [1,0,0,0,0,0,0,1],
                                          [1,0,1,0,0,1,0,1],
                                          [1,0,1,0,0,0,0,1],
                                          [1,0,1,0,0,0,0,1],
                                          [1,1,1,1,1,1,1,1],];
// obviously this will break if `MAP_SIZE` is changed to something other than 8.


struct Controls {
    forward: bool,
    backward: bool,
    sleft: bool,
    sright: bool,
    tleft: bool,
    tright: bool
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

    let window1 = video_subsystem.window("top-down", SCREEN_WIDTH, SCREEN_HEIGHT)
        .position_centered()
        .build()
        .unwrap();
    let window2 = video_subsystem.window("first-person",FP_SCREEN_WIDTH, FP_SCREEN_HEIGHT)
        .position(0, 0)
        .build()
        .unwrap();

    let mut canvas1 = window1.into_canvas().build().unwrap();
    let mut canvas2 = window2.into_canvas().build().unwrap();

    canvas1.set_draw_color(Color::RGB(0, 255, 255));
    canvas1.clear();
    canvas1.present();
    canvas2.set_draw_color(Color::RGB(0, 255, 255));
    canvas2.clear();
    canvas2.present();
    let mut event_pump = sdl_context.event_pump().unwrap();

    // ix is an integer rounding of float x
    let mut player = Entity {
        x: 256.0,
        y: 256.0,
        ix: 256,
        iy: 256,
        angle: HALF_PI,
        color: (255,0,0),
        size: 8,

    };

    let mut input = Controls {
        forward: false,
        backward: false,
        sleft: false,
        sright: false,
        tleft: false,
        tright: false
    };

    'running: loop {
        canvas1.set_draw_color(Color::RGB(0, 0, 0));
        canvas1.clear();
        canvas2.set_draw_color(Color::RGB(0, 0, 0));
        canvas2.clear();

        // player.check_collide();

        if check_keys(&mut event_pump, &mut input) {
            break 'running
        }
        update_player_pos(&mut player, &input);

        draw_world(&mut canvas1);
        cast_rays(&mut canvas1, &mut canvas2, &player);
        player.draw(&mut canvas1);


        canvas1.present();
        canvas2.present();

        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
}

fn update_player_pos(player: &mut Entity, input: &Controls) {
    let mut changed = false;

    if input.tleft {
        player.angle += ONE_DEGREE*1.5;
        changed = true;
    }
    if input.tright {
        player.angle -= ONE_DEGREE*1.5;
        changed = true;
    }

    if input.forward {
        player.y += MOVEMENT_SPEED * (player.angle + HALF_PI).cos();
        player.x += MOVEMENT_SPEED * (player.angle + HALF_PI).sin();
        changed = true;
    }
    if input.backward {
        player.y -= MOVEMENT_SPEED * (player.angle + HALF_PI).cos();
        player.x -= MOVEMENT_SPEED * (player.angle + HALF_PI).sin();
        changed = true;
    }

    if input.sleft {
        player.y += MOVEMENT_SPEED * (player.angle + HALF_PI + 90.0*ONE_DEGREE).cos();
        player.x += MOVEMENT_SPEED * (player.angle + HALF_PI + 90.0*ONE_DEGREE).sin();
        changed = true;
    }
    if input.sright {
        player.y += MOVEMENT_SPEED * (player.angle + HALF_PI - 90.0*ONE_DEGREE).cos();
        player.x += MOVEMENT_SPEED * (player.angle + HALF_PI - 90.0*ONE_DEGREE).sin();
        changed = true;
    }


    if player.angle > TWO_PI {player.angle = 0.0;}
    if player.angle < 0.0 {player.angle = TWO_PI;}

    player.ix = player.x as i32;
    player.iy = player.y as i32;

    if changed {println!("{:?}", player);}

}
fn cast_rays(canvas: &mut WindowCanvas, canvas2_ref: &mut WindowCanvas, player: &Entity) {
    
    for i in 0..FOV {
        let mut ray_x_off: f32 = 0.0; let mut ray_y_off: f32 = 0.0;
        let mut dof: u8 = 0;
        let start = Point::new(player.ix,player.iy);
        let mut angle = player.angle + (i as f32*ONE_DEGREE) - ((FOV/2) as f32*ONE_DEGREE);
        if angle > TWO_PI {angle -= TWO_PI;}
        if angle < 0.0 {angle += TWO_PI;}

        let inver_tan: f32 = 1.0/angle.tan();
        let mut horiz_ray = Ray {
            x: 0.0,
            y: 0.0,
            r: 0.0,
        };

        if angle < PI { // looking up
            // round the y value to nearest 64
            horiz_ray.y = ((player.iy >> 6) << 6) as f32 - 0.0001;
            horiz_ray.x = (player.y - horiz_ray.y) * inver_tan + player.x;
            ray_y_off = -64.0;
            ray_x_off = -ray_y_off*inver_tan;
        }
        if angle > PI { // looking down
            horiz_ray.y = ((player.iy >> 6) << 6) as f32 + 64.0;
            horiz_ray.x = (player.y - horiz_ray.y) * inver_tan + player.x;
            ray_y_off = 64.0;
            ray_x_off = -ray_y_off*inver_tan;
        }
        if (angle > -0.00 && angle < 0.00) || 
           (angle > PI+0.00 && angle < PI-0.00) {
            horiz_ray.x = player.x;
            horiz_ray.y = player.y;
            ray_x_off = 0.0;
            ray_y_off = 0.0;
            dof = 8;
        }

        while dof < 8 {
            let map_x: usize = horiz_ray.x as usize >> 6;
            let map_y: usize = horiz_ray.y as usize >> 6;
            if map_x <= 7 && map_y <= 7 && MAP[map_y][map_x] == 1 {
                dof = 8;
            } // wall hit
            else {
                horiz_ray.x += ray_x_off; horiz_ray.y += ray_y_off; dof += 1;
            }
        }
        horiz_ray.r = ((player.x - horiz_ray.x).powi(2) + (player.y - horiz_ray.y).powi(2)).sqrt();

        // vertical line check !
        let mut vert_ray = Ray {
            x: 0.0,
            y: 0.0,
            r: 0.0,
        };
        ray_x_off = 0.0; ray_y_off = 0.0; dof = 0;
        if angle > HALF_PI && angle < THREEHALFS_PI { // looking left
            // round x to the nearest 64
            vert_ray.x = ((player.ix >> 6) << 6) as f32 - 0.0001;
            vert_ray.y = (player.x - vert_ray.x) / inver_tan + player.y;
            ray_x_off = -64.0;
            ray_y_off = -ray_x_off/inver_tan;
        }
        if angle < HALF_PI || angle > THREEHALFS_PI { // looking right
            vert_ray.x = ((player.ix >> 6) << 6) as f32 + 64.0;
            vert_ray.y = (player.x - vert_ray.x) / inver_tan + player.y;
            ray_x_off = 64.0;
            ray_y_off = -ray_x_off/inver_tan;
        }
        if (angle > HALF_PI && angle < HALF_PI) || 
           (angle > THREEHALFS_PI && angle < THREEHALFS_PI) { // floating point weirdness
            vert_ray.x = player.x;
            vert_ray.y = player.y;
            ray_x_off = 0.0;
            ray_y_off = 0.0;
            dof = 8;
        }

        while dof < 8 {
            let map_x: usize = vert_ray.x as usize >> 6;
            let map_y: usize = vert_ray.y as usize >> 6;
            if map_x <= 7 && map_y <= 7 && MAP[map_y][map_x] == 1 {
                dof = 8;
            } // wall hit
            else {
                vert_ray.x += ray_x_off; vert_ray.y += ray_y_off; dof += 1;
            }
        }
        vert_ray.r = ((player.x - vert_ray.x).powi(2) + (player.y - vert_ray.y).powi(2)).sqrt();

        canvas.set_draw_color(Color::RGB(0,255,0));

        if horiz_ray.r < vert_ray.r {
            canvas.draw_line(start, Point::new(horiz_ray.x as i32,horiz_ray.y as i32)).expect("lol");
            project_line(canvas2_ref, &horiz_ray, i.try_into().unwrap(), &angle, player);
        }
        else {
            canvas.draw_line(start, Point::new(vert_ray.x as i32,vert_ray.y as i32)).expect("lol");
            project_line(canvas2_ref, &vert_ray, i.try_into().unwrap(), &angle, player);
        }

    }
}

// this shit needs to be fixed immediately.
fn project_line(canvas: &mut WindowCanvas, ray: &Ray, i: u16, angle: &f32, player: &Entity) {
    // distance (resultant) of the ray, not sure why cosine works
    let dist = ray.r * (player.angle - angle).cos();
    // length of the column on the screen, multiplyer shrinks the viewable space
    let length = (FP_SCREEN_HEIGHT as f32 * 0.9 - dist) as i32;
    // x position of the column
    let x: i32 = (FP_SCREEN_WIDTH - (i as u32 * FP_SCREEN_WIDTH/FOV)) as i32;
    // iterate through every column (width/fov) finding the color and drawing the line
    for i in 0..FP_SCREEN_WIDTH/FOV {
        let mut color_val = 255.0-(ray.r*0.6);
        if color_val < 0.0 {color_val = 0.0;}
        if color_val > 255.0 {color_val = 255.0;}


        canvas.set_draw_color(Color::RGB(color_val as u8,color_val as u8,255));
        let start = Point::new(x + i as i32,(720-length)/2);
        let end = Point::new(x + i as i32,(720-length)/2+length);
        canvas.draw_line(start, end).expect("lol");
    }

}

fn draw_world(canvas: &mut WindowCanvas) {
    let square_width = SCREEN_WIDTH / MAP_SIZE as u32;
    let mut square_pos_x: i32 = 0;
    let mut square_pos_y: i32 = 0;
    for r in MAP {
        for c in r {
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

            Event::KeyDown { keycode: Some(Keycode::W), .. } => {
                input.forward = true;
            },
            Event::KeyUp { keycode: Some(Keycode::W), .. } => {
                input.forward = false;
            },

            Event::KeyDown { keycode: Some(Keycode::S), .. } => {
                input.backward = true;
            },
            Event::KeyUp { keycode: Some(Keycode::S), .. } => {
                input.backward = false;
            },

            Event::KeyDown { keycode: Some(Keycode::A), .. } => {
                input.sleft = true;
            },
            Event::KeyUp { keycode: Some(Keycode::A), .. } => {
                input.sleft = false;
            },

            Event::KeyDown { keycode: Some(Keycode::D), .. } => {
                input.sright = true;
            },
            Event::KeyUp { keycode: Some(Keycode::D), .. } => {
                input.sright = false;
            },

            Event::KeyDown { keycode: Some(Keycode::Left), .. } => {
                input.tleft = true;
            },
            Event::KeyUp { keycode: Some(Keycode::Left), .. } => {
                input.tleft = false;
            },

            Event::KeyDown { keycode: Some(Keycode::Right), .. } => {
                input.tright = true;
            },
            Event::KeyUp { keycode: Some(Keycode::Right), .. } => {
                input.tright = false;
            },

            
            _ => {}
        }
    }
    return false;
}