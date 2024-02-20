use anyhow::Ok;

use crossterm::{cursor, event::{self, read, Event}, style::{Color, PrintStyledContent, Stylize}, ExecutableCommand, QueueableCommand};
use crossterm::style::Print;
use crossterm::terminal::{self, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen};

use std::io::{stdout, Write};
enum Action {
    Quit,
    MoveUp,
    MoveDown,
    MoveLeft,
    MoveRight,
    EnterMode(Mode),
    InsertChar(char),
}

#[derive(Debug)]
enum Mode {
    Normal,
    Insert,
}

pub struct Editor {
    size: (u16, u16),
    cx: usize,
    cy: usize,
    mode: Mode,
    stdout: std::io::Stdout,
}


impl Editor {
    pub fn new() -> anyhow::Result<Self> {

        terminal::enable_raw_mode().unwrap();
        let mut stdout = stdout();
        stdout
            .execute(EnterAlternateScreen)?
            .execute(Clear(ClearType::All))?;

        Ok(Editor{
            size: terminal::size()?,
            cx: 0,
            cy: 0,
            mode: Mode::Normal,
            stdout: stdout,
        })
    }
    pub fn run(&mut self) -> anyhow::Result<()> {
      
        loop {
            self.draw_status_line()?;
            self.stdout.queue(cursor::MoveTo(self.cx as u16, self.cy as u16))?;
            self.stdout.flush()?;
            
            if let Some(action) = self.handle_event(read()?)? {
                match action {
                    Action::Quit => break,
                    Action::MoveUp => self.cy = self.cy.saturating_sub(1),
                    Action::MoveDown => self.cy += 1,
                    Action::MoveLeft => self.cx = self.cx.saturating_sub(1),
                    Action::MoveRight => self.cx += 1,
                    Action::InsertChar(c) => {
                        self.stdout.queue(Print(c))?;
                        self.cx += 1;
                    }
                    Action::EnterMode(new_mode) => self.mode = new_mode,
                }
            }
        }
        self.stdout.execute(LeaveAlternateScreen)?;
        terminal::disable_raw_mode()?;
        Ok(())
    }
    fn draw_status_line(&mut self) -> anyhow::Result<()>{
        let mode = format!(" {:?} ", self.mode).to_uppercase();
        let file = "src/main.rs";
        let pos = format!(" {}:{} ", self.cx, self.cy);

        let file_width = self.size.0 - mode.len() as u16 - pos.len() as u16 - 2;

        self.stdout.queue(cursor::MoveTo(0, self.size.1 - 2))?;
        self.stdout.queue(PrintStyledContent(
            mode.with(Color::Rgb { r: 0, g: 0, b: 0 })
                .bold()
                .on(Color::Rgb {
                    r: 184,
                    g: 144,
                    b: 243,
                }),
        ))?;
        self.stdout.queue(PrintStyledContent(
            ""
                .with(Color::Rgb {
                    r: 184,
                    g: 144,
                    b: 243,
                })
                .on(Color::Rgb {
                    r: 67,
                    g: 70,
                    b: 89,
                }),
        ))?;
        self.stdout.queue(PrintStyledContent(
            format!("{:width$}", file, width = file_width as usize)
                .with(Color::Rgb {
                    r: 255,
                    g: 255,
                    b: 255,
                })
                .bold()
                .on(Color::Rgb {
                    r: 67,
                    g: 70,
                    b: 89,
                }),
        ))?;
        self.stdout.queue(PrintStyledContent(
            ""
                .with(Color::Rgb {
                    r: 184,
                    g: 144,
                    b: 243,
                })
                .on(Color::Rgb {
                    r: 67,
                    g: 70,
                    b: 89,
                }),
        ))?;
        self.stdout.queue(PrintStyledContent(
            pos.with(Color::Rgb { r: 0, g: 0, b: 0 })
                .bold()
                .on(Color::Rgb {
                    r: 184,
                    g: 144,
                    b: 243,
                }),
        ))?;
        
        Ok(())
    }
    fn handle_event(&self, event: Event) -> anyhow::Result<Option<Action>>{
        if matches!(self.mode, Mode::Normal) {
            self.handle_normal_event(event)
        }
        else {
            self.handle_insert_event(event)
        }
    }
    fn handle_normal_event (&self,event: event::Event) -> anyhow::Result<Option<Action>>{
        match event {
            event::Event::Key(event) => {
                if matches!(event.kind, event::KeyEventKind::Release) {
                        match event.code {
                            event::KeyCode::Char('q') => Ok(Some(Action::Quit)),
                            event::KeyCode::Up | event::KeyCode::Char('k') => Ok(Some(Action::MoveUp)),
                            event::KeyCode::Down | event::KeyCode::Char('j') => Ok(Some(Action::MoveDown)),
                            event::KeyCode::Left | event::KeyCode::Char('h') => Ok(Some(Action::MoveLeft)),
                            event::KeyCode::Right | event::KeyCode::Char('l') => Ok(Some(Action::MoveRight)),
                            event::KeyCode::Char('i') => Ok(Some(Action::EnterMode(Mode::Insert))),
                            _ => Ok(None),
                    }
                }
                else {
                    Ok(None)
                }
            },
            _ => Ok(None),
        }
    }
    
    fn handle_insert_event (&self, event: Event) -> anyhow::Result<Option<Action>>{
        match event {
            event::Event::Key(event) =>
                if matches!(event.kind, event::KeyEventKind::Release) {
                    match event.code {
                        event::KeyCode::Esc => Ok(Some(Action::EnterMode(Mode::Normal))),
                        event::KeyCode::Char(c) => Ok(Some(Action::InsertChar(c))),
                        _ => Ok(None),
                    }
                }
                else {
                    Ok(None)
                },
            _ => Ok(None),
        }
    }
}