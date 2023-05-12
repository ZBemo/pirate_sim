//! TODO: SET UP LOGGER NOW
//! TODO: switch from f64 to f64 throughout the program

#![warn(clippy::pedantic, clippy::perf)]

mod helpers;
mod worldgen;

use bracket_lib::terminal::{BTerm, GameState, RGB};
use std::error::Error;

use log::{debug, trace, warn};

use helpers::Distance;
use helpers::RectArea;
use worldgen::gen_map;

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
    // generate a screen with dimensions large enough to display the map
    let ctx = bracket_lib::terminal::BTermBuilder::simple80x50()
        .with_fitscreen(true)
        .with_title("Generated Map")
        .build()?;

    let gs = RenderMap::new(hm, seed);

    bracket_lib::prelude::main_loop(ctx, gs)
}

fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    env_logger::init();
    warn!("HI");

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
    let gen = worldgen::GenParam {
        seed,
        poles: worldgen::Poles::Random,
        // ~70% is 178/255
        target_water: 178,
        max_ports: 100,
        max_civilizations: 18,

        world_size: RectArea::new(70, 40),
    };

    let map = gen_map(gen);

    render_map(map, seed as usize)
}
