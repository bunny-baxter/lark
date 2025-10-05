mod content;
mod game_model;
mod strings;
mod types;
mod ui_common;

use std::collections::HashMap;

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
use strings::NamedType;
use types::*;
use ui_common::ItemMenu;

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
    game.current_room.create_item(ItemType::LumpOfBlackstone, vec2(4, 1));
    game.current_room.create_item(ItemType::BlackstoneSpear, vec2(5, 1));
    game.current_room.create_item(ItemType::CarmineChainmail, vec2(5, 2));
    game.current_room.create_item(ItemType::Bloodflower, vec2(1, 3));
}

fn create_lines_for_events<'a, 'b, 'c>(events: &'a [GameEvent], type_table: &'b HashMap<u32, NamedType>) -> Vec<Line<'c>> {
    let player_name = "rodney";
    events.iter().map(|event| {
        let color = match event {
            GameEvent::Bonk { .. } => Color::DarkGray,
            GameEvent::MeleeAttack { .. } => Color::Red,
            GameEvent::Death { .. } => Color::DarkGray,
            GameEvent::GotItem { .. } => Color::LightYellow,
            GameEvent::DroppedItem { .. } => Color::LightYellow,
            GameEvent::EquippedItem { .. } => Color::LightYellow,
            GameEvent::UnequippedItem { .. } => Color::LightYellow,
            GameEvent::AteItem { .. } => Color::LightYellow,
            GameEvent::ItemNotEdible { .. } => Color::DarkGray,
            GameEvent::EffectHealed { .. } => Color::LightGreen,
        };
        let parts = vec![
            Span::styled("=> ", Style::default().fg(color)),
            Span::from(strings::get_string(event.clone(), player_name, type_table)),
        ];
        Line::from(parts)
    }).collect()
}

pub struct TerminalApp {
    game: GameInstance,
    unread_event_index: usize,
    item_menu: Option<ItemMenu>,
    exit: bool,
}

impl TerminalApp {
    fn new() -> Self {
        let mut game = GameInstance::new();
        init_test_level(&mut game);
        TerminalApp {
            game,
            unread_event_index: 0,
            item_menu: None,
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

    fn get_char_for_cell(&self, position: TilePoint) -> Span<'_> {
        let actors = self.game.current_room.find_actors_at(position, true);
        if actors.len() > 0 {
            // TODO: Should sort `actors` by which should be on top.
            let actor_index = actors[0];
            let actor = &self.game.current_room.actors[actor_index];
            let mut c = match actor.actor_type {
                ActorType::Player => "@".light_yellow().on_black(),
                ActorType::Toad => "t".light_green().on_black(),
            };
            if actor.is_dead {
                c = c.dark_gray();
            } else if actor.current_hp <= (actor.max_hp as f32 / 4.0).round() as i32 {
                c = c.red();
            } else if actor.current_hp <= (actor.max_hp as f32 / 2.0).round() as i32 {
                c = c.light_red();
            }
            return c;
        }

        let items = self.game.current_room.find_loose_items_at(position);
        if items.len() > 0 {
            let item_index = items[0];
            let item = &self.game.current_room.items[item_index];
            return match item.item_type {
                ItemType::LumpOfBlackstone => "*".gray().on_black(),
                ItemType::BlackstoneSpear => "|".gray().on_black(),
                ItemType::CarmineChainmail => "[".red().on_black(),
                ItemType::Bloodflower => "%".light_red().on_black(),
            };
        }

        let display = display_for_cell_type(self.game.current_room.get_cell_type(position));
        Span::styled(display.c.to_string(), Style::default().fg(display.fg_color).bg(display.bg_color))
    }

    fn build_type_table(&self) -> HashMap<u32, NamedType> {
        let mut result = HashMap::new();
        for actor in self.game.current_room.actors.iter() {
            result.insert(actor.id, NamedType::ActorType { actor_type: actor.actor_type });
        }
        for item in self.game.current_room.items.iter() {
            result.insert(item.id, NamedType::ItemType { item_type: item.item_type });
        }
        result
    }

    fn walk_or_fight(&mut self, delta: TileDelta) {
        let next_position = self.game.current_room.get_player().position + delta;
        let other_actors = self.game.current_room.find_actors_at(next_position, false);
        if other_actors.len() > 0 {
            self.game.execute_command(Command::Fight { delta });
        } else {
            self.game.execute_command(Command::Walk { delta });
        }
    }

    fn get_first_item(&mut self) {
        let position = self.game.current_room.get_player().position;
        let items = self.game.current_room.find_loose_items_at(position);
        if items.len() > 0 {
            let item_id = self.game.current_room.items[items[0]].id;
            self.game.execute_command(Command::GetItem { item_id });
        }
    }

    fn handle_key_main_screen(&mut self, key_code: KeyCode) {
        match key_code {
            KeyCode::Char('q') => self.exit = true,
            KeyCode::Left | KeyCode::Char('h') => self.walk_or_fight(vec2(-1, 0)),
            KeyCode::Right | KeyCode::Char('l') => self.walk_or_fight(vec2(1, 0)),
            KeyCode::Up | KeyCode::Char('k') => self.walk_or_fight(vec2(0, -1)),
            KeyCode::Down | KeyCode::Char('j') => self.walk_or_fight(vec2(0, 1)),
            KeyCode::Char('.') => self.game.execute_command(Command::Wait),
            KeyCode::Char('g') | KeyCode::Char(',') => self.get_first_item(),
            KeyCode::Char('i') => {
                let item_ids = self.game.current_room.player_inventory.clone();
                self.item_menu = Some(ItemMenu::new(item_ids));
            },
            _ => {}
        }
    }

    fn get_selected_item_id(&self) -> Option<u32> {
        let item_menu = self.item_menu.as_ref().unwrap();
        if item_menu.is_empty() {
            return None;
        }
        Some(item_menu.item_ids[item_menu.cursor_index])
    }

    fn handle_key_item_menu(&mut self, key_code: KeyCode) {
        match key_code {
            KeyCode::Char('q') => self.exit = true,
            KeyCode::Up | KeyCode::Char('k') => self.item_menu.as_mut().unwrap().move_cursor(-1),
            KeyCode::Down | KeyCode::Char('j') => self.item_menu.as_mut().unwrap().move_cursor(1),
            KeyCode::Char('d') => if let Some(item_id) = self.get_selected_item_id() {
                self.game.execute_command(Command::DropItem { item_id });
                self.item_menu = None;
            },
            KeyCode::Char('w') => if let Some(item_id) = self.get_selected_item_id() {
                self.game.execute_command(Command::ToggleEquipment { item_id });
                self.item_menu = None;
            },
            KeyCode::Char('e') => if let Some(item_id) = self.get_selected_item_id() {
                self.game.execute_command(Command::EatItem { item_id });
                self.item_menu = None;
            },
            KeyCode::Esc => self.item_menu = None,
            _ => {}
        }
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        if self.item_menu.is_some() {
            self.handle_key_item_menu(key_event.code);
        } else {
            self.handle_key_main_screen(key_event.code);
        }
    }

    fn handle_events(&mut self) -> Result<()> {
        let turn = self.game.turn;
        let event_log_len = self.game.event_log.len();
        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event)
            }
            _ => {}
        };
        if self.game.turn > turn {
            self.unread_event_index = event_log_len;
        }
        Ok(())
    }

    fn render_main_screen(&self, buf: &mut Buffer) {
        let map_block = Block::bordered()
            .padding(Padding::uniform(1))
            .border_type(ratatui::widgets::BorderType::Thick)
            .title(Line::from(" Lark ".bold()).centered());

        let mut lines_vec = vec![];
        for y in 0..(self.game.current_room.size.y as i32) {
            let mut char_vec = vec![];
            for x in 0..(self.game.current_room.size.x as i32) {
                char_vec.push(self.get_char_for_cell(vec2(x, y)));
            }
            lines_vec.push(Line::from(char_vec));
        }
        let map_text = Text::from(lines_vec);
        Paragraph::new(map_text)
            .centered()
            .block(map_block)
            .render(Rect::new(0, 0, 46, 13), buf);
    }

    fn render_item_menu(&self, buf: &mut Buffer, type_table: &HashMap<u32, NamedType>) {
        let menu_block = Block::bordered()
            .padding(Padding::uniform(1))
            .border_type(ratatui::widgets::BorderType::Thick)
            .title(Line::from(" Inventory ".bold()).centered());

        let item_menu = self.item_menu.as_ref().unwrap();
        let mut lines_vec = vec![];
        if item_menu.is_empty() {
            lines_vec.push(Line::from(strings::EMPTY_INVENTORY.white()));
        } else {
            for i in 0..item_menu.item_ids.len() {
                let item_id = item_menu.item_ids[i];
                let s = if self.game.current_room.get_item(item_id).equipped {
                    format!("{} ({})", strings::get_item_name(item_id, type_table), strings::get_equipped_participle(item_id, type_table))
                } else {
                    strings::get_item_name(item_id, type_table).to_string()
                };
                let mut span = Span::from(s);
                if i == item_menu.cursor_index {
                    span = span.black().on_white();
                } else {
                    span = span.white();
                }
                lines_vec.push(Line::from(span));
            }
        }

        Paragraph::new(Text::from(lines_vec))
            .left_aligned()
            .block(menu_block)
            .render(Rect::new(0, 0, 46, 13), buf);
    }
}

impl Widget for &TerminalApp {
    fn render(self, _area: Rect, buf: &mut Buffer) {
        let type_table = self.build_type_table();

        if self.item_menu.is_some() {
            self.render_item_menu(buf, &type_table);
        } else {
            self.render_main_screen(buf);
        }

        let unread_events = &self.game.event_log[self.unread_event_index..];
        if unread_events.len() > 0 {
            let event_block = Block::bordered()
                .padding(Padding::horizontal(1))
                .border_type(ratatui::widgets::BorderType::Thick);
            let lines = create_lines_for_events(&unread_events, &type_table);
            // TODO: Handle the case where we have more than 8 unread lines. Right now this will just truncate them.
            let height = (2 + lines.len().min(8)) as u16;
            Paragraph::new(Text::from(lines))
                .left_aligned()
                .block(event_block)
                .render(Rect::new(0, 13, 46, height), buf);
        }
    }
}

fn main() -> Result<()> {
    color_eyre::install()?;
    let mut terminal = ratatui::init();
    let app_result = TerminalApp::new().run(&mut terminal);
    ratatui::restore();
    app_result
}
