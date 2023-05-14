//TODO: remember to remove this later
#![allow(unused)]

mod terrain;

use log::{debug, error, info, log_enabled, trace, warn, Level};
use std::collections::HashSet;
use std::sync::mpsc::{Receiver, Sender};

use bracket_lib::pathfinding::BaseMap;

use bracket_lib::prelude::Algorithm2D;
use bracket_lib::terminal::{Point, BLACK};
use bracket_lib::{
    noise::{FastNoise, FractalType, NoiseType},
    random::RandomNumberGenerator,
};

use crate::helpers::{index_to_point, point_to_index, Distance};
use crate::render::{Frame, InputPacket, RenderPacket, Tile};

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
    /// whether to have a random number of poles, 1, or 2
    pub poles: Poles,
    /// what percentage of the world should be water 1-100 with a unknown percent margin
    // TODO: consider allowing setting for % water amt level variance
    pub target_water: u8,
    /// the maximum amount of ports to generate. 0-255
    pub max_ports: u8,
    /// the maximum amount of civilizations to generate. 0-255 accepted, stick to 0-30
    pub max_civilizations: u8,
    /// the total size of the world
    pub world_size: RectDimension,
}

// todo: make this more interesting later
pub struct FullWorld {
    terrain: TerrainMap,
    rivers: Vec<River>,
    // ...
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

                fg = bracket_lib::color::RGB::from_f32(
                    0.,
                    0.,
                    (1.0 - Distance::distance(world.terrain.sea_level, h)
                        / Distance::distance(world.terrain.min_height, world.terrain.sea_level))
                        as f32,
                );
            } else {
                // TODO: figure out difference between mountain & land
                let height = 1.0
                    - Distance::distance(world.terrain.sea_level, h)
                        / Distance::distance(world.terrain.max_height, world.terrain.sea_level);

                rc = MOUNTAIN_CHAR;

                fg = bracket_lib::color::RGB::from_f32(
                    (1. * height).clamp(0.0001, f64::INFINITY) as f32,
                    0.75 * height as f32,
                    0.,
                );
            }

            Tile::new(rc, fg, bg)

            //TODO: implement rivers
        })
        .collect();

    // send tilemap

    sender.send(RenderPacket::NewFrame(Frame {
        dimensions: world.terrain.dimensions,
        to_render: tiles,
    }));

    // todo!()
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
    mut channels: Option<(Sender<RenderPacket>, Receiver<InputPacket>)>,
) -> FullWorld {
    let mut rng = RandomNumberGenerator::seeded(params.seed);

    // generate the base terrain of the map
    let base_map = terrain::gen_base_map(params, rng);

    // render base map
    match &mut channels {
        Some((sender, _)) => render_world(&base_map, sender),
        None => {}
    }

    todo!();
}
