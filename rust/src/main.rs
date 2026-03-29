mod data;
mod game_model;
mod generate;
mod strings;
mod ui_common;

use std::collections::HashMap;
use std::env;

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

use data::{ActorType, CellType, ItemType, MiscEntityType, GameEvent, STEEL_THISTLE_CYCLE_MAX, TilePoint, TileDelta};
use game_model::{Command, GameInstance};
use strings::NamedType;
use ui_common::ItemMenu;

const MAIN_AREA_HEIGHT: u16 = 16;

struct CellDisplay {
    c: char,
    fg_color: Color,
    bg_color: Color,
}

fn display_for_cell_type(cell_type: CellType) -> CellDisplay {
    match cell_type {
        CellType::OutOfBounds | CellType::Empty => CellDisplay { c: ' ', fg_color: Color::Reset, bg_color: Color::Black },
        CellType::DefaultFloor => CellDisplay { c: '.', fg_color: Color::White, bg_color: Color::Black },
        CellType::FloorMoss => CellDisplay { c: ':', fg_color: Color::LightGreen, bg_color: Color::Black },
        CellType::FloorThyme => CellDisplay { c: '"', fg_color: Color::Yellow, bg_color: Color::Black },
        CellType::DefaultWall => CellDisplay { c: '#', fg_color: Color::Black, bg_color: Color::White },
        CellType::RoomExit => CellDisplay { c: 'o', fg_color: Color::White, bg_color: Color::LightBlue },
        CellType::Water => CellDisplay { c: '~', fg_color: Color::Cyan, bg_color: Color::Black },
    }
}

fn init_test_level(game: &mut GameInstance) {
    game.current_room.set_cell(vec2(5, 1), CellType::Water);
    game.current_room.set_cell(vec2(5, 2), CellType::Water);

    game.current_room.set_cell(vec2(5, 3), CellType::FloorThyme);
    game.current_room.set_cell(vec2(6, 1), CellType::FloorThyme);
    game.current_room.set_cell(vec2(6, 2), CellType::FloorThyme);

    game.current_room.set_cell(vec2(2, 4), CellType::FloorMoss);
    game.current_room.set_cell(vec2(2, 5), CellType::FloorMoss);
    game.current_room.set_cell(vec2(3, 4), CellType::FloorMoss);
    game.current_room.set_cell(vec2(3, 5), CellType::FloorMoss);
    game.current_room.set_cell(vec2(3, 6), CellType::FloorMoss);

    game.current_room.create_player(vec2(2, 1));
    game.current_room.create_item(ItemType::FeatheredCavalier, vec2(1, 2));
    game.current_room.create_item(ItemType::CarmineHelm, vec2(2, 2));
    game.current_room.create_item(ItemType::Bloodflower, vec2(1, 3));
    game.current_room.create_item(ItemType::Bloodflower, vec2(2, 3));
    game.current_room.create_item(ItemType::Bloodflower, vec2(3, 3));
    game.current_room.create_item(ItemType::Bloodflower, vec2(4, 3));
    game.current_room.create_item(ItemType::Bloodflower, vec2(5, 3));
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
            GameEvent::SlowedByWater { .. } => Color::Cyan,
            GameEvent::ActivatedItem { .. } => Color::LightYellow,
            GameEvent::EffectIceDamage { .. } => Color::Red,
            GameEvent::NoEffect { .. } => Color::DarkGray,
            GameEvent::SteelThistleHit { .. } => Color::Red,
            GameEvent::ThrownStoneDamage { .. } => Color::Red,
            GameEvent::JavelinDamage { .. } => Color::Red,
            GameEvent::WandExpended { .. } => Color::DarkGray,
            GameEvent::ItemIsHere { .. } => Color::Yellow,
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
    direction_selection_item: Option<u32>,
    exit: bool,
}

impl TerminalApp {
    fn new(use_test_level: bool) -> Self {
        let mut game = GameInstance::new();
        if use_test_level {
            init_test_level(&mut game);
        } else {
            game.create_first_room();
        }

        TerminalApp {
            game,
            unread_event_index: 0,
            item_menu: None,
            direction_selection_item: None,
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
        let visible = self.game.current_room.visible.contains(&position);
        let explored = self.game.current_room.explored.contains(&position);
        if !visible && !explored {
            return Span::from(" ");
        }

        if visible {
            let mut actors = self.game.current_room.find_actors_at(position, true);
            if actors.len() > 0 {
                // Sort so alive actors are displayed above dead actors
                actors.sort_by_key(|&index| self.game.current_room.actors[index].is_dead);
                let actor_index = actors[0];
                let actor = &self.game.current_room.actors[actor_index];
                let mut c = match actor.actor_type {
                    ActorType::Player => "@".light_yellow().on_black(),
                    ActorType::Toad => "t".light_green().on_black(),
                    ActorType::MouseWarrior => "m".light_cyan().on_black(),
                    ActorType::MouseSkirmisher => "m".magenta().on_black(),
                    ActorType::ToothyStarling => "s".cyan().on_black(),
                    ActorType::DustySkeleton => "z".white().on_black(),
                    ActorType::BlueJelly => "j".light_blue().on_black(),
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
                    ItemType::CarmineSword => "\\".red().on_black(),
                    ItemType::MoonlightKnife => "-".white().on_black(),
                    ItemType::BoneLamellar => "[".white().on_black(),
                    ItemType::FeatheredCavalier => "^".yellow().on_black(),
                    ItemType::CarmineHelm => "^".red().on_black(),
                    ItemType::CarmineChainmail => "[".red().on_black(),
                    ItemType::Bloodflower => "%".light_red().on_black(),
                    ItemType::WandOfIce => "/".light_cyan().on_black(),
                };
            }

            let misc_entities = self.game.current_room.find_misc_entities_at(position);
            if misc_entities.len() > 0 {
                let entity_index = misc_entities[0];
                let entity = &self.game.current_room.misc_entities[entity_index];
                const STEEL_THISTLE_CYCLE_MAX_MINUS_1: i32 = STEEL_THISTLE_CYCLE_MAX - 1;
                return match entity.entity_type {
                    MiscEntityType::SteelThistle => match entity.data {
                        0..STEEL_THISTLE_CYCLE_MAX_MINUS_1 => "+".white().on_black(),
                        STEEL_THISTLE_CYCLE_MAX_MINUS_1 => "+".light_magenta().on_black(),
                        STEEL_THISTLE_CYCLE_MAX => "%".light_magenta().on_black(),
                        _ => unreachable!(),
                    },
                    MiscEntityType::TreasureChest => "=".yellow().on_black(),
                };
            }
        }

        let display = display_for_cell_type(self.game.current_room.get_cell_type(position));
        let mut fg_color = display.fg_color;
        let mut bg_color = display.bg_color;
        if !visible {
            if display.bg_color == Color::Black {
                fg_color = Color::DarkGray;
            } else {
                bg_color = Color::DarkGray;
            }
        }
        Span::styled(display.c.to_string(), Style::default().fg(fg_color).bg(bg_color))
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
            KeyCode::Char('v') | KeyCode::Char('t') => if let Some(item_id) = self.get_selected_item_id() {
                self.direction_selection_item = Some(item_id);
                self.item_menu = None;
            },
            KeyCode::Esc => self.item_menu = None,
            _ => {}
        }
    }

    fn handle_key_direction_selection(&mut self, key_code: KeyCode) {
        match key_code {
            KeyCode::Char('q') => self.exit = true,
            KeyCode::Left | KeyCode::Char('h') => {
                self.game.execute_command(Command::ActivateItemByDirection { item_id: self.direction_selection_item.unwrap(), direction: vec2(-1, 0) });
                self.direction_selection_item = None;
            },
            KeyCode::Right | KeyCode::Char('l') => {
                self.game.execute_command(Command::ActivateItemByDirection { item_id: self.direction_selection_item.unwrap(), direction: vec2(1, 0) });
                self.direction_selection_item = None;
            },
            KeyCode::Up | KeyCode::Char('k') => {
                self.game.execute_command(Command::ActivateItemByDirection { item_id: self.direction_selection_item.unwrap(), direction: vec2(0, -1) });
                self.direction_selection_item = None;
            },
            KeyCode::Down | KeyCode::Char('j') => {
                self.game.execute_command(Command::ActivateItemByDirection { item_id: self.direction_selection_item.unwrap(), direction: vec2(0, 1) });
                self.direction_selection_item = None;
            },
            KeyCode::Esc => self.direction_selection_item = None,
            _ => {}
        }
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        if self.item_menu.is_some() {
            self.handle_key_item_menu(key_event.code);
        } else if self.direction_selection_item.is_some() {
            self.handle_key_direction_selection(key_event.code);
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
            .render(Rect::new(0, 0, 48, MAIN_AREA_HEIGHT), buf);

        let side_hud_block = Block::new()
            .padding(Padding::symmetric(2, 1));
        let player_ref = self.game.current_room.get_player();
        let side_hud_lines = vec![
            Line::from(format!("Depth {}", self.game.current_room.depth + 1)),
            Line::from(""),
            Line::from(format!("Health {}/{}", player_ref.current_hp, player_ref.max_hp)),
            Line::from(format!("Attack {}", player_ref.attack_power)),
            Line::from(format!("Defense {}", player_ref.defense_power)),
        ];
        Paragraph::new(Text::from(side_hud_lines))
            .block(side_hud_block)
            .render(Rect::new(48, 0, 16, MAIN_AREA_HEIGHT), buf);
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
            .render(Rect::new(0, 0, 64, MAIN_AREA_HEIGHT), buf);
    }
}

impl Widget for &TerminalApp {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let type_table = self.build_type_table();

        if self.item_menu.is_some() {
            self.render_item_menu(buf, &type_table);
        } else {
            self.render_main_screen(buf);
        }

        let lines = if self.direction_selection_item.is_some() {
            let parts = vec![
                "?> ".light_yellow(),
                strings::DIRECTION_SELECTION_PROMPT.white(),
            ];
            vec![ Line::from(parts) ]
        } else {
            let unread_events = &self.game.event_log[self.unread_event_index..];
            if unread_events.len() > 0 {
                create_lines_for_events(&unread_events, &type_table)
            } else {
                vec![]
            }
        };
        if lines.len() > 0 {
            let event_block = Block::bordered()
                .padding(Padding::horizontal(1))
                .border_type(ratatui::widgets::BorderType::Thick);
            // TODO: Handle the case where we have more than 7 unread lines. Right now this will just truncate them.
            let height = (2 + lines.len().min(7)) as u16;
            Paragraph::new(Text::from(lines))
                .left_aligned()
                .block(event_block)
                .render(Rect::new(0, MAIN_AREA_HEIGHT, 64, height), buf);
        }

        let reminder_y = area.height - 2;
        if self.item_menu.is_some() {
            Line::from("arrow keys = select, 'd' = drop, 'w' = wear/wield, 'e' = eat,".dark_gray())
                .render(Rect::new(0, reminder_y, 64, 1), buf);
            Line::from("'v'/'t' = evoke/throw ".dark_gray())
                .render(Rect::new(0, reminder_y + 1, 64, 1), buf);
        } else {
            Line::from("arrow keys = move, '.' = wait, 'g' = pick up, 'i' = inventory ".dark_gray())
                .render(Rect::new(0, reminder_y, 64, 1), buf);
        }
    }
}

fn main() -> Result<()> {
    color_eyre::install()?;
    let use_test_level = env::args().any(|arg| arg == "--test-level");
    let mut app = TerminalApp::new(use_test_level);
    let mut terminal = ratatui::init();
    let app_result = app.run(&mut terminal);
    ratatui::restore();
    app_result
}
