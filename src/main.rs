//! TODO: SET UP LOGGER NOW
//! TODO: switch from f64 to f64 throughout the program

#![warn(clippy::pedantic, clippy::perf)]

mod helpers;
mod worldgen;

use bracket_lib::terminal::{BTerm, GameState, RGB};
use std::error::Error;
use std::sync::mpsc;
use std::thread;

use log::{debug, trace, warn};

use helpers::Distance;
use helpers::RectDimensions;
use worldgen::gen_base_map;

struct RenderMap {
    map: worldgen::Map,
    /// use this to decide whether to debug or not, as map as static there's no need to debug on every tick
    has_ticked: bool,
    seed: usize,
}

impl GameState for RenderMap {
    fn tick(&mut self, ctx: &mut BTerm) {
        if !self.has_ticked {
            ctx.cls();

            let map = &self.map;

            //TODO: make sure this is covering all of map.len()
            let mut i = 0;
            for x in 1..=map.size.width {
                for y in 1..=map.size.height {
                    let color;
                    let bg;
                    let char;
                    let h = map.height_map[i];
                    if h <= map.sea_level {
                        //TODO: calculate so that darkness is relative to distance of sea level and min
                        // for example 1.0 - h.abs() / distance(min,sea_level)

                        color = bracket_lib::color::RGB::from_f32(
                            0.,
                            0.,
                            (1.0 - Distance::distance(map.sea_level, h)
                                / Distance::distance(map.min_height, map.sea_level))
                                as f32,
                        );
                        bg = RGB::named(bracket_lib::color::BLACK);
                        char = '~';

                        // underwater
                    } else {
                        let height = 1.0
                            - Distance::distance(map.sea_level, h)
                                / Distance::distance(map.max_height, map.sea_level);

                        color = bracket_lib::color::RGB::from_f32(
                            (1. * height).clamp(0.0001, f64::INFINITY) as f32,
                            0.75 * height as f32,
                            0.,
                        );
                        bg = RGB::named(bracket_lib::color::BLACK);
                        char = '^';
                        // on land
                    }

                    ctx.set(x, y, color, bg, bracket_lib::prelude::to_cp437(char));

                    if x as u32 > ctx.get_char_size().0 {
                        warn!(
                            "x of {} printing out of screen\nmax possible x is {}",
                            x,
                            ctx.get_char_size().0
                        );
                    }
                    if y as u32 > ctx.get_char_size().1 {
                        warn!(
                            "y of {} printing out of screen\nmax possible y is {}",
                            y,
                            ctx.get_char_size().1
                        );
                    }

                    i += 1;
                }
            }

            if !self.has_ticked {
                debug!("drawn {} characters", i);
                assert_eq!(i, map.height_map.len());
                assert_eq!(i, map.size.height as usize * map.size.width as usize);
            }

            ctx.print(0, map.size.height + 1, format!("seed: {}", self.seed));
            self.has_ticked = true;
        }
    }
}

impl RenderMap {
    pub fn new(hm: worldgen::Map, seed: usize) -> Self {
        RenderMap {
            map: hm,
            has_ticked: false,
            seed,
        }
    }
}

//TODO: remove later
#[allow(unused)]
fn render_map(hm: worldgen::Map, seed: usize) -> Result<(), Box<dyn Error + Send + Sync>> {
    let wanted_x = hm.size.width;
    let wanted_y = hm.size.height + 2;

    // generate a screen with dimensions large enough to display the map
    let ctx = bracket_lib::terminal::BTermBuilder::simple(wanted_x, wanted_y)?
        .with_title("Generated Map")
        .build()?;

    let gs = RenderMap::new(hm, seed);

    bracket_lib::prelude::main_loop(ctx, gs)
}

fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    env_logger::init();

    let seed = std::env::var("PS_SEED").map_or_else(
        |_| {
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() as usize
        },
        |s| s.parse().unwrap(),
    ) as u64;

    trace!("Starting generating map with seed {}", seed);

    //TODO: make sure Xs and Ys align with width/height correctly throughout the program x = width, y = height
    //TODO: feature to regenerate with new seed graphically?
    let gen = worldgen::GenParam {
        seed,
        poles: worldgen::Poles::Random,
        // 178/255 around 70%
        target_water: 178,
        max_ports: 100,
        max_civilizations: 18,

        world_size: RectDimensions::new(100, 100),
    };

    let map = gen_base_map(gen);

    //TODO: RENDER THREAD !!!!!! RENDER THREAD !!!!!!

    // we want to gen map -> render it while other thread works on erosion, etc
    // when erosion done, close thread and render newly eroded map
    // continue in similar fashion for other stages of world gen
    // although this structure works well for worldgen, for actual gameplay it would make sense to
    // have a pair of channels, with one sending input data to a "working" thread, and another
    // sending render data to a "render" thread, allowing rendering and workign to be done nearly
    // independently of each other

    let (tx, rx) = mpsc::channel::<worldgen::Map>();
    let new_map = map.clone();

    thread::spawn(move || {
        // run erosion code and send new map back to rx
        todo!();
    });

    render_map(map, seed as usize)
}
