use anyhow::Ok;

use crossterm::{cursor, event::{self, read, Event}, ExecutableCommand, QueueableCommand};
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

enum Mode {
    Normal,
    Insert,
}

pub struct Editor {
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
            cx: 0,
            cy: 0,
            mode: Mode::Normal,
            stdout: stdout,
        })
    }
  
    pub fn run(&mut self) -> anyhow::Result<()> {
      
        loop {
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