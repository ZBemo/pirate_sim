//! a renderer meant to live in its own thread and be passed "RenderRequests"
//! passing back any inputs from the player, if necessary
//!
//! might have to handle logging on single thread as well?

use std::{
    collections::HashMap,
    error::Error,
    sync::mpsc::{Receiver, Sender},
};

use bracket_lib::terminal::{
    main_loop, to_cp437, BTerm, ColorPair, GameState, VirtualKeyCode, BLACK, RGBA, WHITE,
};
use log::{debug, trace, warn};

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

pub enum OffsetX {
    Left(i8),
    Righ(i8)
}

pub enum OffsetY {
    Above(i8),
    Below(i8)
}

pub struct GUI {
    // offset from the rendered frame
    // - x for offset from left, -y for offset from bottom
    // TODO: better way to specify offset
    pub offset: (OffsetX,OffsetY),
    pub to_render: Option<Frame>,
}

/// a packet to the renderer telling it what to render
/// might have to change to a trait
pub enum RenderPacket {
    NewFrame(Frame),
    #[allow(unused)]
    ChangeSize(RectDimension),
    // priority
    RegisterGUI(u8, Option<GUI>),
    // priority,
    RegisterGUIs(u8, Vec<Option<GUI>>),
    UpdateGUI {
        id: u8,
        update: Frame,
    },
}

/// a packet from the renderer forwarding IO
///
/// should renderer send a tick packet?
pub enum RenderTick {
    Key(VirtualKeyCode),
    LoopClosed,
    // priority, id
    RegisteredGUI(u8, usize),
    // priority, ids
    RegisteredGUIs(u8, Vec<usize>), // TODO: error packet
}

/// information necessary to render a single frame
#[derive(Debug, Clone)]
pub struct Frame {
    pub to_render: Vec<Tile>,
    pub dimensions: RectDimension,
}

/// For rendering. all rendering must be on main thread or X11 gets mad
pub struct Renderer {
    receiver: Receiver<RenderPacket>,
    sender: Sender<RenderTick>,
    cur_frame: Frame,
    should_rerender: bool,
    // storing large-ish frames and extremely small Offset{X,Y} pair seperately makes it so we can
    // loop through offsets multiple times with far less performance hit
    gui_frames: Vec<(u8,Option<Frame>)>,
    gui_offset: Vec<(OffsetX,OffsetY)>,
}

/// take a string and construct a frame to render that string 
///
/// It's the caller's job to ensure that the rendering window has proper dimensions for the
/// returned frame
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
            self.sender.send(RenderTick::LoopClosed).unwrap();
        }

        // a key is pressed?
        if let Some(key) = ctx.key {
            match self.sender.send(RenderTick::Key(key)) {
                Ok(_) => {}
                Err(e) => {
                    debug!(
                        "Unable to send packet key from render thread to work thread. error: {}. This may only be an issue if your game goes unresponsive.",
                        e.to_string()
                    );
                }
            };
        };

        if let Ok(r_packet) = self.receiver.try_recv() {
            match r_packet {
                // a new frame has been requested
                RenderPacket::NewFrame(frame) => {
                    // need to update frame
                    self.should_rerender = true;
                    self.cur_frame = frame;
                }
                RenderPacket::ChangeSize(new_size) => {
                    ctx.set_char_size_and_resize_window(
                        new_size.width as u32,
                        new_size.height as u32,
                    );
                }
                RenderPacket::RegisterGUI(priority, to_register) => {
                    let id = self.register_gui(priority, to_register);

                    self.sender
                        .send(RenderTick::RegisteredGUI(priority, id))
                        .unwrap();
                }
                RenderPacket::RegisterGUIs(priority, to_register) => {
                    let ids = to_register
                        .into_iter()
                        .map(|gui| {
                            let id = self.register_gui(priority, gui);
                            id
                        })
                        .collect();

                    self.sender
                        .send(RenderTick::RegisteredGUIs(priority, ids))
                        .unwrap();
                }
                RenderPacket::UpdateGUI { id, update } => {
                    todo!();
                } // Query packet?
            };
        }

        // check for render packets

        if self.should_rerender {
            // do render
            // TODO: render diffing if render takes large amounts of time
            ctx.cls();

            // render gui first, keep track of already rendered areas, gui offset amt?
            let mut x_offset = 0;
            let mut y_offset = 0;
            let mut rendered_ponts = Vec::new();
            rendered_ponts.resize(ctx.get_char_size().0 as usize* ctx.get_char_size().1 as usize, false);


            // TODO: 0 reason to clone this
            // Option<Frame> maybe? would be hacky, but definetly better perf
            let cf = self.cur_frame.clone();
            self.render_frame(&cf, ctx, (0, 0));

            // sort & render guis next

            // TODO this is so sketch
            let mut to_sort: Vec<_> = (0..self.guis.len()).into_iter().collect();

            to_sort.sort_by_key(|idx| self.guis[*idx].0);

            for idx in to_sort {
                if let Some(to_render) = &self.guis[idx].1 {
                    let x,y;  
                    if (to_render.offset.0  > 0 ){
                        x = 
                    }
                }
            }

            // render has been brought up to date
            self.should_rerender = false;
        }
    }
}

impl Renderer {
    fn render_frame(&mut self, frame: &Frame, ctx: &mut BTerm, start_point: (u8, u8)) -> () {
        trace!(
            "rendering frame with dimension ({},{}) and offset ({},{}).",
            frame.dimensions.width,
            frame.dimensions.height,
            start_point.0,
            start_point.1
        );
        for i in 0..frame.to_render.len() {
            let (local_x, local_y) = frame.dimensions.index_to_point(i);
            let x = local_x + start_point.0 as usize;
            let y = local_y + start_point.1 as usize;

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

            let ct = self.cur_frame.to_render[i];

            ctx.set(x, y, ct.colors.fg, ct.colors.bg, to_cp437(ct.char));
        }
    }

    fn register_gui(&mut self, priority: u8, gui: Option<GUI>) -> usize {
        // register gui, sort,

        let new_id = self.guis.len();

        if let &Some(_) = &gui {
            self.should_rerender = true;
        }

        self.guis.push((priority, gui));

        new_id
    }

    /// a new renderer that will start with a blank screen
    pub fn new_blank(
        receiver: Receiver<RenderPacket>,
        sender: Sender<RenderTick>,
        dimensions: RectDimension,
    ) -> Self {
        Self {
            receiver,
            sender,
            cur_frame: Frame::new(dimensions),
            should_rerender: false,
            guis: Vec::new(),
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
