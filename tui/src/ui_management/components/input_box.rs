use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    prelude::{Backend, Rect},
    style::{Color, Style, Stylize},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use tokio::sync::mpsc::UnboundedSender;
use unicode_width::UnicodeWidthChar;

use crate::state_store::{action::Action, State};

use super::{Component, ComponentRender};

pub struct InputBox {
    /// Current value of the input box
    text: String,
    /// Position of cursor in the editor area.
    cursor_position: usize,
}

impl InputBox {
    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn set_text(&mut self, new_text: &str) {
        self.text = String::from(new_text);
        self.cursor_position = self.get_max_cursor_position();
    }

    pub fn reset(&mut self) {
        self.cursor_position = 0;
        self.text.clear();
    }

    pub fn is_empty(&self) -> bool {
        self.text.is_empty()
    }

    /// Gets the maximum cursor position (number of characters). 
    fn get_max_cursor_position(&self) -> usize {
        // Note: text.len() returns the number of bytes, not the number of characters.
        self.text.char_indices().count()
    }

    fn move_cursor_left(&mut self) {
        let cursor_moved_left = self.cursor_position.saturating_sub(1);
        self.cursor_position = self.clamp_cursor(cursor_moved_left);
    }

    fn move_cursor_right(&mut self) {
        let cursor_moved_right = self.cursor_position.saturating_add(1);
        self.cursor_position = self.clamp_cursor(cursor_moved_right);
    }

    fn enter_char(&mut self, new_char: char) {
        self.text.insert(self.get_cursor_byte_index(), new_char);

        self.move_cursor_right();
    }

    /// Gets the current cursor's byte index. Since the string is in UTF-8 format, character index != byte index.
    fn get_cursor_byte_index(&self) -> usize {
        self.text.char_indices().nth(self.cursor_position).map(|(i, _)| i).unwrap_or(self.text.len()) 
    }

    fn delete_char(&mut self) {
        let is_not_cursor_leftmost = self.cursor_position != 0;
        if is_not_cursor_leftmost {
            // Method "remove" is not used on the saved text for deleting the selected char.
            // Reason: Using remove on String works on bytes instead of the chars.
            // Using remove would require special care because of char boundaries.

            let current_index = self.cursor_position;
            let from_left_to_current_index = current_index - 1;

            // Getting all characters before the selected character.
            let before_char_to_delete = self.text.chars().take(from_left_to_current_index);
            // Getting all characters after selected character.
            let after_char_to_delete = self.text.chars().skip(current_index);

            // Put all characters together except the selected one.
            // By leaving the selected one out, it is forgotten and therefore deleted.
            self.text = before_char_to_delete.chain(after_char_to_delete).collect();
            self.move_cursor_left();
        }
    }

    fn clamp_cursor(&self, new_cursor_pos: usize) -> usize {
        new_cursor_pos.clamp(0, self.get_max_cursor_position())
    }

    /// Calculates the cursor offset on the terminal, taking into account character width to handle wide characters (e.g., double-byte characters).
    fn get_cursor_offset_in_terminal(&self) -> u16 {
        let mut cursor_offset = 0;
        for (i, c) in self.text.chars().enumerate() {
            if i == self.cursor_position {
                break;
            }
            // Add character width
            cursor_offset += c.width().unwrap_or(0) as u16;
        }
        cursor_offset
    }
}

impl Component for InputBox {
    fn new(_state: &State, _action_tx: UnboundedSender<Action>) -> Self {
        Self {
            //
            text: String::new(),
            cursor_position: 0,
        }
    }

    fn move_with_state(self, _state: &State) -> Self
    where
        Self: Sized,
    {
        Self { ..self }
    }

    fn name(&self) -> &str {
        "Input Box"
    }

    fn handle_key_event(&mut self, key: KeyEvent) {
        if key.kind != KeyEventKind::Press {
            return;
        }

        match key.code {
            KeyCode::Char(to_insert) => {
                self.enter_char(to_insert);
            }
            KeyCode::Backspace => {
                self.delete_char();
            }
            KeyCode::Left => {
                self.move_cursor_left();
            }
            KeyCode::Right => {
                self.move_cursor_right();
            }
            _ => {}
        }
    }
}

pub struct RenderProps {
    pub title: String,
    pub area: Rect,
    pub border_color: Color,
    pub show_cursor: bool,
}

impl ComponentRender<RenderProps> for InputBox {
    fn render<B: Backend>(&self, frame: &mut Frame<B>, props: RenderProps) {
        let input = Paragraph::new(self.text.as_str())
            .style(Style::default().fg(Color::Yellow))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .fg(props.border_color)
                    .title(props.title),
            );
        frame.render_widget(input, props.area);

        // Cursor is hidden by default, so we need to make it visible if the input box is selected
        if props.show_cursor {
            // Make the cursor visible and ask ratatui to put it at the specified coordinates after
            // rendering
            frame.set_cursor(
                // Draw the cursor at the current position in the input field.
                // This position is can be controlled via the left and right arrow key
                props.area.x + self.get_cursor_offset_in_terminal() as u16 + 1,
                // Move one line down, from the border to the input line
                props.area.y + 1,
            )
        }
    }
}
