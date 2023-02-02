use std::fmt::Display;
use crossterm::style::PrintStyledContent;
use std::io::{stdout, Write};
use crossterm::queue;
use crossterm::cursor::MoveTo;

// Re-export enums and structs used in library
// Why reinvent the wheel?
pub use crossterm::style::{
    ContentStyle, StyledContent,
    Color,
    Attribute, Attributes
};

#[derive(Clone, PartialEq, Eq)]
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

impl From<StyledChar> for StyledContent<String> {
    fn from(styled_char: StyledChar) -> Self {
        StyledContent::new(styled_char.style, styled_char.char.to_string())
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

    pub fn render(&mut self) {
        // Save cursor position, to be returned to later
        queue!(stdout(), crossterm::cursor::SavePosition).unwrap();

        let mut updated_pos = self.get_updated_positions();
        let mut commands = vec![];


        // Add a print command for every char that has been updated
        for pos in &updated_pos {
            commands.push(PrintStyledContent(self.current_buffer[pos.0][pos.1].clone().into()));
        }


        // Combine consecutive print commands that have the same styling
        self.combine_print_cmds(&mut commands, &mut updated_pos);


        // Queue each print command, add extra move mouse if they aren't consecutive
        self.queue_print_cmds(&commands, &updated_pos);


        // Update display_buffer
            // For each position in the diff list, clone the current_buffer into display_buffer position
            // If clear_buffer = true, clear the current_buffer
        for pos in &updated_pos {
            self.display_buffer[pos.0][pos.1] = Some(self.current_buffer[pos.0][pos.1].clone())
        }

        if self.clear_buffer {
            for pos in &updated_pos {
                self.current_buffer[pos.0][pos.1] = StyledChar::default()
            }
        }


        queue!(stdout(), crossterm::cursor::RestorePosition).unwrap();
        stdout().flush().unwrap();
    }
    
    fn get_updated_positions(&self) -> Vec<(usize, usize)> {
        // Checks each current_buffer tile to the relative display_buffer tile to see if anything has changed
        // Returns a list of positions that a change has occurred

        let mut vec = vec![];

        for y in 0..self.size.1 {
            for x in 0..self.size.0 {

                if self.display_buffer[x][y].is_none() {
                    vec.push((x, y))
                }

                // I can only compare the value in the display option if I unwrap,
                // but I can only unwrap if it's a ref or else it takes ownership/consumes the value
                // Therefor LHS has to be ref as well so it is same type
                else if &self.current_buffer[x][y] != self.display_buffer[x][y].as_ref().unwrap() {
                    vec.push((x, y))
                }

            }
        }

        vec
    }

    fn combine_print_cmds(&self, cmd_vec: &mut Vec<PrintStyledContent<String>>, pos_vec: &mut Vec<(usize, usize)>) {
        // Combines print cmd_vec that are consecutive and have the same styling

        let mut i = 0;
        while i < cmd_vec.len()-1 {
            // If they are NOT consecutive
            if pos_vec[i].0+1 != pos_vec[i+1].0 {
                i+=1;
                continue
            }

            if cmd_vec[i].0.style() == cmd_vec[i+1].0.style() {
                // Combine the content
                let content1 = cmd_vec[i].0.content();
                let content2 = cmd_vec[i+1].0.content();

                // I'm getting the feeling like this is bad and wrong
                let mut new_content: String = content1.to_string();
                new_content.push_str(content2.to_string().as_str());

                let new_cmd = PrintStyledContent(
                    StyledContent::new(*cmd_vec[i].0.style(), new_content)
                );

                // Replace first two items in vec with new command
                cmd_vec[i] = new_cmd;
                cmd_vec.remove(i+1);
                pos_vec.remove(i+1);

                // Skip incrementation, as we want to compare new ith object to i+1 object
                continue
            }

            // Increment
            i+=1;
        }
    }

    fn queue_print_cmds(&self, cmd_vec: &Vec<PrintStyledContent<String>>, pos_vec: &Vec<(usize, usize)>) {
        // Takes a vec of print commands, and a vec of locations for those prints
        // Adds each print cmd to queue, while also adding in mouse move cmds if necessary


        // If there's nothing to do, just exit
        if cmd_vec.is_empty() { return; }


        // abs_pos is the position on the terminal
        // Is the position in the vec, plus the engine position as an offset
        let mut abs_pos = self.to_absolute_pos(pos_vec[0]);
        queue!(stdout(), MoveTo(abs_pos.0, abs_pos.1)).unwrap();


        // len-1 because we will manually handle last command and don't want indexOOB
        for i in 0..(cmd_vec.len()-1) {
            queue!(stdout(), cmd_vec[i].clone()).unwrap(); // Clone as queue takes ownership
                                                           // I think there should be a way to use iterator
                                                           // or something, since I don't need vec after this

            // Consecutive check
            if pos_vec[i].1 == pos_vec[i+1].1         // Same Y
                && pos_vec[i].0+1 == pos_vec[i+1].0 { // Next is one x ahead
                continue
            }

            abs_pos = self.to_absolute_pos(pos_vec[i+1]);
            queue!(stdout(), MoveTo(abs_pos.0, abs_pos.1)).unwrap();
        }


        // Handle last print cmd manually
        queue!(stdout(), cmd_vec[cmd_vec.len()-1].clone()).unwrap();
    }

    // fn debug_render(&self) {
    //     for y in 0..self.size.1 {
    //         for x in 0..self.size.0 {
    //             let sc = StyledContent::new(self.current_buffer[x][y].style, self.current_buffer[x][y].char);
    //             queue!(stdout(), PrintStyledContent(sc)).unwrap();
    //         }
    //
    //         queue!(stdout(), Print("\n")).unwrap();
    //     }
    //
    //     stdout().flush().unwrap();
    // }

    fn to_absolute_pos(&self, pos: (usize, usize)) -> (u16, u16) {
        // Takes a relative position and turns it into absolute position on terminal
        // u16 so it is ready to be used in MoveTo commands

        ((pos.0+self.position.0) as u16, (pos.1+self.position.1) as u16)
    }

    



    fn is_valid_pos(&self, pos: &(usize, usize)) -> bool {
        // Validate pos is within area
        // Don't need to check < as unsigned (no negatives)
        if pos.0 >= self.size.0 { return false }
        if pos.1 >= self.size.1 { return false }

        true
    }

    //todo add various drawing methods (String, char, change region style etc)
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
}