// Re-export enums and structs used in library
// Why reinvent the wheel?
pub use crossterm::style::{
    ContentStyle, StyledContent,
    Color,
    Attribute, Attributes
};
use crossterm::style::{Print, PrintStyledContent, SetStyle};


#[derive(Clone)]
pub struct StyledChar {
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
    pub fn new(position: (usize, usize), size: (usize, usize), clear_buffer: bool) -> TerminalRenderingEngine {
        TerminalRenderingEngine {
            position,
            size,
            clear_buffer,
            display_buffer: vec![vec![None; size.1]; size.0],
            current_buffer: vec![vec![StyledChar::default(); size.1]; size.0]
        }
    }

    pub fn update_size(&mut self, size: (usize, usize)) {
        todo!()
    }

    pub fn update_pos(&mut self, pos: (usize, usize)) {
        todo!()
    }

    pub fn render(&self) {
        todo!()
    }

    fn is_valid_pos(&self, pos: &(usize, usize)) -> bool {
        // Validate pos is within area
        // Don't need to check < as unsigned (no negatives)
        if pos.0 >= self.size.0 { return false }
        if pos.1 >= self.size.1 { return false }

        true
    }

    //todo add various drawing methods (String, char, change region style etc)
    pub fn draw_char(&mut self, pos: (usize, usize), char: char) {
        if !self.is_valid_pos(&pos) { return; }
        self.current_buffer[pos.0][pos.1] = StyledChar::from(char)
    }

    pub fn draw_styled_char(&mut self, pos: (usize, usize), styled_char: StyledChar) {
        if !self.is_valid_pos(&pos) { return; }
        self.current_buffer[pos.0][pos.1] = styled_char
    }

    pub fn draw_styled_string(&mut self, pos: (usize, usize), str: String, style: ContentStyle) {
        // Pos is index to start inserting string from
        // If string exceeds area, it is ignored
        if !self.is_valid_pos(&pos) { return; }

        let mut chars = str.chars();
        for i in 0..str.len() {
            // If there is a char
            if let Some(c) = chars.next() {

                // Don't bother with the rest
                if pos.0+i >= self.size.0 { return; }

                self.current_buffer[pos.0+i][pos.1] = StyledChar { char: c, style: style.clone() }

            }
        }
    }
}