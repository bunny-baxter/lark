mod game_model;
mod types;

use color_eyre::Result;
use cgmath::vec2;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{Block, Paragraph, Widget},
    widgets::block::Padding,
    DefaultTerminal, Frame,
};

use game_model::{CellType, Command, GameInstance};
use types::*;

struct CellDisplay {
    c: char,
    fg_color: Color,
    bg_color: Color,
}

fn display_for_cell_type(cell_type: CellType) -> CellDisplay {
    match cell_type {
        CellType::OutOfBounds | CellType::Empty => CellDisplay { c: ' ', fg_color: Color::Reset, bg_color: Color::Black },
        CellType::Floor => CellDisplay { c: '.', fg_color: Color::White, bg_color: Color::Black },
        CellType::DefaultWall => CellDisplay { c: '#', fg_color: Color::Black, bg_color: Color::White },
    }
}

fn init_test_level(game: &mut GameInstance) {
    game.current_room.create_player(vec2(2, 1));
    game.current_room.create_actor(ActorType::Toad, vec2(4, 4));
}

pub struct TerminalApp {
    game: GameInstance,
    exit: bool,
}

impl TerminalApp {
    fn new() -> Self {
        let mut game = GameInstance::new();
        init_test_level(&mut game);
        TerminalApp {
            game,
            exit: false,
        }
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('q') => self.exit = true,
            KeyCode::Left => self.game.execute_command(Command::Walk { delta: vec2(-1, 0) }),
            KeyCode::Right => self.game.execute_command(Command::Walk { delta: vec2(1, 0) }),
            KeyCode::Up => self.game.execute_command(Command::Walk { delta: vec2(0, -1) }),
            KeyCode::Down => self.game.execute_command(Command::Walk { delta: vec2(0, 1) }),
            KeyCode::Char('.') => self.game.execute_command(Command::Wait),
            _ => {}
        }
    }

    fn handle_events(&mut self) -> Result<()> {
        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event)
            }
            _ => {}
        };
        Ok(())
    }
}

impl Widget for &TerminalApp {
    fn render(self, _area: Rect, buf: &mut Buffer) {
        let title = Line::from(" Lark ".bold());
        let block = Block::bordered()
            .padding(Padding::uniform(1))
            .border_type(ratatui::widgets::BorderType::Thick)
            .title(title.centered());

        let mut lines_vec = vec![];
        for y in 0..(self.game.current_room.size.y as i32) {
            let mut char_vec = vec![];
            for x in 0..(self.game.current_room.size.x as i32) {
                let actors = self.game.current_room.find_actors_at(vec2(x, y));
                if actors.len() > 0 {
                    let actor_index = actors[0];
                    let c = match self.game.current_room.actors[actor_index].actor_type {
                        ActorType::Player => "@".light_yellow().on_black(),
                        ActorType::Toad => "t".light_green().on_black(),
                    };
                    char_vec.push(c);
                } else {
                    let display = display_for_cell_type(self.game.current_room.get_cell_type(vec2(x, y)));
                    char_vec.push(Span::styled(display.c.to_string(), Style::default().fg(display.fg_color).bg(display.bg_color)));
                }
            }
            lines_vec.push(Line::from(char_vec));
        }
        let text = Text::from(lines_vec);
        Paragraph::new(text)
            .centered()
            .block(block)
            .render(Rect::new(0, 0, 23, 13), buf);
    }
}

fn main() -> Result<()> {
    color_eyre::install()?;
    let mut terminal = ratatui::init();
    let app_result = TerminalApp::new().run(&mut terminal);
    ratatui::restore();
    app_result
}
