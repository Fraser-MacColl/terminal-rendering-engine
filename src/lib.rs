// Re-export enums and structs used in library
// Why reinvent the wheel?
pub use crossterm::style::{
    ContentStyle, StyledContent,
    Color,
    Attribute, Attributes
};



#[derive(Clone)]
struct StyledChar {
    // Default is an empty (space) char with no colouring or attributes
    char: char,
    style: ContentStyle
}

impl From<char> for StyledChar {
    fn from(char: char) -> Self {
        StyledChar {
            char,
            style: ContentStyle::default()
        }
    }
}

impl Default for StyledChar {
    fn default() -> Self {
        StyledChar {
            char: ' ',
            style: ContentStyle::default() //Default is None colour, no attributes
        }
    }
}



pub struct TerminalRenderingEngine {
    // End user struct that holds drawing area information

    position: (usize, usize),
    size: (usize, usize),

    // If buffer should clear after every update, or keep previous state which can then be edited as necessary
    clear_buffer: bool,

    // Buffers that hold what's currently displayed, along with editable buffer
    // None represents a pos with an unknown state, which will be forced to update next render
    //
    // If render area is resized larger, terminal chars with unknown states will be within region.
    // If they already are styled, this won't necessarily be updated unless forced to do so.
    display_buffer: Vec<Vec<Option<StyledChar>>>,
    current_buffer: Vec<Vec<StyledChar>>
}

impl TerminalRenderingEngine {
    fn new(position: (usize, usize), size: (usize, usize), clear_buffer: bool) -> TerminalRenderingEngine {
        TerminalRenderingEngine {
            position,
            size,
            clear_buffer,
            display_buffer: vec![vec![None; size.1]; size.0],
            current_buffer: vec![vec![StyledChar::default(); size.1]; size.0]
        }
    }

    fn update_size(&mut self, size: (usize, usize)) {
        todo!()
    }

    fn update_pos(&mut self, pos: (usize, usize)) {
        todo!()
    }

    fn render(&self) {
        todo!()
    }

    //todo add various drawing methods (String, char, change region style etc)
}