use std::fmt::Display;
use std::ops::Range;
use crossterm::style::ContentStyle;
use crate::style::StyledChar;

pub struct Window {
    position: (usize, usize), // Absolute position on screen
    depth: usize,
    size: (usize, usize),

    // If buffer should clear after every update, or keep previous state which can then be edited as necessary
    clear_buffer: bool,

    // Buffer that holds char data of this window
    // None represents transparency
    char_data: Vec<Vec<Option<StyledChar>>>,
}

impl Window {

    pub fn new(position: (usize, usize), depth: usize, size: (usize, usize)) -> Window {
        Window {
            position,
            depth,
            size,
            clear_buffer: false,
            char_data: vec![vec![None; size.1]; size.0],
        }
    }

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

    pub fn get_depth(&self) -> &usize { &self.depth }
    pub fn set_depth(&mut self, depth: usize) { self.depth = depth }

    pub fn get_position(&self) -> &(usize, usize) { &self.position }
    pub fn set_position(&mut self, pos: (usize, usize)) { self.position = pos }

    pub fn set_clear_buffer(&mut self, clear: bool) {
        // Whether or not the current_buffer should be cleared every time
        // render is called. Does not affect the number of print calls
        // given the same current_buffer

        self.clear_buffer = clear;
    }
    pub fn get_clear_buffer(&self) -> &bool { &self.clear_buffer }

    pub fn get_char(&self, pos: &(usize, usize)) -> &Option<StyledChar> {
        // Returns a ref to char information at a given location
        // If location isn't in window, just return None

        if pos.0 >= self.size.0 || pos.1 >= self.size.1 {
            return &None
        }

        &self.char_data[pos.0][pos.1]
    }





    // #-------------------#
    // | USER MANIPULATION |
    // #-------------------#

    // todo add more stuff down here
    fn is_valid_pos(&self, pos: &(usize, usize)) -> bool {
        // Validate pos is within area
        // Don't need to check < as unsigned (no negatives)
        if pos.0 >= self.size.0 { return false }
        if pos.1 >= self.size.1 { return false }

        true
    }



    pub fn draw<T: Display>(&mut self, pos: (usize, usize), item: T) {
        // Pos is index to start inserting item from
        // If item exceeds area, it is ignored
        if !self.is_valid_pos(&pos) { return; }

        let str = item.to_string();
        let mut chars = str.chars();
        for i in 0..str.len() {

            // Don't bother with the rest if indexing OOB
            if pos.0+i >= self.size.0 { return; }

            // If there is a char
            if let Some(c) = chars.next() {
                self.char_data[pos.0+i][pos.1] = Some(StyledChar::from(c))
            }
        }
    }

    pub fn draw_styled<T: Display>(&mut self, pos: (usize, usize), item: T, style: ContentStyle) {
        // Pos is index to start inserting item from
        // If item exceeds area, it is ignored
        if !self.is_valid_pos(&pos) { return; }

        let str = item.to_string();
        let mut chars = str.chars();
        for i in 0..str.len() {

            // Don't bother with the rest
            if pos.0+i >= self.size.0 { return; }

            // If there is a char
            if let Some(c) = chars.next() {
                self.char_data[pos.0+i][pos.1] = Some(StyledChar{ char: c, style: style.clone() })

            }
        }
    }



    pub fn clear(&mut self) {
        // Clears the area to all transparent chars

        for y in 0..self.size.1 {
            for x in 0..self.size.0 {
                self.char_data[x][y] = None;
            }
        }
    }

    pub fn fill(&mut self, fill_char: StyledChar) {
        // Fills area with specific char

        for y in 0..self.size.1 {
            for x in 0..self.size.0 {
                self.char_data[x][y] = Some(fill_char.clone())
            }
        }
    }

    pub fn fill_row(&mut self, row: usize, fill_char: StyledChar) {
        // Fills a row with a character

        if row >= self.size.1 { return; } // index OOB

        for x in 0..self.size.0 {
            self.char_data[x][row] = Some(fill_char.clone())
        }
    }

    pub fn fill_column(&mut self, column: usize, fill_char: StyledChar) {
        // Fills a column with a character

        if column >= self.size.0 { return; } // index OOB

        for y in 0..self.size.1 {
            self.char_data[column][y] = Some(fill_char.clone())
        }
    }



    pub fn set_style_square(&mut self, x_range: Range<usize>, y_range: Range<usize>, style: ContentStyle) {
        // Change the style for a group of chars in the square specified

        for y in y_range {
            if y >= self.size.1 { break }

            // todo Find a way of doing this without cloning
            for x in x_range.clone() { // Clone as using it here turns it into a iter, which consumes values, then being unable to use next loop
                if x >= self.size.0 { break } // Break if we're indexing outside valid area

                if let Some(c) = &mut self.char_data[x][y] {
                    c.style = style.clone()
                }
            }
        }
    }

    pub fn set_global_style(&mut self, style: ContentStyle) {
        // Sets the style for all tiles that have a char

        for y in 0..self.size.1 {
            for x in 0..self.size.0 {
                if let Some(c) = &mut self.char_data[x][y] {
                    c.style = style.clone()
                }
            }
        }
    }

}