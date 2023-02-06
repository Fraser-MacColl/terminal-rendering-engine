use crate::style::StyledChar;

pub struct Window {
    position: (usize, usize),
    size: (usize, usize),

    // If buffer should clear after every update, or keep previous state which can then be edited as necessary
    clear_buffer: bool,

    // Prints extra debug information at a set absolute location on terminal
    debug: bool,

    // Buffer that holds char data of this window
    // None represents transparency
    char_data: Vec<Vec<Option<StyledChar>>>,
}

impl Window {
    pub fn get_size(&self) -> &(usize, usize) { &self.size }
    pub fn set_size(&mut self, size: (usize, usize)) {
        // Resizes vecs to to fit new size

        /*
        Ok things get weird with the order of resizing

        When indexing with [x][y], it means the vecs are being stored as:
            - Outer vec of columns, each column being of the same X val
            - Inner vec of chars within the column, each having a different Y val

        So when resizing the inner vec, you have to resize to the new Y size,
        and vice versa for the outer
         */
        
        for x in 0..self.size.0 {
            self.char_data[x].resize(size.1, None)
        }
        self.char_data.resize(size.0, vec![None; size.1]);

        // Set new size var
        self.size = size
    }

    pub fn get_position(&self) -> &(usize, usize) { &self.size }
    pub fn set_position(&mut self, pos: (usize, usize)) { self.position = pos }

    pub fn set_clear_on_render(&mut self, clear: bool) {
        // Whether or not the current_buffer should be cleared every time
        // render is called. Does not affect the number of print calls
        // given the same current_buffer

        self.clear_buffer = clear;
    }
    pub fn set_debug(&mut self, debug: bool) { self.debug = debug }
}