use std::ops::{Add, Mul};
use std::cmp;
use ncurses::*;

// These 2 Variables represent color pairs.
pub const REGULAR_PAIR: i16 = 0;
pub const HIGHLIGHT_PAIR: i16 = 10;

// struct Vec2 represents a 2D vector(of i32), 
// for storing coordinates of terminal.
#[derive(Default, Copy, Clone)]
pub struct Vec2 {
    x: i32,
    y: i32,
}

// Implementation of the Vec2 struct
impl Add for Vec2 {
    type Output = Vec2;

    // Function to add two vectors
    fn add(self, rhs: Vec2) -> Vec2 {
        Vec2 {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

// Function to multiply two vectors
impl Mul for Vec2 {
    type Output = Vec2;

    fn mul(self, rhs: Vec2) -> Vec2 {
        Vec2 {
            x: self.x * rhs.x,
            y: self.y * rhs.y,
        }
    }
}

// Function initialize a vector which returns a vector
impl Vec2 {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

pub enum LayoutKind {
    Vert,
    Horz,
}

// This code Defines a struct Layout for managing with layout structures and enum LayoutKind
pub struct Layout {
    // LayoutKind defines 2 types of the layouts vertical or horizontal, 
    // Specifies the kind of Layout
    kind: LayoutKind,
    pos: Vec2,
    size: Vec2,
}

// Layout has 2 functions implemented
//      1. `available_pos` for getting the available position for adding a widget.
//          (By incrementing the vertical and horizontal values until the position is available)
//      2. `add_widget` to actually add widget on the screen
impl Layout {
    pub fn available_pos(&self) -> Vec2 {
        use LayoutKind::*;
        match self.kind {
            Horz => self.pos + self.size * Vec2::new(1, 0),
            Vert => self.pos + self.size * Vec2::new(0, 1),
        }
    }

    pub fn add_widget(&mut self, size: Vec2) {
        use LayoutKind::*;
        match self.kind {
            Horz => {
                self.size.x += size.x;
                self.size.y = cmp::max(self.size.y, size.y);
            }
            Vert => {
                self.size.x = cmp::max(self.size.x, size.x);
                self.size.y += size.y;
            }
        }
    }
}

// This defines a struct Ui to manage UI and Layouts
#[derive(Default)]
pub struct Ui {
    pub layouts: Vec<Layout>,
    pub key: Option<i32>,
}

// Defines the functions to work with the UI of the App
impl Ui {
    // beign : It Initialize new layout in UI
    pub fn begin(&mut self, pos: Vec2, kind: LayoutKind) {
        assert!(self.layouts.is_empty());
        self.layouts.push(Layout {
            kind,
            pos,
            size: Vec2::new(0, 0),
        })
    }

    // begin_layout : To start a new nested layout in UI.
    pub fn begin_layout(&mut self, kind: LayoutKind) {
        let layout = self
            .layouts
            .last()
            .expect("Can't create a layout outside of Ui::begin() and Ui::end()");
        let pos = layout.available_pos();
        self.layouts.push(Layout {
            kind,
            pos,
            size: Vec2::new(0, 0),
        });
    }

    // end_layout : To close the current nested layout in UI.
    pub fn end_layout(&mut self) {
        let layout = self
            .layouts
            .pop()
            .expect("Unbalanced Ui::begin_layout() and Ui::end_layout() calls.");
        self.layouts
            .last_mut()
            .expect("Unbalanced Ui::begin_layout() and Ui::end_layout() calls.")
            .add_widget(layout.size);
    }

    // `label_fixed_width` : To **render a fixed-width label** in current layout.
    pub fn label_fixed_width(&mut self, text: &str, width: i32, pair: i16) {
        // TODO(#17): Ui::label_fixed_width() does not elide the text when width < text.len()
        let layout = self
            .layouts
            .last_mut()
            .expect("Trying to render label outside of any layout");
        let pos = layout.available_pos();

        mv(pos.y, pos.x);
        attron(COLOR_PAIR(pair));
        addstr(text);
        attroff(COLOR_PAIR(pair));

        layout.add_widget(Vec2::new(width, 1));
    }

    // edit_field : To Interactively edit Tasks.
    pub fn edit_field(&mut self, buffer: &mut String, cursor: &mut usize, width: i32) {
        let layout = self
            .layouts
            .last_mut()
            .expect("Trying to render edit field outside of any layout");
        let pos = layout.available_pos();

        if *cursor > buffer.len() {
            *cursor = buffer.len();
        }

        if let Some(key) = self.key.take() {
            match key {
                32..=126 => {
                    if *cursor >= buffer.len() {
                        buffer.push(key as u8 as char);
                    } else {
                        buffer.insert(*cursor, key as u8 as char);
                    }
                    *cursor += 1;
                }
                constants::KEY_LEFT => {
                    if *cursor > 0 {
                        *cursor -= 1
                    }
                }
                constants::KEY_RIGHT => {
                    if *cursor < buffer.len() {
                        *cursor += 1;
                    }
                }
                constants::KEY_BACKSPACE => {
                    if *cursor > 0 {
                        *cursor -= 1;
                        if *cursor < buffer.len() {
                            buffer.remove(*cursor);
                        }
                    }
                }
                constants::KEY_DC => {
                    if *cursor < buffer.len() {
                        buffer.remove(*cursor);
                    }
                }
                _ => {
                    self.key = Some(key);
                }
            }
        }

        // Buffer
        {
            mv(pos.y, pos.x);
            attron(COLOR_PAIR(REGULAR_PAIR));
            addstr(buffer);
            attroff(COLOR_PAIR(REGULAR_PAIR));
            layout.add_widget(Vec2::new(width, 1));
        }

        // Cursor
        {
            mv(pos.y, pos.x + *cursor as i32);
            attron(COLOR_PAIR(HIGHLIGHT_PAIR));
            addstr(buffer.get(*cursor..=*cursor).unwrap_or(" "));
            attroff(COLOR_PAIR(HIGHLIGHT_PAIR));
        }
    }

    // label : For Rendering labels
    #[allow(dead_code)]
    pub fn label(&mut self, text: &str, pair: i16) {
        self.label_fixed_width(text, text.len() as i32, pair);
    }

    // end : To ensure that layout closes correctly.
    pub fn end(&mut self) {
        self.layouts
            .pop()
            .expect("Unbalanced Ui::begin() and Ui::end() calls.");
    }
}
