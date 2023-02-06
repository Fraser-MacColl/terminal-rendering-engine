use crossterm::style::{ContentStyle, StyledContent};

#[derive(Clone, PartialEq, Eq)]
pub struct StyledChar {
    // Default is an empty (space) char with no colouring or attributes
    pub char: char,
    pub style: ContentStyle
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