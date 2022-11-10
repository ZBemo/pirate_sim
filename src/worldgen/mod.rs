//TODO: remember to remove this later
#![allow(unused)]

mod visualize;

use std::collections::HashSet;

use bracket_lib::pathfinding::BaseMap;

use bracket_lib::prelude::Algorithm2D;
use bracket_lib::terminal::Point;
use bracket_lib::{
    noise::{FastNoise, FractalType, NoiseType},
    random::RandomNumberGenerator,
};

use crate::helpers::{index_to_point, point_to_index, Distance};

use super::basics::RectArea;

/// a river or body of water
pub struct River {
    // which tiles are covered by the water form
    covered_tiles: Vec<u32>,
}

/// a static map of a world with geological features
pub struct Map {
    pub size: RectArea,
    /// the level at which something goes into the sea/underwater
    pub sea_level: f32,
    pub min_height: f32,
    pub max_height: f32,
    pub height_map: Vec<f32>,
    pub rivers: Vec<River>,
}

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
    /// what percentage of the world should be water 1-100 with a 10% error margin before reset
    pub target_water: u8,
    /// the maximum amount of ports to generate. 0-255
    pub max_ports: u8,
    /// the maximum amount of civilizations to generate. 0-255 accepted, stick to 0-30
    pub max_civilizations: u8,
    /// the total size of the world
    pub world_size: RectArea,
}

fn gen_region(param: GenParam) {}

pub fn gen_map(params: GenParam) -> Map {
    let (h, w) = (params.world_size.height, params.world_size.width);
    // let map = [[0f32; std::u8::MAX as usize]; std::u8::MAX as usize];
    let mut rng = RandomNumberGenerator::seeded(params.seed);
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

            height_map.push(n);
        }
    }

    let sea_level = decide_sea_level(&*height_map, (params.target_water as f32) / 255.).unwrap();

    let mut min_height = height_map.iter().fold(f32::INFINITY, |a, &b| a.min(b));
    let mut max_height = height_map.iter().fold(f32::NEG_INFINITY, |a, &b| a.max(b));

    let ret = Map {
        size: params.world_size,
        height_map,
        rivers: Vec::new(),
        sea_level,
        max_height,
        min_height,
    };

    return ret;
}

struct ErodeMap<'a> {
    base: &'a [f32],
    area: RectArea,
    highest_height: f32,
    lowest_height: f32,
    sea_lvl: f32,
    seed: u64,
}

impl<'a> ErodeMap<'a> {
    fn is_underwater(&self, height: f32) -> bool {
        height <= self.sea_lvl
    }
    fn total_size(&self) -> u32 {
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
            println!("{}", {
                let (l, r) = c_p.to_tuple();
                format!("({},{})", l, r)
            });

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
                Distance::distance(p_h, h) / Distance::distance(self.sea_lvl, self.highest_height),
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
/// maybe A* towards inland regions below sea level with higher heights weighted lower.
/// erode out from there?
fn erode(map: ErodeMap, area: &RectArea) -> Vec<f32> {
    // first: randomly erode, so that water can move inland or outland

    let mut eroded_height_map: Vec<f32> = map.base.iter().copied().collect();
    // initialize seed
    let mut rng = RandomNumberGenerator::seeded(map.seed);
    let erosion_weight = 0.05;
    let is_land = |h: f32| h > map.sea_lvl;

    // make white noise for a very very loose approximation of erosion
    // in the future maybe simulate rainfall

    let white_noise = bracket_lib::noise::FastNoise::seeded(rng.next_u64());

    for (i, h) in eroded_height_map.iter_mut().enumerate() {
        // set x ,y
        let y = i % map.area.height as usize;
        let x = (i - y) / map.area.width as usize;

        if (is_land(*h)) {
            *h -= white_noise.get_noise(x as f32, y as f32).abs() * erosion_weight;
        }
    }

    // pathfind in from shores towards low points

    let shores = HashSet::<(u8, u8)>::new();

    // find sea shores.
    let is_sea_map: Vec<_> = eroded_height_map.iter().map(|h| is_land(*h)).collect();
    let mut shore_indices = HashSet::new();
    let mut i = map.area.width as usize + 1;

    loop {
        let is_land = is_sea_map[i];
        if is_land {
            let (x, y) = index_to_point(i, map.area.width as usize, map.area.height as usize);
            // loop over points around this point
            for (x, y) in [
                (x, y),
                (x, y + 1),
                (x + 1, y + 1),
                (x - 1, y + 1),
                (x - 1, y),
                (x + 1, y),
                (x, y - 1),
                (x + 1, y - 1),
                (x - 1, y - 1),
            ] {
                let p = is_sea_map[point_to_index(x, y, map.area.width as usize)];
                if !p {
                    shore_indices.insert(point_to_index(x, y, map.area.width as usize));
                }
            }
        }

        // update i
        i += 1;
        if i % map.area.width as usize == 0 {
            i += map.area.width as usize + 1;
        }
        if i >= map.area.width as usize * map.area.height as usize {
            break;
        }
    }

    // find points below sea level inland, determine what is / isn't inland

    let mut watersheds = Vec::<usize>::new();
    let mut connected_to_corner = Vec::<bool>::new();
    let (mut x, mut y) = (0usize, 0usize);
    let mut x_margin = 0;
    let mut y_margin = 0;

    connected_to_corner.resize(is_sea_map.len(), false);
    connected_to_corner.fill(false);

    // check if (x,y) is connected to the edge of the map by sea.
    // if it is, then it sets it to true in connected_to_corner
    let mut check_connected = |x: usize, y: usize| {
        if !is_sea_map[point_to_index(x, y, map.area.width as usize)] {
            return;
        }
        if x == 0 || y == 0 || x == map.area.width as usize || y == map.area.height as usize {
            // on the edge of map.
            connected_to_corner[point_to_index(x, y, map.area.width as usize)] = true;
        } else {
            for (cx, cy) in [
                (x - 1, y - 1),
                (x, y - 1),
                (x + 1, y - 1),
                (x - 1, y),
                (x, y),
                (x + 1, y),
                (x - 1, y + 1),
                (x, y + 1),
                (x + 1, y + 1),
            ] {
                if connected_to_corner[point_to_index(cx, cy, map.area.width as usize)] {
                    connected_to_corner[point_to_index(x, y, map.area.width as usize)] = true;
                }
            }
        }
    };

    // do left side
    while x_margin < map.area.width as usize / 2 && y_margin < map.area.height as usize / 2 {
        // top left corner + margin
        x = x_margin;
        y = y_margin;

        // go right until you hit margin
        while x <= map.area.width as usize - x_margin {
            check_connected(x, y);
            x += 1;
        }
        // go down until you hit margin
        while y <= map.area.height as usize - y_margin {
            check_connected(x, y);
            y += 1
        }
        // go left until you hit margin
        while x >= x_margin {
            check_connected(x, y);
            x -= 1;
        }
        // go up until you hit margin
        while y >= y_margin {
            check_connected(x, y);
            y -= 1;
        }

        // increase margins
        if (x_margin < map.area.width as usize / 2) {
            x_margin += 1;
        }
        if (y_margin < map.area.height as usize / 2) {
            y_margin += 1;
        }
    }

    // for starting pathfinding, find a close shore -> pathfind in from there
    // i is index, cornered is wether it is connected to corner and underwater is if it's under sea
    // level
    for (i, (&cornered, &underwater)) in connected_to_corner
        .iter()
        .zip(is_sea_map.iter())
        .enumerate()
    {
        // if  both underwater, but not connected to corner, then we want to pathfind river(s) in
        // towards it
        if underwater && !cornered {
            // pathfind to a shore
            // no idea how to pick which shore... just use random number?

            let mut dice = bracket_lib::random::RandomNumberGenerator::seeded(map.seed);

            // roll until we find a tile that's sea
            let start_idx = loop {
                let roll = dice.roll_dice(1, map.area.area() as i32);

                if connected_to_corner[roll as usize] {
                    break roll;
                }
            };
        }
    }

    todo!()
}

/// Does a binary search of sea levels starting from min + max / 2 until sea level is within 1 / height_map.len() of wanted_percent.
///
/// The 1 / height_map.len() variance is allowed in order to accommodate for lower resolution maps where you might not be able to get exactly the desired percent.
///
/// Should only error if a NaN is present in the input, or if the algorithm is bugged.
fn decide_sea_level(height_map: &[f32], wanted_percent: f32) -> Result<f32, ()> {
    let mut min = height_map.iter().fold(f32::INFINITY, |a, &b| a.min(b));
    let mut max = height_map.iter().fold(f32::NEG_INFINITY, |a, &b| a.max(b));
    let lowest_percent = wanted_percent - 1. / height_map.len() as f32;
    let highest_percent = wanted_percent + 1. / height_map.len() as f32;

    let percent_uw = |sl: &f32| {
        height_map
            .iter()
            .fold(0., |t, h| if h <= sl { t + 1. } else { t })
            / (height_map.len() as f32)
    };

    loop {
        // mid != min
        #[allow(clippy::similar_names)]
        let mid = (min + max) * 0.5;

        let percent_uw = percent_uw(&mid);
        if (percent_uw > highest_percent) {
            println!(
                "decreasing uw%: {}\nmid: {}\nmax: {}\nmin{}",
                percent_uw, mid, max, min
            );
            max = mid + f32::EPSILON;
        } else if (percent_uw < lowest_percent) {
            println!(
                "increasing uw%: {}\nmid: {}\nmax: {}\nmin{}",
                percent_uw, mid, max, min
            );
            min = mid - f32::EPSILON;
        } else {
            return Ok(mid);
        }

        //TODO: yeah, this is broken idc because it seems like it still ends up with a close enough
        // percent anyways
        // assert!(min > max, "min gt max");
    }

    Err(())
}
