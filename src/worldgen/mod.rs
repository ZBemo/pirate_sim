//TODO: remember to remove this later
#![allow(unused)]

mod terrain;

use log::{debug, error, info, log_enabled, trace, warn, Level};
use std::collections::HashSet;
use std::sync::mpsc::Receiver;

use bracket_lib::pathfinding::BaseMap;

use bracket_lib::prelude::Algorithm2D;
use bracket_lib::terminal::Point;
use bracket_lib::{
    noise::{FastNoise, FractalType, NoiseType},
    random::RandomNumberGenerator,
};

use crate::helpers::{index_to_point, point_to_index, Distance};

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

/// a struct containing the information necessary to render a world
pub struct RenderWorld {}

// todo: make this more interesting later
pub struct FullWorld {}

/// A function to generate a whole world, starting with terrain and geography and going all the way
/// to history and settlements
///
/// put Some(rx) for [render] in order to receive render-able worlds back during certain stages
///
/// this function coordinates the generation of worlds and the random number generation involved,
/// allowing us to make deterministic worlds more easily.
pub fn gen_full_world(params: GenParam, render: Option<Receiver<RenderWorld>>) -> FullWorld {
    let mut rng = RandomNumberGenerator::seeded(params.seed);

    let base_map = terrain::gen_base_map(params, rng);

    todo!();
}
