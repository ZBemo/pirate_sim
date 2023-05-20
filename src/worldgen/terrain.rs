use bracket_lib::{
    prelude::{Algorithm2D, BaseMap, FastNoise, FractalType, NoiseType},
    random::RandomNumberGenerator,
    terminal::Point,
};
use log::{debug, log_enabled, trace, warn, Level};

use crate::helpers::{Distance, RectDimension};

use super::{FullWorld, GenContext, GenParam, Pole};

/// a river or body of water
/// TODO: better format, use lines?
#[derive(Debug, Clone)]
pub struct River {
    // which tiles are covered by the water form
    covered_tiles: Vec<u32>,
}

/// a static map of a world and its terrain
#[derive(Debug, Clone)]
pub struct Map {
    pub dimensions: RectDimension,
    /// the level at which something goes into the sea/underwater
    pub sea_level: f64,
    pub min_height: f64,
    pub max_height: f64,
    pub height_map: Vec<f64>,
    pub rivers: Vec<River>,
}

/// Does a binary search of sea levels until amount of world covered by sea is withing a reasonable distance from `wanted_percent`
///
/// Should only error if a NaN is present in the input, or if the algorithm is bugged.
/// allows variance of `1./ height_map.len()` in order to ensure that the wanted percent is
/// actually achievable
fn decide_sea_level(height_map: &[f64], wanted_percent: f64) -> Result<f64, ()> {
    let mut min = height_map.iter().fold(f64::INFINITY, |a, &b| a.min(b));
    let mut max = height_map.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
    let lowest_percent = wanted_percent - 1. / height_map.len() as f64;
    let highest_percent = wanted_percent + 1. / height_map.len() as f64;

    let percent_uw = |sl: &f64| {
        height_map
            .iter()
            .fold(0., |t, h| if h <= sl { t + 1. } else { t })
            / (height_map.len() as f64)
    };

    loop {
        // mid != min
        #[allow(clippy::similar_names)]
        let mid = (min + max) * 0.5;

        let percent_uw = percent_uw(&mid);
        if (percent_uw > highest_percent) {
            trace!(
                "decreasing uw%: {}\nmid: {}\nmax: {}\nmin: {}\nwanted percent: {}-{}",
                percent_uw,
                mid,
                max,
                min,
                lowest_percent,
                highest_percent
            );
            max = mid;
        } else if (percent_uw < lowest_percent) {
            trace!(
                "increasing uw%: {}\nmid: {}\nmax: {}\nmin: {}\nwanted percent: {}-{}",
                percent_uw,
                mid,
                max,
                min,
                lowest_percent,
                highest_percent
            );
            min = mid;
        } else {
            return Ok(mid);
        }

        //TODO: yeah, this is broken idc because it seems like it still ends up with a close enough
        // percent anyways

        if (min > max) {
            warn!("minimum of {} greater than maximum of {}", min, max);
        }

        assert!(min < max, "min gt max");
    }

    Err(())
}

pub fn gen_base_map<'a, 'b>(context: &mut GenContext) -> FullWorld {
    let params = context.params;
    let mut rng = &mut context.rng;
    let (h, w) = (params.world_size.height, params.world_size.width);
    let mut noise = FastNoise::seeded(rng.next_u64());

    noise.set_noise_type(NoiseType::PerlinFractal);
    noise.set_fractal_type(FractalType::FBM);
    noise.set_fractal_octaves(10);
    noise.set_fractal_gain(0.5);
    noise.set_fractal_lacunarity(3.0);
    noise.set_frequency(2.0);

    //TODO: there's got to be a more elegant way to do all this
    let mut height_map = Vec::new();
    // less allocation /shrug
    height_map.reserve_exact(h as usize * w as usize);

    for y in 0..w {
        for x in 0..h {
            let n = noise.get_noise(((x + 1) as f32) / h as f32 * 2., ((y + 1) as f32) / 100.)
                * w as f32
                * 2.;

            height_map.push(n as f64);
        }
    }

    let sea_level = decide_sea_level(&height_map, (params.target_water as f64) / 255.).unwrap();

    let mut min_height = height_map.iter().fold(f64::INFINITY, |a, &b| a.min(b));
    let mut max_height = height_map.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));

    let ret = Map {
        dimensions: params.world_size,
        height_map,
        rivers: Vec::new(),
        sea_level,
        max_height,
        min_height,
    };

    FullWorld {
        terrain: ret,
        rivers: Vec::new(),
        pole: Pole::new(),
    }
}

/// erode a map down
/// updates heightmap & rivers(?)
pub fn erode(map: &FullWorld, master_generator: &mut RandomNumberGenerator) -> Vec<f64> {
    // most importantly, seeding infrastructure is not in place

    let mut generator = RandomNumberGenerator::seeded(master_generator.next_u64());

    // to do first: random rain erosion tile-by tile
    // glaciers?
    // next, pathfind rivers, etc
    // erode around rivers?
    // run a little more erosion

    let mut ret_hmap = map.terrain.height_map.clone();

    // rain erosion
    let passes_per_tile = 0.5; // 0.5 erosion passes per tile
    let total_passes = { (map.terrain.height_map.len() as f64 * passes_per_tile).round() } as u64;

    debug!(
        "running {} rain erosion passes for {} total tiles",
        total_passes,
        map.terrain.height_map.len()
    );

    for _ in 0..total_passes {
        // choose a random tile
    }

    todo!()
}
