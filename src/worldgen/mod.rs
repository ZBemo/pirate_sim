//TODO: remember to remove this later
#![allow(unused)]

mod terrain;

use log::{debug, error, info, log_enabled, trace, warn, Level};
use std::collections::HashSet;
use std::fmt::format;
use std::sync::mpsc::{Receiver, Sender};
use std::time::Duration;

use bracket_lib::pathfinding::BaseMap;

use bracket_lib::prelude::Algorithm2D;
use bracket_lib::terminal::{Point, BLACK};
use bracket_lib::{
    noise::{FastNoise, FractalType, NoiseType},
    random::RandomNumberGenerator,
};

use crate::helpers::{index_to_point, point_to_index, Distance};
use crate::render::{self, Frame, RenderPacket, RenderTick, Tile, GUI};

use self::terrain::River;

use super::helpers::RectDimension;

pub use terrain::Map as TerrainMap;

/// a Map + Civilizations, history, etc that is actually playable by a player
pub struct Region {}

pub enum Poles {
    Random,
    One,
    Two,
}
/// The parameters for generating a world
pub struct GenParam {
    /// The seed to use during world generation
    pub seed: u64,
    /// one pole all around map
    // pub poles: Poles,
    /// what percentage of the world should be water 1-100 with a unknown percent margin
    // TODO: consider allowing setting for % water amt level variance
    pub target_water: u8,
    /// the maximum amount of ports to generate. 0-255
    pub max_ports: u8,
    /// the maximum amount of civilizations to generate. 0-255 accepted, stick to 0-30
    pub max_civilizations: u8,
    /// the total size of the world
    pub world_size: RectDimension,
    // the range of possible polar tiles
    pub max_polar_tiles: u32,
    pub min_polar_tiles: u32,
}

struct Pole {
    frozen_tiles: Vec<(u8, u8)>,
}

impl Pole {
    pub fn new() -> Self {
        Pole {
            frozen_tiles: Vec::new(),
        }
    }
}

// todo: make this more interesting later
pub struct FullWorld {
    terrain: TerrainMap,
    rivers: Vec<River>,
    pole: Pole, // ...
}

impl FullWorld {
    // mainly getters for various sub-elements
    pub fn dimensions(&self) -> RectDimension {
        self.terrain.dimensions
    }
    pub fn num_tiles(&self) -> usize {
        self.terrain.height_map.len()
    }
    pub fn max_height(&self) -> f64 {
        self.terrain.max_height
    }
    pub fn min_height(&self) -> f64 {
        self.terrain.min_height
    }
}

fn render_world(world: &FullWorld, sender: &mut Sender<RenderPacket>) -> () {
    const MOUNTAIN_CHAR: char = '^';
    const LAND_CHAR: char = '-';
    const SEA_CHAR: char = '~';

    let is_sea = |h: f64| world.terrain.sea_level >= h;

    let tiles: Vec<_> = Iterator::zip(world.terrain.height_map.iter(), 0..)
        .map(|(&h, p)| {
            let (x, y) = world.terrain.dimensions.index_to_point(p);

            let mut rc = ' ';
            let bg = BLACK;
            let fg;

            if is_sea(h) {
                rc = SEA_CHAR;

                fg = bracket_lib::color::RGBA::from_f32(
                    0.,
                    0.,
                    (1.0 - Distance::distance(world.terrain.sea_level, h)
                        / Distance::distance(world.terrain.min_height, world.terrain.sea_level))
                        as f32,
                    1.,
                );
            } else {
                // TODO: figure out difference between mountain & land
                let height = 0.5
                    + Distance::distance(world.terrain.sea_level, h)
                        / Distance::distance(world.terrain.max_height, world.terrain.sea_level);

                rc = MOUNTAIN_CHAR;

                fg = bracket_lib::color::RGBA::from_f32(1., 0.75, 0., height as f32);
            }

            Tile::new(rc, fg, bg)

            //TODO: implement rivers
        })
        .collect();

    // send tilemap

    match sender.send(RenderPacket::NewFrame(Frame {
        dimensions: world.terrain.dimensions,
        to_render: tiles,
    })) {
        Ok(_) => {}
        Err(e) => {
            error!("Unable to send message to render thread. Error: {}", e);
        }
    };

    // todo!()
}

/// the mutable context necessary for all world generation functions
pub struct GenContext<'a, 'b> {
    rng: &'a mut RandomNumberGenerator,
    params: &'b GenParam,
}

/// A function to generate a whole world, starting with terrain and geography and going all the way
/// to history and settlements
///
/// put Some(rx) for [render] in order to receive render-able worlds back during certain stages
///
/// this function coordinates the generation of worlds and the random number generation involved,
/// allowing us to make deterministic worlds more easily.
pub fn gen_full_world(
    params: GenParam,
    mut channels: Option<(Sender<RenderPacket>, Receiver<RenderTick>)>,
) -> FullWorld {
    let mut rng = RandomNumberGenerator::seeded(params.seed);
    let mut context = GenContext {
        rng: &mut rng,
        params: &params,
    };
    let title_id;

    // add title and seed

    match &mut channels {
        Some((sender, receiver)) => {
            // build title frame
            // build seed frame
            // register 2 frames
            // wait for response
            // continue?

            // set up gui

            // todo: update title as progresses
            let title_frame = render::string_to_frame("Generating world!".into());
            let seed_frame = render::string_to_frame(format!("Seed: {}", params.seed));

            let title_gui = GUI {
                offset: (0, 1),
                to_render: title_frame,
            };

            let seed_gui = GUI {
                offset: (0, -1),
                to_render: seed_frame,
            };

            let gui_vec = vec![Some(title_gui), Some(seed_gui)];

            sender.send(RenderPacket::RegisterGUIs(0, gui_vec));

            title_id = receiver.recv_timeout(Duration::from_secs(3)).unwrap();
        }
        None => {}
    }

    // generate the base terrain of the map
    let mut base_map = terrain::gen_base_map(&mut context);

    // fill in omni-present things like poles, etc here

    add_pole(&mut base_map, &mut context);

    // render base map
    match &mut channels {
        Some((sender, _)) => render_world(&base_map, sender),
        None => {}
    }

    let eroded_map = terrain::erode(&base_map, &mut rng);

    todo!();
}

fn add_pole(base_map: &mut FullWorld, context: &mut GenContext) -> () {
    let mut polar_tiles = HashSet::<(u8, u8)>::new();
    let mut rng = RandomNumberGenerator::seeded(context.rng.next_u64());

    // surround edge of map

    // top and bottom edge
    for x in 0..base_map.dimensions().width {
        let top_point = (x, 0);
        let bottom_point = (x, base_map.dimensions().height);

        polar_tiles.insert(top_point);
        polar_tiles.insert(bottom_point);
    }
    for y in 0..base_map.dimensions().height {
        let top_point = (0, y);
        let bottom_point = (base_map.dimensions().width, y);

        polar_tiles.insert(top_point);
        polar_tiles.insert(bottom_point);
    }

    // start on inner edge, roll for points
    // to calculate do something like
    // (#Surrounding polars) * mp + (closeness to edge) * mp - (closeness to max polar tiles) * mp > roll

    let mut x_border = 1;
    let mut y_border = 1;
    while x_border < base_map.dimensions().width / 2 && y_border < base_map.dimensions().height / 2
    {
        // do stuff with border here

        if x_border < base_map.dimensions().width / 2 {
            x_border += 1;
        }
        if y_border < base_map.dimensions().height / 2 {
            y_border += 1;
        }
    }
}
