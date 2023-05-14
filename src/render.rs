//! a renderer meant to live in its own thread and be passed "RenderRequests"
//! passing back any inputs from the player, if necessary
//!
//! might have to handle logging on single thread as well?

use std::{
    error::Error,
    sync::mpsc::{Receiver, Sender},
};

use bracket_lib::terminal::{
    main_loop, to_cp437, BTerm, ColorPair, GameState, VirtualKeyCode, BLACK, RGBA, WHITE,
};
use log::{debug, warn};

use crate::helpers::RectDimension;

#[derive(Debug, Clone, Copy)]
pub struct Tile {
    colors: ColorPair,
    char: char,
}

impl Tile {
    pub fn new<FG: Into<RGBA>, BG: Into<RGBA>>(char: char, fg: FG, bg: BG) -> Self {
        Tile {
            colors: ColorPair::new(fg, bg),
            char,
        }
    }
}

/// a packet to the renderer telling it what to render
/// might have to change to a trait
pub enum RenderPacket {
    NewFrame(Frame),
    #[allow(unused)]
    ChangeSize(RectDimension),
}

/// a packet from the renderer forwarding IO
///
/// should renderer send a tick packet?
pub enum InputPacket {
    Key(VirtualKeyCode),
    LoopClosed,
}

/// information necessary to render a single frame
pub struct Frame {
    pub to_render: Vec<Tile>,
    pub dimensions: RectDimension,
}

/// For rendering. all rendering must be on main thread or X11 gets mad
pub struct Renderer {
    receiver: Receiver<RenderPacket>,
    sender: Sender<InputPacket>,
    cur_frame: Frame,
    should_rerender: bool,
}

/// take a string and construct a frame that is able to render that string.
/// good for quick messages
///
/// dimensions will be updated to fit this frame
pub fn string_to_frame(string: String) -> Frame {
    let dimension = if string.len() > 100 {
        let x = 100;
        let y = (string.len() - 100) / 100 + 1;

        RectDimension::new(x as u8, y as u8)
    } else {
        RectDimension::new(string.len() as u8, 3)
    };

    let tiles = string.chars().into_iter().map(|c| Tile {
        char: c,
        colors: ColorPair {
            fg: WHITE.into(),
            bg: BLACK.into(),
        },
    });

    Frame {
        to_render: tiles.collect(),
        dimensions: dimension,
    }
}

impl GameState for Renderer {
    // the main loop of the renderer
    fn tick(&mut self, ctx: &mut bracket_lib::terminal::BTerm) {
        if ctx.quitting {
            self.sender.send(InputPacket::LoopClosed).unwrap();
        }

        // a key is pressed?
        if let Some(key) = ctx.key {
            match self.sender.send(InputPacket::Key(key)) {
                Ok(_) => {}
                Err(e) => {
                    debug!(
                        "Unable to send packet key from render thread to work thread. error: {}. This may only be an issue if your game is unresponsive.",
                        e.to_string()
                    );
                }
            };
        };

        // a new frame has been requested
        if let Ok(r_packet) = self.receiver.try_recv() {
            // switch enum

            match r_packet {
                RenderPacket::NewFrame(frame) => {
                    // need to update frame
                    self.should_rerender = true;
                    self.cur_frame = frame;
                }
                RenderPacket::ChangeSize(_new_size) => {
                    todo!()
                }
            };
        }

        // check for render packets

        if self.should_rerender {
            // do render
            // TODO: render diffing if render takes large amounts of time
            ctx.cls();

            // let cf = self.cur_frame;
            let cf_dimensions = self.cur_frame.dimensions.clone();

            debug!(
                "rendering frame with dimension {},{}",
                cf_dimensions.width, cf_dimensions.height
            );

            for p in 0..self.cur_frame.to_render.len() {
                let (x, y) = cf_dimensions.index_to_point(p);
                // don't keep printing to empty screen
                // this will break on resizing
                if x as u32 > ctx.get_char_size().0 {
                    warn!(
                            "CANCELLING PRINTING! x of {} printing outside of screen. max possible x is {}.",
                            x,
                            ctx.get_char_size().0
                        );
                    break;
                }
                if y as u32 > ctx.get_char_size().1 {
                    warn!(
                            "CANCELLING PRINTING! y of {} printing outside of screen. max possible x is {}.",
                            x,
                            ctx.get_char_size().1
                        );
                    break;
                }

                let ct = self.cur_frame.to_render[p];

                ctx.set(x, y, ct.colors.fg, ct.colors.bg, to_cp437(ct.char));
            }

            // render has been brought up to date
            self.should_rerender = false;
        }
    }
}

impl Renderer {
    /// a new renderer that will start with a blank screen
    pub fn new_blank(
        receiver: Receiver<RenderPacket>,
        sender: Sender<InputPacket>,
        dimensions: RectDimension,
    ) -> Self {
        Self {
            receiver,
            sender,
            cur_frame: Frame::new(dimensions),
            should_rerender: false,
        }
    }

    /// meant to be spawned in seperate render thread
    pub fn start_render(self, ctx: BTerm) -> Result<(), Box<dyn Error + Send + Sync>> {
        // start ticking here
        main_loop(ctx, self)?;

        Ok(())
    }
}
impl Frame {
    fn new(dimensions: RectDimension) -> Self {
        Self {
            to_render: Vec::new(),
            dimensions,
        }
    }
}
