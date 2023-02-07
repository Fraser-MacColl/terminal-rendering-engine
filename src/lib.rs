pub mod window;
pub mod style;

use std::collections::HashMap;
use std::hash::Hash;
use crossterm::style::PrintStyledContent;
use std::io::{stdout, Write};
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

    pub fn new(identifier: I, win: Window) -> Engine<I> {
        let position = win.get_position();
        let size = win.get_size();
        let mut e = Engine {
            windows: HashMap::new(),
            debug: false,
            display_buffer: vec![vec![None; size.1]; size.0],
            current_buffer: vec![vec![None; size.1]; size.0],
            position: position.clone(),
            size: size.clone(),
        };

        e.windows.insert(identifier, win);

        e
    }

    pub fn set_debug(&mut self, debug: bool) { self.debug = debug }

    pub fn add_window(&mut self, identifier: I, window: Window) -> Option<Window> {
        // Add window to hashmap
        // Checks to make sure internal vec sizes and the like don't need updating

        // x Left
        if window.get_position().0 < self.position.0 {
            let diff = self.position.0 - window.get_position().0;

            self.position.0 -= diff;
            self.size.0 += diff;

            // Have to add diff number of new rows
            for _ in 0..diff { self.current_buffer.insert(0, vec![None; self.size.1]); }
            for _ in 0..diff { self.display_buffer.insert(0, vec![None; self.size.1]); }
        }

        // x Right
        if window.get_position().0+window.get_size().0 > self.position.0+self.size.0 {
            // Idk why but without brackets it adds weird to way too big a num
            let diff = (window.get_position().0+window.get_size().0) - (self.position.0+self.size.0);

            self.size.0 += diff;

            self.current_buffer.resize(self.size.0, vec![None; self.size.1]);
            self.display_buffer.resize(self.size.0, vec![None; self.size.1]);
        }

        // y Up
        if window.get_position().1 < self.position.1 {
            let diff = self.position.1 - window.get_position().1;

            self.position.1 -= diff;
            self.size.1 += diff;

            for x in &mut self.current_buffer {
                // Have to add a new element for each new line
                for _ in 0..diff { x.insert(0, None) }
            }
            for x in &mut self.display_buffer {
                for _ in 0..diff { x.insert(0, None) }
            }
        }

        // y Down
        if window.get_position().1+window.get_size().1 > self.position.1+self.size.1 {
            // See x Right
            let diff = (window.get_position().1+window.get_size().1) - (self.position.1+self.size.1);

            self.size.1 += diff;

            for x in &mut self.current_buffer { x.resize(self.size.1, None) }
            for x in &mut self.display_buffer { x.resize(self.size.1, None) }
        }

        self.windows.insert(identifier, window)
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
            let line3 = StyledContent::new(
                style,
                format!("size: {:3?}", self.size)
            );
            let line4 = StyledContent::new(
                style,
                format!("pos:  {:3?}", self.position)
            );

            execute!(stdout(),
                MoveTo(0, 0), PrintStyledContent(line1),
                MoveTo(0, 1), PrintStyledContent(line2),
                MoveTo(0, 2), PrintStyledContent(line3),
                MoveTo(0, 3), PrintStyledContent(line4),
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
            if pos_vec[i-1].0+1 != pos_vec[i].0 || pos_vec[i].1 != pos_vec[i-1].1 {
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
        let sc = StyledContent::new(cmd_style, cmd_str);
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