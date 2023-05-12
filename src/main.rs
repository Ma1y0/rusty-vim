use std::io::{self, Read, stdout, Write};
use std::time::Duration;
use crossterm::{ event, terminal, execute, cursor, queue };
use crossterm::event::{ Event, KeyCode, KeyEvent };

struct CleanUp;

impl Drop for CleanUp {
    fn drop(&mut self) {
        terminal::disable_raw_mode().expect("Couldn't switch back to canonical mode");
        Output::clear_screeen().expect("Error");
    }
}

struct Output {
    win_size: (usize, usize),
    editor_contents: EditorContents,
    cursor_controller: CursorController
}

impl Output {
    fn new() -> Self {
        let win_size = terminal::size()
            .map(|(x, y)| (x as usize, y as usize))
            .unwrap();
        Self { 
            win_size,
            editor_contents: EditorContents::new(),
            cursor_controller: CursorController::new()
        }
    }

    fn clear_screeen() -> crossterm::Result<()> {
        execute!(stdout(), terminal::Clear(terminal::ClearType::All))?;
        execute!(stdout(), cursor::MoveTo(0, 0))
    }

    fn move_cursor(&mut self, direction: char) {
       self.cursor_controller.move_cursor(direction); 
    }

    fn draw_rows(&mut self) {
        let screen_rows = self.win_size.1;
        let screen_columns = self.win_size.0;
        for i in 0..screen_rows {
            if i == screen_rows / 3 {
                let mut welcome = format!("Rusty vim --- Version 0.1.1");
                if welcome.len() > screen_columns {
                    welcome.truncate(screen_columns)
                }
                let mut padding = (screen_columns - welcome.len()) / 2;
                if padding != 0 {
                    self.editor_contents.push('~');
                    padding -= 1
                }
                (0..padding).for_each(|_| self.editor_contents.push(' '));
                self.editor_contents.push_str(&welcome);
            } else {
                self.editor_contents.push('~');
            }
            queue!(
                self.editor_contents,
                terminal::Clear(terminal::ClearType::UntilNewLine)
                )
                .unwrap();
            if i < screen_rows - 1 {
                self.editor_contents.push_str("\r\n");
            }
        }
    }

    fn refresh_screen(&mut self) -> crossterm::Result<()> {
        queue!(self.editor_contents, cursor::MoveTo(0, 0), cursor::Hide)?;
        self.draw_rows();
        let cursor_x = self.cursor_controller.cursor_x;
        let cursor_y = self.cursor_controller.cursor_y;
        queue!(
            self.editor_contents,
            cursor::MoveTo(cursor_x as u16, cursor_y as u16),
            cursor::Show
        )?;
        self.editor_contents.flush()
    }
}

struct CursorController {
    cursor_x: usize,
    cursor_y: usize
}

impl CursorController {
    fn new() -> CursorController {
        Self { cursor_x: 0, cursor_y: 0 }
    }

    fn move_cursor(&mut self, direction: char) {
        match direction {
            'j' => {
                self.cursor_y += 1;
            }
            'h' => {
                self.cursor_x -= 1;
            }
            'k' => {
                self.cursor_y -= 1;
            }
            'l' => {
                self.cursor_x += 1;
            }
            _ => unimplemented!(),
        }
    }

}

struct Reader;

impl Reader {
    fn read_key(&self) -> crossterm::Result<KeyEvent> {
        loop {
            if event::poll(Duration::from_millis(5000))? {
                if let Event::Key(event) = event::read()? {
                    return Ok(event);
                }
            }
        }
    }
}

struct EditorContents {
    content: String
}

impl EditorContents {
    fn new() -> Self {
        Self { content: String::new() }
    }

    fn push(&mut self, ch: char) {
        self.content.push(ch);
    }

    fn push_str(&mut self, string: &str) {
        self.content.push_str(string);
    }
}

impl io::Write for EditorContents {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match std::str::from_utf8(buf) {
            Ok(s) => {
                self.content.push_str(s);
                Ok(s.len())
            }
            Err(_) => Err(io::ErrorKind::WriteZero.into())
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        let out = write!(stdout(), "{}", self.content);
        stdout().flush()?;
        self.content.clear();
        out
    }
}

struct Editor {
    reader: Reader,
    output: Output
}

impl Editor {
    fn new() -> Self {
        Self {
            reader: Reader,
            output: Output::new()
        }
    }

    fn process_keypress(&mut self) -> crossterm::Result<bool> {
        match self.reader.read_key()? {
            KeyEvent {
                code: KeyCode::Char('q'),
                modifiers: event::KeyModifiers::CONTROL,
                ..
            } => return Ok(false),
            /* note the following*/
            KeyEvent {
                code: KeyCode::Char(val),
                modifiers: event::KeyModifiers::NONE,
                ..
            }  => {
                match val {
                    'h'| 'j'|'k'|'l' => self.output.move_cursor(val),
                    _=> {/*do nothing*/}
                }
                
            },
            // end
            _ => {}
        }
        Ok(true)
    }

    fn run(&mut self) -> crossterm::Result<bool> {
        self.output.refresh_screen()?;
        self.process_keypress()
    }
}

fn main() -> crossterm::Result<()> {
    let _clean_up = CleanUp;
    terminal::enable_raw_mode()?;

    let mut editor = Editor::new();
    while editor.run()? {}
    Ok(())
}
