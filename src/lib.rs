pub mod window;
pub mod style;

use std::collections::HashMap;
use std::fmt::Display;
use std::hash::Hash;
use crossterm::style::PrintStyledContent;
use std::io::{stdout, Write};
use std::ops::Range;
use crossterm::{execute, queue};
use crossterm::cursor::MoveTo;

// Re-export enums and structs used in library
// Why reinvent the wheel?
pub use crossterm::style::{
    ContentStyle, StyledContent, Stylize,
    Color,
    Attribute, Attributes
};
use crate::style::StyledChar;
use crate::window::Window;

pub struct Engine<I> {
    // Map of windows/layers, along with an identifier for said window
    windows: HashMap<I, Window>,

    // Whether or not extra debug information should be displayed
    debug: bool,

    // Buffers for what is being currently displayed, and one that is being currently edited
    //
    // display_buffer None is an unknown state. For example if the area was extended, the chars in the new area
    // are of an unknown state; They could be anything so we can't track if they should be updated or not.
    //
    // current_buffer None represents transparency, and shouldn't updated the currently displayed char, even
    // if that is a char of unknown state.
    display_buffer: Vec<Vec<Option<StyledChar>>>,
    current_buffer: Vec<Vec<Option<StyledChar>>>,

    // Information on the buffers
    position: (usize, usize),
    size: (usize, usize)
}

impl<I: Eq + Hash> Engine<I> {

    pub fn new(position: (usize, usize), size: (usize, usize)) -> Engine<I> {
        Engine {
            windows: HashMap::new(),
            debug: false,
            display_buffer: vec![vec![None; size.1]; size.0],
            current_buffer: vec![vec![None; size.1]; size.0],
            position,
            size,
        }
    }

    pub fn set_debug(&mut self, debug: bool) { self.debug = debug }

    pub fn add_window(&mut self, identifier: I, window: Window) -> Option<Window> {
        todo!()
        // Remember to resize buffers if new windows extend beyond their bounds
    }

    pub fn get_window(&self, identifier: &I) -> Option<&Window> {
        self.windows.get(identifier)
    }





    // #-----------#
    // | RENDERING |
    // #-----------#

    pub fn render(&mut self) {
        // Main render method

        // Save cursor position, to be returned to later
        queue!(stdout(), crossterm::cursor::SavePosition).unwrap();

        self.flatten_windows();
        let updated_chars = self.get_updated_positions();

        let (cmds, pos) = self.generate_print_commands(&updated_chars);

        self.queue_print_cmds(&cmds, &pos);

        // Update display_buffer
        // For each position in the diff list, clone the current_buffer into display_buffer position
        // If clear_buffer = true, clear the current_buffer
        for pos in &updated_chars {
            self.display_buffer[pos.0][pos.1] = self.current_buffer[pos.0][pos.1].clone()
        }

        // Restore cursor, and do cmds
        queue!(stdout(), crossterm::cursor::RestorePosition).unwrap();
        stdout().flush().unwrap();

        // Extra debug printing
        // After rest of printing so it will always be on top
        if self.debug {
            let style = ContentStyle::new().black().on_white();

            let line1 = StyledContent::new(
                style,
                format!("num updt: {:6}", updated_chars.len())
            );
            let line2 = StyledContent::new(
                style,
                format!("num cmds: {:6}", cmds.len())
            );
            execute!(stdout(),
                MoveTo(0, 0), PrintStyledContent(line1),
                MoveTo(0, 1), PrintStyledContent(line2),
                crossterm::cursor::RestorePosition // Restore position again as it was moved again
            ).unwrap();
        }
    }

    fn flatten_windows(&mut self) {
        // Takes all the windows, and flattens char data from them into current_buffer

        let mut windows: Vec<&Window> = self.windows.values().collect();
        windows.sort_unstable_by_key(|k| k.get_depth()); // Sorts vec by lowest depth first

        // Iterate over windows, lowest first
        for win in windows.iter()
        {
            // Iterate through each char in win
            for y in 0..win.get_size().1 {
                for x in 0..win.get_size().0 {
                    let eng_pos = self.win_to_eng_pos(win.get_position(), &(x, y));
                    let win_char = win.get_char(&(x,y));

                    match win_char {
                        Some(_) => self.current_buffer[eng_pos.0][eng_pos.1] = win_char.clone(),
                        None    => continue
                    }
                }
            }
        }

    }

    fn get_updated_positions(&self) -> Vec<(usize, usize)> {
        // Checks each current_buffer tile to the relative display_buffer tile to see if anything has changed
        // Returns a list of positions that a change has occurred

        let mut vec = vec![];

        for y in 0..self.size.1 {
            for x in 0..self.size.0 {

                // Ignore if transparent
                if self.current_buffer[x][y].is_none() { continue }

                if self.current_buffer[x][y] != self.display_buffer[x][y] {
                    vec.push((x, y))
                }

            }
        }

        vec
    }

    fn generate_print_commands(&self, pos_vec: &Vec<(usize, usize)>) -> (Vec<PrintStyledContent<String>>, Vec<(usize, usize)>) {
        // Takes a vec of positions of updated chars and
        // generates the print commands to then be queued
        // Returns a vec of the commands and locations of said commands

        let mut cmd_vec = vec![];
        let mut cmd_pos_vec = vec![];

        // Need at least 1 item in vec after here, so we can return an empty vec if no changes
        if pos_vec.len() == 0 { return (cmd_vec, cmd_pos_vec) }

        let mut cmd_str = String::from(self.get_current_char(pos_vec[0]));
        let mut cmd_pos = pos_vec[0];
        let mut cmd_style = self.get_current_style(pos_vec[0]);

        // Comparing to previous item
        for i in 1..pos_vec.len() {

            // If current is nonconsecutive or on different row,
            // what's stored currently is a complete cmd
            if pos_vec[i-1].0+1 != pos_vec[i].0 || pos_vec[i].1 != pos_vec[i+1].1 {
                let sc = StyledContent::new(cmd_style.clone(), cmd_str);
                cmd_vec.push(PrintStyledContent(sc));
                cmd_pos_vec.push(cmd_pos);

                cmd_str = String::from(self.get_current_char(pos_vec[i]));
                cmd_pos = pos_vec[i];
                cmd_style = self.get_current_style(pos_vec[i]);

                continue
            }

            // Compare style to previous
            if self.get_current_style(pos_vec[i]) != self.get_current_style(pos_vec[i-1]) {
                let sc = StyledContent::new(cmd_style.clone(), cmd_str);
                cmd_vec.push(PrintStyledContent(sc));
                cmd_pos_vec.push(cmd_pos);

                cmd_str = String::from(self.get_current_char(pos_vec[i]));
                cmd_pos = pos_vec[i];
                cmd_style = self.get_current_style(pos_vec[i]);

                continue
            }

            // At this point, current index is consecutive, on the same row, and has the same style
            // So it can be appended to current cmd_str
            cmd_str.push(self.get_current_char(pos_vec[i]))
        }

        // One last command push to make up for the last item in pos_vec
        // It would either have started a new cmd, or be appended the last cmd
        // Either way, needs to get dealt with manually
        let sc = StyledContent::new(cmd_style.clone(), cmd_str);
        cmd_vec.push(PrintStyledContent(sc));
        cmd_pos_vec.push(cmd_pos);

        (cmd_vec, cmd_pos_vec)
    }

    fn queue_print_cmds(&self, cmd_vec: &Vec<PrintStyledContent<String>>, pos_vec: &Vec<(usize, usize)>) {
        // Takes a vec of print commands, and a vec of locations for those prints
        // Adds each print cmd to queue, while also adding in mouse move cmds if necessary

        // If there's nothing to do, just exit
        if cmd_vec.is_empty() { return; }

        // abs_pos is the position on the terminal
        let mut abs_pos = self.to_absolute_pos(pos_vec[0]);
        queue!(stdout(), MoveTo(abs_pos.0, abs_pos.1)).unwrap();

        // len-1 because we will manually handle last command and don't want indexOOB
        for i in 0..(cmd_vec.len()-1) {
            queue!(stdout(), &cmd_vec[i]).unwrap();
            // I think there should be a way to use iterator
            // or something, since I don't need vec after this

            // If consecutive, skip moving the cursor
            if pos_vec[i].1 == pos_vec[i+1].0 && pos_vec[i].0+1 == pos_vec[i+1].0 {
                continue
            }

            abs_pos = self.to_absolute_pos(pos_vec[i+1]);
            queue!(stdout(), MoveTo(abs_pos.0, abs_pos.1)).unwrap();
        }

        // Handle last print cmd manually
        queue!(stdout(), &cmd_vec[cmd_vec.len()-1]).unwrap();
    }

    fn win_to_eng_pos(&self, win_pos: &(usize, usize), pos: &(usize, usize)) -> (usize, usize) {
        // Converts a window position into a position on the engines current_buffer

        let x = pos.0 + win_pos.0 - self.position.0;
        let y = pos.1 + win_pos.1 - self.position.1;

        (x, y)
    }

    fn to_absolute_pos(&self, pos: (usize, usize)) -> (u16, u16) {
        // Takes a relative position and turns it into absolute position on terminal
        // u16 so it is ready to be used in MoveTo commands

        ((pos.0+self.position.0) as u16, (pos.1+self.position.1) as u16)
    }

    fn get_current_char(&self, pos: (usize, usize)) -> char {
        // Convenience method for getting chars in current_buffer
        // Assumes the position is valid,
        // SO ONLY USE WITH RENDER METHOD WHERE THE VEC OF POSITIONS IS DEFINITELY VALID
        self.current_buffer[pos.0][pos.1].as_ref().unwrap().char
    }
    fn get_current_style(&self, pos: (usize, usize)) -> ContentStyle {
        // Convenience method for getting styles in current_buffer
        // Assumes the position is valid,
        // SO ONLY USE WITH RENDER METHOD WHERE THE VEC OF POSITIONS IS DEFINITELY VALID
        self.current_buffer[pos.0][pos.1].as_ref().unwrap().style
    }

}













pub struct TerminalRenderingEngine {
    // End user struct that holds drawing area information

    position: (usize, usize),
    size: (usize, usize),

    // If buffer should clear after every update, or keep previous state which can then be edited as necessary
    clear_buffer: bool,

    // Prints extra debug information at a set absolute location on terminal
    debug: bool,
    default_char: StyledChar,

    // Buffers that hold what's currently displayed, along with editable buffer
    // None represents a pos with an unknown state, which will be forced to update next render
    //
    // If render area is resized larger, terminal chars with unknown states will be within region.
    // If they already are styled, this won't necessarily be updated unless forced to do so.
    display_buffer: Vec<Vec<Option<StyledChar>>>,
    current_buffer: Vec<Vec<StyledChar>>
}

impl TerminalRenderingEngine {

    pub fn new(position: (usize, usize), size: (usize, usize)) -> TerminalRenderingEngine {
        TerminalRenderingEngine {
            position,
            size,
            clear_buffer: false,
            debug: false,
            default_char: StyledChar::default(),
            display_buffer: vec![vec![None; size.1]; size.0],
            current_buffer: vec![vec![StyledChar::default(); size.1]; size.0]
        }
    }

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
            // If there is a char
            if let Some(c) = chars.next() {

                // Don't bother with the rest
                if pos.0+i >= self.size.0 { return; }

                self.current_buffer[pos.0+i][pos.1] = StyledChar::from(c)

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
            // If there is a char
            if let Some(c) = chars.next() {

                // Don't bother with the rest
                if pos.0+i >= self.size.0 { return; }

                self.current_buffer[pos.0+i][pos.1] = StyledChar{ char: c, style: style.clone() }

            }
        }
    }

    pub fn clear(&mut self) {
        // Clears the area with default char

        for y in 0..self.size.1 {
            for x in 0..self.size.0 {
                self.current_buffer[x][y] = self.default_char.clone();
            }
        }
    }
    pub fn fill(&mut self, fill_char: StyledChar) {
        // Fills area with specific char

        for y in 0..self.size.1 {
            for x in 0..self.size.0 {
                self.current_buffer[x][y] = fill_char.clone()
            }
        }
    }
    pub fn fill_row(&mut self, row: usize, fill_char: StyledChar) {
        // Fills a row with a character

        if row >= self.size.1 { return; } // index OOB

        for x in 0..self.size.0 {
            self.current_buffer[x][row] = fill_char.clone()
        }
    }
    pub fn fill_column(&mut self, column: usize, fill_char: StyledChar) {
        // Fills a column with a character

        if column >= self.size.0 { return; } // index OOB

        for y in 0..self.size.1 {
            self.current_buffer[column][y] = fill_char.clone()
        }
    }

    pub fn set_style_square(&mut self, x_range: Range<usize>, y_range: Range<usize>, style: ContentStyle) {
        // Change the style for a group of chars in the square specified

        for y in y_range {
            if y >= self.size.1 { break }

            for x in x_range.clone() { // Clone as using it here turns it into a iter, which consumes values, then being unable to use next loop
                if x >= self.size.0 { break } // Break if we're indexing outside valid area

                self.current_buffer[x][y].style = style.clone()
            }
        }
    }
    pub fn set_global_style(&mut self, style: ContentStyle) {
        // Sets the style for all tiles

        for y in 0..self.size.1 {
            for x in 0..self.size.0 {
                self.current_buffer[x][y].style = style.clone()
            }
        }
    }
}