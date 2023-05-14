use bracket_lib::{
    prelude::{Algorithm2D, BaseMap, FastNoise, FractalType, NoiseType},
    random::RandomNumberGenerator,
    terminal::Point,
};
use log::{debug, log_enabled, trace, warn, Level};

use crate::helpers::{Distance, RectDimension};

use super::{FullWorld, GenParam};

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

/// Does a binary search of sea levels starting from min + max / 2 until sea level is within 1 / height_map.len() of wanted_percent.
///
/// The 1 / height_map.len() variance is allowed in order to accommodate for lower resolution maps where you might not be able to get exactly the desired percent.
///
/// Should only error if a NaN is present in the input, or if the algorithm is bugged.
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
                "decreasing uw%: {}\nmid: {}\nmax: {}\nmin: {}",
                percent_uw,
                mid,
                max,
                min
            );
            max = mid + f64::EPSILON;
        } else if (percent_uw < lowest_percent) {
            trace!(
                "increasing uw%: {}\nmid: {}\nmax: {}\nmin: {}",
                percent_uw,
                mid,
                max,
                min
            );
            min = mid - f64::EPSILON;
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

pub fn gen_base_map(params: GenParam, mut rng: RandomNumberGenerator) -> FullWorld {
    let (h, w) = (params.world_size.height, params.world_size.width);
    // let map = [[0f64; std::u8::MAX as usize]; std::u8::MAX as usize];
    let mut noise = FastNoise::seeded(rng.next_u64());

    noise.set_noise_type(NoiseType::PerlinFractal);
    noise.set_fractal_type(FractalType::FBM);
    noise.set_fractal_octaves(10);
    noise.set_fractal_gain(0.5);
    noise.set_fractal_lacunarity(3.0);
    noise.set_frequency(2.0);

    //TODO: there's got to be a more elegant way to do all this
    let mut height_map = Vec::new();

    for y in 0..w {
        for x in 0..h {
            let n = noise.get_noise(((x + 1) as f32) / h as f32 * 2., ((y + 1) as f32) / 100.)
                * w as f32
                * 2.;

            height_map.push(n as f64);
        }
    }

    let sea_level = decide_sea_level(&*height_map, (params.target_water as f64) / 255.).unwrap();

    let mut min_height = height_map.iter().fold(f64::INFINITY, |a, &b| a.min(b));
    let mut max_height = height_map.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));

    // to do: ERODE MAP

    let ret = Map {
        dimensions: params.world_size,
        height_map,
        rivers: Vec::new(),
        sea_level,
        max_height,
        min_height,
    };

    return FullWorld {
        terrain: ret,
        rivers: Vec::new(),
    };
}

struct ErodeMap<'a> {
    base: &'a [f64],
    area: RectDimension,
    highest_height: f64,
    lowest_height: f64,
    sea_lvl: f64,
    seed: u64,
}

impl<'a> ErodeMap<'a> {
    pub fn is_underwater(&self, height: f64) -> bool {
        height <= self.sea_lvl
    }
    pub fn total_size(&self) -> u32 {
        self.area.width as u32 * self.area.height as u32
    }
}

impl<'a> BaseMap for ErodeMap<'a> {
    fn is_opaque(&self, _idx: usize) -> bool {
        false
    }

    fn get_available_exits(
        &self,
        idx: usize,
    ) -> bracket_lib::prelude::SmallVec<[(usize, f32); 10]> {
        let mut ret = bracket_lib::prelude::SmallVec::<[(usize, f32); 10]>::new();

        let p = self.index_to_point2d(idx);
        let p_h = self.base[idx];

        for c_p in [
            (0, 0),
            (1, 0),
            (-1, 0),
            (1, 1),
            (-1, 1),
            (1, -1),
            (0, 1),
            (0, -1),
            (-1, -1),
        ]
        .map(Point::from_tuple)
        .map(|o| o + p)
        {
            if (log_enabled!(Level::Debug)) {
                debug!("{}", {
                    let (l, r) = c_p.to_tuple();
                    format!("({},{})", l, r)
                });
            }

            let h = self.base[self.point2d_to_index(c_p)];
            if h <= self.sea_lvl {
                // rivers don't flow underwater
                continue;
            }

            // don't consider if it increases too much
            if h > p_h
                && Distance::distance(h, p_h)
                    < 3. / Distance::distance(self.sea_lvl, self.highest_height)
            {
                continue;
            }

            //it has passed all criteria, find its weight and add it as an exit

            ret.push((
                self.point2d_to_index(c_p),
                (Distance::distance(p_h, h) / Distance::distance(self.sea_lvl, self.highest_height))
                    as f32,
            ));
        }

        ret
    }

    fn get_pathing_distance(&self, idx1: usize, idx2: usize) -> f32 {
        let p1 = self.index_to_point2d(idx1);
        let p2 = self.index_to_point2d(idx2);

        bracket_lib::geometry::DistanceAlg::Diagonal.distance2d(p1, p2)
    }
}

impl<'a> Algorithm2D for ErodeMap<'a> {
    fn dimensions(&self) -> bracket_lib::terminal::Point {
        bracket_lib::terminal::Point::new(self.area.width, self.area.height)
    }
}

/// erode a map down
/// returns a new heightmap
fn erode(
    map: ErodeMap,
    area: &RectDimension,
    master_generator: &mut RandomNumberGenerator,
) -> Vec<f64> {
    // most importantly, seeding infrastructure is not in place

    let mut generator = RandomNumberGenerator::seeded(master_generator.next_u64());

    // to do first: random rain erosion tile-by tile
    // glaciers?
    // next, pathfind rivers, etc
    // erode around rivers?
    // run a little more erosion

    let mut ret_hmap = map.base.clone();

    // rain erosion
    let passes_per_tile = 0.5; // 0.5 erosion passes per tile
    let total_passes = { (map.total_size() as f64 * passes_per_tile).round() } as u64;

    debug!(
        "running {} rain erosion passes for {} total tiles",
        total_passes,
        map.total_size()
    );

    for _ in 0..total_passes {
        // choose a random tile
    }

    todo!()
}
