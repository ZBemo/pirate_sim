//! TODO: SET UP LOGGER NOW
//! TODO: switch from f64 to f64 throughout the program

#![warn(clippy::pedantic, clippy::perf)]

mod helpers;
mod render;
mod worldgen;

use std::sync::mpsc;
use std::thread;
use std::{error::Error, time::UNIX_EPOCH};

use log::{trace, warn};

use helpers::RectDimension;

use crate::render::{InputPacket, RenderPacket, Renderer};

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

    trace!("Starting render test");

    //TODO: make sure Xs and Ys align with width/height correctly throughout the program x = width, y = height
    //TODO: feature to regenerate with new seed graphically?
    let gen = worldgen::GenParam {
        seed,
        poles: worldgen::Poles::Random,
        // 178/255 around 70%
        target_water: 200,
        max_ports: 100,
        max_civilizations: 18,

        world_size: RectDimension::new(70, 70),
    };

    let dimensions = RectDimension::new(80, 80);
    let ctx = bracket_lib::terminal::BTermBuilder::simple(dimensions.width, dimensions.height)?
        .with_title("Pirate Sim World Gen")
        .build()?;

    let (render_s, render_r) = mpsc::channel::<RenderPacket>();
    let (input_s, input_r) = mpsc::channel::<InputPacket>();

    let renderer = Renderer::new_blank(render_r, input_s, dimensions);

    // TODO: set up render thread, send receiver and sender to renderer
    thread::spawn(move || {
        worldgen::gen_full_world(gen, Some((render_s, input_r)));
    });

    renderer.start_render(ctx).unwrap();

    Ok(())
}

/// a test function to use while architecting renderer
fn render_test() -> Result<(), Box<dyn Error + Send + Sync>> {
    use render::*;

    let dimensions = RectDimension::new(30, 10);
    let ctx = bracket_lib::terminal::BTermBuilder::simple(dimensions.width, dimensions.height)?
        .with_title("Render Test!")
        .build()?;

    let (render_s, render_r) = mpsc::channel::<RenderPacket>();
    let (input_s, input_r) = mpsc::channel::<InputPacket>();

    let renderer = Renderer::new_blank(render_r, input_s, dimensions);

    thread::spawn(move || {
        let mut running = true;

        while running {
            if let Ok(input) = input_r.try_recv() {
                match input {
                    InputPacket::Key(_) => {
                        let message = format!(
                            "Last update at time {}",
                            std::time::SystemTime::now()
                                .duration_since(UNIX_EPOCH)
                                .unwrap()
                                .as_secs()
                        );

                        trace!("{}", message);

                        let to_display = render::string_to_frame(message);

                        render_s.send(RenderPacket::NewFrame(to_display)).unwrap();
                    }
                    InputPacket::LoopClosed => {
                        trace!("window has exited, closing loop");
                        running = false;
                    }
                }
            }
        }
    });

    renderer.start_render(ctx).unwrap();

    Ok(())
}
