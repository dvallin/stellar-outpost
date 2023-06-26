use catppuccin::{Colour, Flavour};
use crossterm::event::KeyCode;
use std::io;
use tui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Cell, List, ListItem, ListState, Paragraph, Row, Table, Wrap},
    Frame,
};

use crate::model::{
    crew::CrewMember, modules::Module, resources::Resources, sector::SectorType, Game,
};

pub struct App {
    pub game: Game,
    pub palette: Flavour,

    pub state: Vec<State>,
}

#[derive(Clone)]
pub enum State {
    GameMenu,
    Overview,
    // module states
    Modules(usize),
    AssignCrew(usize, usize),
    // crew states
    Crew(usize),
    AssignToModule(usize, usize),
    // research states
    Research,
    // sector states
    Sector(i32, i32),
    SelectMission(i32, i32, usize),
}

impl State {
    fn transitions(&self, app: &App) -> Vec<StateTransition> {
        use DomainEvent::*;
        use KeyCode::*;
        use State::*;
        use StateTransition::*;
        match *self {
            GameMenu => vec![PopState(Esc), QuitAndSave(Char('q'))],
            Overview => vec![
                PushState(Esc, GameMenu),
                PushState(Char('c'), Crew(0)),
                PushState(Char('m'), Modules(0)),
                PushState(Char('r'), Research),
                PushState(Char('s'), Sector(0, 0)),
                ApplyDomainEvent(Enter, FinishTurn, false),
            ],
            Modules(i) => vec![
                PopState(Esc),
                ReplaceState(Char('c'), Crew(0)),
                ReplaceState(Char('r'), Research),
                ReplaceState(Char('s'), Sector(0, 0)),
                ReplaceState(Tab, Crew(0)),
                ReplaceState(BackTab, Sector(0, 0)),
                ReplaceState(
                    Char('j'),
                    Modules(circular_index(
                        (i as i32) + 1,
                        app.game.outpost.modules_len(),
                    )),
                ),
                ReplaceState(
                    Char('k'),
                    Modules(circular_index(
                        (i as i32) - 1,
                        app.game.outpost.modules_len(),
                    )),
                ),
                ApplyDomainEvent(Char('+'), IncrementModuleEnergyLevel, false),
                ApplyDomainEvent(Char('-'), DecrementModuleEnergyLevel, false),
                PushState(Char('a'), AssignCrew(0, i)),
            ],
            Crew(i) => vec![
                PopState(Esc),
                ReplaceState(Char('m'), Modules(0)),
                ReplaceState(Char('r'), Research),
                ReplaceState(Char('s'), Sector(0, 0)),
                ReplaceState(Tab, Research),
                ReplaceState(BackTab, Modules(0)),
                ReplaceState(
                    Char('j'),
                    Crew(circular_index((i as i32) + 1, app.game.outpost.crew_len())),
                ),
                ReplaceState(
                    Char('k'),
                    Crew(circular_index((i as i32) - 1, app.game.outpost.crew_len())),
                ),
                PushState(Char('a'), AssignToModule(i, 0)),
            ],
            Research => vec![
                PopState(Esc),
                ReplaceState(Char('c'), Crew(0)),
                ReplaceState(Char('m'), Modules(0)),
                ReplaceState(Char('s'), Sector(0, 0)),
                ReplaceState(Tab, Sector(0, 0)),
                ReplaceState(BackTab, Crew(0)),
            ],
            Sector(x, y) => vec![
                PopState(Esc),
                ReplaceState(Char('c'), Crew(0)),
                ReplaceState(Char('m'), Modules(0)),
                ReplaceState(Char('r'), Research),
                ReplaceState(Tab, Modules(0)),
                ReplaceState(BackTab, Research),
                ReplaceState(
                    Char('h'),
                    Sector(clamp(x - 1, app.game.sector.bounds_at_y(y)), y),
                ),
                ReplaceState(
                    Char('l'),
                    Sector(clamp(x + 1, app.game.sector.bounds_at_y(y)), y),
                ),
                ReplaceState(
                    Char('j'),
                    Sector(x, clamp(y + 1, app.game.sector.bounds_at_x(x))),
                ),
                ReplaceState(
                    Char('k'),
                    Sector(x, clamp(y - 1, app.game.sector.bounds_at_x(x))),
                ),
                PushState(Enter, SelectMission(x, y, 0)),
            ],
            SelectMission(x, y, m) => vec![
                PopState(Esc),
                ReplaceState(
                    Char('j'),
                    SelectMission(
                        x,
                        y,
                        circular_index((m as i32) + 1, app.game.sector.missions_at(x, y).len()),
                    ),
                ),
                ReplaceState(
                    Char('k'),
                    SelectMission(
                        x,
                        y,
                        circular_index((m as i32) - 1, app.game.sector.missions_at(x, y).len()),
                    ),
                ),
            ],
            AssignToModule(c, m) => vec![
                PopState(Esc),
                ReplaceState(
                    Char('j'),
                    AssignToModule(
                        c,
                        circular_index((m as i32) + 1, app.game.outpost.modules_len()),
                    ),
                ),
                ReplaceState(
                    Char('k'),
                    AssignToModule(
                        c,
                        circular_index((m as i32) - 1, app.game.outpost.modules_len()),
                    ),
                ),
                ApplyDomainEvent(Enter, AssignCrewMemberToModule, true),
            ],
            AssignCrew(c, m) => vec![
                PopState(Esc),
                ReplaceState(
                    Char('j'),
                    AssignCrew(
                        circular_index((c as i32) + 1, app.game.outpost.crew_len()),
                        m,
                    ),
                ),
                ReplaceState(
                    Char('k'),
                    AssignCrew(
                        circular_index((c as i32) - 1, app.game.outpost.crew_len()),
                        m,
                    ),
                ),
                ApplyDomainEvent(Enter, AssignCrewMemberToModule, true),
            ],
        }
    }
}

impl std::string::ToString for State {
    fn to_string(&self) -> String {
        use State::*;
        match *self {
            GameMenu => String::from("Game Menu"),
            Overview => String::from("Outpost"),
            Modules(_) => String::from("Modules"),
            Crew(_) => String::from("Crew"),
            Sector(_, _) => String::from("Sector"),
            SelectMission(_, _, _) => String::from("Select Mission"),
            Research => String::from("Research"),
            AssignToModule(_, _) | AssignCrew(_, _) => String::from("Assign Crew Member to Module"),
        }
    }
}

#[derive(Clone)]
enum StateTransition {
    PushState(KeyCode, State),
    PopState(KeyCode),
    ReplaceState(KeyCode, State),
    QuitAndSave(KeyCode),
    ApplyDomainEvent(KeyCode, DomainEvent, bool),
}

#[derive(Clone)]
enum DomainEvent {
    IncrementModuleEnergyLevel,
    DecrementModuleEnergyLevel,
    AssignCrewMemberToModule,
    FinishTurn,
}

impl App {
    pub fn new() -> App {
        let input_path = "./saves/current.json";

        let game: Game = std::fs::File::open(input_path)
            .ok()
            .and_then(|data| serde_json::from_reader(data).ok())
            .unwrap_or_else(Game::new);

        App {
            game,
            palette: Flavour::Mocha,
            state: vec![State::Overview],
        }
    }

    pub fn input(&mut self, code: KeyCode) -> Option<io::Result<()>> {
        use DomainEvent::*;
        use State::*;
        use StateTransition::*;
        let transitions = self.current_state().transitions(self);
        let transition = transitions.iter().find(|t| match t {
            PopState(c)
            | QuitAndSave(c)
            | PushState(c, _)
            | ReplaceState(c, _)
            | ApplyDomainEvent(c, _, _) => c.eq(&code),
        });
        match transition {
            Some(transition) => match transition {
                PopState(_) => {
                    self.state.pop();
                    if self.state.is_empty() {
                        Some(Ok(()))
                    } else {
                        None
                    }
                }
                PushState(_, s) => {
                    self.state.push(s.clone());
                    None
                }
                ReplaceState(_, s) => {
                    self.state.pop();
                    self.state.push(s.clone());
                    None
                }
                QuitAndSave(_) => {
                    let data = serde_json::to_string(&self.game).unwrap();
                    let output_path = "./saves/current.json";
                    let _ = std::fs::create_dir_all("./saves");
                    let _ = std::fs::write(output_path, data);
                    Some(Ok(()))
                }
                ApplyDomainEvent(_, e, and_pop) => {
                    match e {
                        IncrementModuleEnergyLevel => match self.current_state() {
                            State::Modules(i) => {
                                self.game.increment_energy_level(*i);
                            }
                            _ => (),
                        },
                        DecrementModuleEnergyLevel => match self.current_state() {
                            State::Modules(i) => {
                                self.game.decrement_energy_level(*i);
                            }
                            _ => (),
                        },
                        AssignCrewMemberToModule => match self.current_state() {
                            AssignToModule(c, m) | AssignCrew(c, m) => {
                                self.game.assign_crew_member_to_module(*c, *m);
                            }
                            _ => (),
                        },
                        FinishTurn => {
                            self.game.finish_turn();
                        }
                    }
                    if *and_pop {
                        self.state.pop();
                        if self.state.is_empty() {
                            Some(Ok(()))
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                }
            },
            None => None,
        }
    }

    pub fn render<B: Backend>(&self, f: &mut Frame<B>) {
        use Constraint::*;
        use Direction::*;

        let window = f.size();
        let block = Block::default().style(
            Style::default()
                .bg(to_color(self.palette.base()))
                .fg(to_color(self.palette.text())),
        );
        f.render_widget(block, window);

        let outer_layout = Layout::default()
            .direction(Vertical)
            .margin(5)
            .constraints([Length(1), Min(0)].as_ref())
            .split(window);

        let inner_layout = Layout::default()
            .direction(Horizontal)
            .constraints([Percentage(20), Min(0), Percentage(20)].as_ref())
            .split(outer_layout[1]);

        let left_pane = Layout::default()
            .direction(Vertical)
            .constraints([Percentage(50), Percentage(50)].as_ref())
            .split(inner_layout[0]);

        let right_pane = Layout::default()
            .direction(Vertical)
            .constraints([Percentage(60), Percentage(20), Percentage(20)].as_ref())
            .split(inner_layout[2]);

        self.header(f, outer_layout[0]);

        self.modules_list_tab(f, left_pane[0]);
        self.crew_list(f, left_pane[1]);

        self.logs(f, right_pane[0]);
        self.research_summary(f, right_pane[1]);
        self.mission_summary(f, right_pane[2]);

        self.focus(f, inner_layout[1]);
    }

    fn current_state(&self) -> &State {
        self.state.last().unwrap()
    }

    fn header<B: Backend>(&self, f: &mut Frame<B>, area: Rect) {
        let consumption = self.game.outpost.consumption() + self.game.outpost.crew_upkeep();
        let production = self.game.outpost.production();
        let resources = self.game.outpost.resources();
        let text = vec![Spans::from(vec![
            Span::styled(
                format!("turn {}", self.game.state.current_turn),
                Style::default().fg(to_color(self.palette.text())),
            ),
            Span::raw(" | "),
            Span::styled(
                format!(
                    "{}/{}",
                    consumption.energy.to_string(),
                    production.energy.to_string(),
                ),
                Style::default().fg(to_color(self.palette.yellow())),
            ),
            Span::raw(" | "),
            Span::styled(
                format!(
                    "{}/{}",
                    consumption.living_space.to_string(),
                    production.living_space.to_string(),
                ),
                Style::default().fg(to_color(self.palette.peach())),
            ),
            Span::raw(" | "),
            Span::styled(
                format!(
                    "{}({})",
                    resources.minerals.to_string(),
                    print_i32(production.minerals - consumption.minerals),
                ),
                Style::default().fg(to_color(self.palette.sapphire())),
            ),
            Span::raw(" | "),
            Span::styled(
                format!(
                    "{}({})",
                    resources.food.to_string(),
                    print_i32(production.food - consumption.food),
                ),
                Style::default().fg(to_color(self.palette.green())),
            ),
            Span::raw(" | "),
            Span::styled(
                format!(
                    "{}({})",
                    resources.water.to_string(),
                    print_i32(production.water - consumption.water),
                ),
                Style::default().fg(to_color(self.palette.blue())),
            ),
        ])];

        f.render_widget(Paragraph::new(text).alignment(Alignment::Center), area)
    }

    fn border<'a>(&self, title: &'a str, focused: bool) -> Block<'a> {
        let fg: Colour = if focused {
            self.palette.lavender()
        } else {
            self.palette.overlay0()
        };
        Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(to_color(fg)))
    }

    fn modules_list_tab<B: Backend>(&self, f: &mut Frame<B>, area: Rect) {
        let mut state: ListState = ListState::default();
        let mut focused = false;
        match self.current_state() {
            State::Modules(s) => {
                state.select(Some(*s));
                focused = true
            }
            _ => (),
        };

        let modules: Vec<ListItem> = self
            .game
            .outpost
            .modules()
            .iter()
            .map(|m| {
                ListItem::new(Spans::from(vec![Span::styled(
                    m.name(),
                    Style::default().fg(to_color(self.palette.text())),
                )]))
            })
            .collect();

        f.render_stateful_widget(
            List::new(modules)
                .block(self.border("Modules (m)", focused))
                .highlight_style(
                    Style::default()
                        .add_modifier(Modifier::BOLD)
                        .bg(to_color(self.palette.overlay0())),
                )
                .highlight_symbol("> "),
            area,
            &mut state,
        )
    }

    fn modules_list_assign_to_module<B: Backend>(
        &self,
        f: &mut Frame<B>,
        crew: &CrewMember,
        area: Rect,
    ) {
        let mut state: ListState = ListState::default();
        let mut focused = false;
        match self.current_state() {
            State::AssignToModule(_, m) => {
                state.select(Some(*m));
                focused = true
            }
            _ => (),
        };

        let modules: Vec<ListItem> = self
            .game
            .outpost
            .modules()
            .iter()
            .map(|m| {
                let mut line = vec![Span::styled(
                    m.name(),
                    Style::default().fg(to_color(self.palette.text())),
                )];
                line.append(&mut self.resource_string(&m.production_bonus(crew)));
                ListItem::new(Spans::from(line))
            })
            .collect();

        f.render_stateful_widget(
            List::new(modules)
                .block(self.border("Assign To Module", focused))
                .highlight_style(
                    Style::default()
                        .add_modifier(Modifier::BOLD)
                        .bg(to_color(self.palette.overlay0())),
                )
                .highlight_symbol("> "),
            area,
            &mut state,
        )
    }

    fn crew_list_assign_to_module<B: Backend>(
        &self,
        f: &mut Frame<B>,
        module: &Box<dyn Module>,
        area: Rect,
    ) {
        let mut state: ListState = ListState::default();
        let mut focused = false;
        match self.current_state() {
            State::AssignCrew(c, _) => {
                state.select(Some(*c));
                focused = true
            }
            _ => (),
        };

        let crew: Vec<ListItem> = self
            .game
            .outpost
            .crew()
            .iter()
            .map(|c| {
                let mut line = vec![Span::styled(
                    c.name(),
                    Style::default().fg(to_color(self.palette.text())),
                )];
                line.append(&mut self.resource_string(&module.production_bonus(c)));
                ListItem::new(Spans::from(line))
            })
            .collect();

        f.render_stateful_widget(
            List::new(crew)
                .block(self.border("Assign To Module", focused))
                .highlight_style(
                    Style::default()
                        .add_modifier(Modifier::BOLD)
                        .bg(to_color(self.palette.overlay0())),
                )
                .highlight_symbol("> "),
            area,
            &mut state,
        )
    }

    fn crew_list<B: Backend>(&self, f: &mut Frame<B>, area: Rect) {
        let crew: Vec<ListItem> = self
            .game
            .outpost
            .crew()
            .iter()
            .map(|m| {
                ListItem::new(Spans::from(vec![Span::styled(
                    m.name(),
                    Style::default().fg(to_color(self.palette.text())),
                )]))
            })
            .collect();

        let mut state: ListState = ListState::default();
        let mut focused = false;
        match self.current_state() {
            State::Crew(s) | State::AssignToModule(s, _) => {
                state.select(Some(*s));
                focused = true
            }
            _ => (),
        };

        f.render_stateful_widget(
            List::new(crew)
                .block(self.border(&String::from("Crew (c)"), focused))
                .highlight_style(
                    Style::default()
                        .add_modifier(Modifier::BOLD)
                        .bg(to_color(self.palette.overlay0())),
                )
                .highlight_symbol("> "),
            area,
            &mut state,
        )
    }

    fn mission_summary<B: Backend>(&self, f: &mut Frame<B>, area: Rect) {
        let mut focused = false;
        match self.current_state() {
            State::Sector(_, _) => focused = true,
            _ => (),
        };
        f.render_widget(self.border(&String::from("Sector Map (s)"), focused), area)
    }

    fn logs<B: Backend>(&self, f: &mut Frame<B>, area: Rect) {
        f.render_widget(self.border(&String::from("Logs"), false), area)
    }

    fn research_summary<B: Backend>(&self, f: &mut Frame<B>, area: Rect) {
        let mut focused = false;
        match self.current_state() {
            State::Research => focused = true,
            _ => (),
        };
        f.render_widget(self.border(&String::from("Research (r)"), focused), area)
    }

    fn resource_string(&self, resource: &Resources) -> Vec<Span> {
        let mut result = vec![];
        if resource.energy != 0 {
            result.push(Span::styled(
                format!("{}e ", print_i32(resource.energy)),
                Style::default().fg(to_color(self.palette.yellow())),
            ))
        }
        if resource.living_space != 0 {
            result.push(Span::styled(
                format!("{}l ", print_i32(resource.living_space)),
                Style::default().fg(to_color(self.palette.peach())),
            ))
        }
        if resource.minerals != 0 {
            result.push(Span::styled(
                format!("{}m ", print_i32(resource.minerals)),
                Style::default().fg(to_color(self.palette.sapphire())),
            ))
        }
        if resource.food != 0 {
            result.push(Span::styled(
                format!("{}f ", print_i32(resource.food)),
                Style::default().fg(to_color(self.palette.green())),
            ))
        }
        if resource.water != 0 {
            result.push(Span::styled(
                format!("{}w ", print_i32(resource.water)),
                Style::default().fg(to_color(self.palette.blue())),
            ))
        }
        result
    }

    fn focus<B: Backend>(&self, f: &mut Frame<B>, area: Rect) {
        use Constraint::*;
        use Direction::*;
        use State::*;
        match self.current_state() {
            Overview => {
                f.render_widget(self.border(&self.current_state().to_string(), false), area)
            }
            Crew(i) => {
                if self.game.outpost.crew_len() <= *i {
                    return;
                }

                let crew_id = self.game.outpost.crew_member_id_by_index(*i);
                let crew_member = self.game.outpost.get_crew_member(&crew_id);
                let description = self.game.outpost.describe_crew_member(&crew_member);

                let health = vec![
                    Span::raw("health: "),
                    Span::raw(print_percentage(crew_member.health())),
                ];
                let mood = vec![
                    Span::raw("mood: "),
                    Span::raw(print_i32(crew_member.mood())),
                ];
                let mut upkeep = vec![Span::raw("upkeep: ")];
                upkeep.append(&mut self.resource_string(&description.upkeep));

                let mut assignment = vec![Span::raw(format!(
                    "assignment: {} ",
                    description.assigned_module_name()
                ))];
                assignment.append(&mut self.resource_string(&description.flow()));

                let biography = Span::raw("Lorem ipsum dolor sit amet, consetetur sadipscing elitr, sed diam nonumy eirmod tempor invidunt ut labore et dolore magna aliquyam erat, sed diam voluptua. At vero eos et accusam et justo duo dolores et ea rebum. Stet clita kasd gubergren, no sea takimata sanctus est Lorem ipsum dolor sit amet. Lorem ipsum dolor sit amet, consetetur sadipscing elitr, sed diam nonumy eirmod tempor invidunt ut labore et dolore magna aliquyam erat, sed diam voluptua. At vero eos et accusam et justo duo dolores et ea rebum. Stet clita kasd gubergren, no sea takimata sanctus est Lorem ipsum dolor sit amet.");

                let chunks = Layout::default()
                    .direction(Vertical)
                    .constraints([Length(6), Length(8), Min(0)].as_ref())
                    .split(area);

                let biology = print_percentage(description.stats.biology);
                let chemistry = print_percentage(description.stats.chemistry);
                let engineering = print_percentage(description.stats.engineering);
                let geology = print_percentage(description.stats.geology);
                let astrophysics = print_percentage(description.stats.astrophysics);
                let military = print_percentage(description.stats.military);

                let data: Vec<Vec<&str>> = vec![
                    vec!["biology", &biology],
                    vec!["chemistry", &chemistry],
                    vec!["engineering", &engineering],
                    vec!["geology", &geology],
                    vec!["astrophysics", &astrophysics],
                    vec!["military", &military],
                ];
                let rows = data
                    .iter()
                    .map(|row| Row::new(row.iter().map(|c| Cell::from(*c))));

                f.render_widget(
                    Paragraph::new(vec![
                        Spans::from(health),
                        Spans::from(mood),
                        Spans::from(upkeep),
                        Spans::from(assignment),
                    ])
                    .block(self.border(description.name, false))
                    .wrap(Wrap { trim: true }),
                    chunks[0],
                );
                f.render_widget(
                    Table::new(rows)
                        .block(self.border("Stats", false))
                        .widths(&[Constraint::Percentage(70), Constraint::Percentage(30)]),
                    chunks[1],
                );
                f.render_widget(
                    Paragraph::new(vec![Spans::from(biography)])
                        .block(self.border("Biography", false))
                        .wrap(Wrap { trim: true }),
                    chunks[2],
                );
            }
            Modules(i) => {
                if self.game.outpost.modules_len() <= *i {
                    return;
                }

                let module_id = self.game.outpost.module_id_by_index(*i);
                let module = self.game.outpost.get_module(&module_id);
                let description = self.game.outpost.describe_module(module);

                let flow = description.production - description.consumption;

                let mut resource_flow = vec![Span::raw("resource flow: ")];
                resource_flow.append(&mut self.resource_string(&flow));

                let slots = description
                    .energy_levels
                    .iter()
                    .filter(|&l| l.is_active)
                    .count();
                let assigned_slots = description
                    .energy_levels
                    .iter()
                    .filter(|&l| l.is_active && l.assignment.is_some())
                    .count();
                let energy_level = Span::raw(format!(
                    "energy level: {}/{}",
                    slots,
                    description.energy_levels.len()
                ));
                let assigned_slots = Span::styled(
                    format!("assigned slots: {}/{}", assigned_slots, slots),
                    Style::default().fg(to_color(if assigned_slots > 0 {
                        self.palette.text()
                    } else {
                        self.palette.red()
                    })),
                );

                let chunks = Layout::default()
                    .direction(Vertical)
                    .constraints([Length(5), Min(0)].as_ref())
                    .split(area);

                let energy_level_chunks = Layout::default()
                    .direction(Vertical)
                    .constraints(vec![Length(3); slots])
                    .split(chunks[1]);

                f.render_widget(
                    Paragraph::new(vec![
                        Spans::from(resource_flow),
                        Spans::from(energy_level),
                        Spans::from(assigned_slots),
                    ])
                    .block(self.border(description.name, false)),
                    chunks[0],
                );
                for (i, d) in description
                    .energy_levels
                    .iter()
                    .filter(|&l| l.is_active)
                    .enumerate()
                {
                    let mut level_description = self.resource_string(&d.flow());
                    level_description.push(Span::raw(" ("));
                    level_description.push(Span::raw(d.assigned_crew_name()));
                    level_description.push(Span::raw(")"));

                    f.render_widget(
                        Paragraph::new(Spans::from(level_description)).block(
                            Block::default()
                                .title(format!("level {}", i))
                                .borders(Borders::TOP)
                                .border_style(
                                    Style::default().fg(to_color(self.palette.overlay0())),
                                ),
                        ),
                        energy_level_chunks[i],
                    )
                }
            }
            GameMenu => {
                let header_data = vec!["Action", "Key"];
                let data: Vec<Vec<&str>> = vec![
                    vec!["go back (or to game menu)", "Esc"],
                    vec!["Quit (in game menu)", "q"],
                    vec!["next pane", "Tab"],
                    vec!["previous pane", "Shift+Tab"],
                    vec!["up (e.g. in lists)", "k"],
                    vec!["down (e.g. in lists)", "j"],
                    vec!["increment energy", "+"],
                    vec!["decrement energy", "-"],
                    vec!["assign to module", "a"],
                ];

                let header_cells = header_data.iter().map(|h| {
                    Cell::from(*h).style(Style::default().fg(to_color(self.palette.subtext0())))
                });
                let header = Row::new(header_cells).height(1).bottom_margin(1);
                let rows = data
                    .iter()
                    .map(|row| Row::new(row.iter().map(|c| Cell::from(*c))));
                f.render_widget(
                    Table::new(rows)
                        .header(header)
                        .block(self.border(&String::from("Game Menu"), false))
                        .widths(&[Constraint::Percentage(70), Constraint::Percentage(30)]),
                    area,
                )
            }
            AssignToModule(c, _) => {
                let crew_id = self.game.outpost.crew_member_id_by_index(*c);
                let crew_member: &CrewMember = self.game.outpost.get_crew_member(&crew_id);
                self.modules_list_assign_to_module(f, crew_member, area);
            }
            AssignCrew(_, m) => {
                let module_id = self.game.outpost.module_id_by_index(*m);
                let module = self.game.outpost.get_module(&module_id);
                self.crew_list_assign_to_module(f, module, area);
            }
            Research => {
                f.render_widget(self.border(&self.current_state().to_string(), false), area)
            }
            Sector(x, y) => {
                use SectorType::*;
                let mut state: ListState = ListState::default();
                let mut regions: Vec<ListItem> = vec![];
                for (coordinates, sector) in self.game.sector.sub_sectors_map() {
                    if coordinates.x == *x && coordinates.y == *y {
                        state.select(Some(regions.len()));
                    }
                    let sector_name = match sector.sector_type {
                        EmptySpace => "empty space",
                        SolarSystem => "solar system",
                        GasCloud => "gas cloud",
                        StellarRift => "stellar rift",
                    };
                    regions.push(ListItem::new(Spans::from(vec![Span::styled(
                        format!("({}, {}) {}", coordinates.x, coordinates.y, sector_name),
                        Style::default().fg(to_color(self.palette.text())),
                    )])))
                }
                let chunks = Layout::default()
                    .direction(Vertical)
                    .constraints(vec![Percentage(80), Percentage(20)])
                    .split(area);

                f.render_stateful_widget(
                    List::new(regions)
                        .block(self.border(&String::from("Sector Map"), true))
                        .highlight_style(
                            Style::default()
                                .add_modifier(Modifier::BOLD)
                                .bg(to_color(self.palette.overlay0())),
                        )
                        .highlight_symbol("> "),
                    chunks[0],
                    &mut state,
                );

                let missions: Vec<ListItem> = self
                    .game
                    .sector
                    .missions_at(*x, *y)
                    .iter()
                    .map(|_m| {
                        ListItem::new(Spans::from(vec![Span::styled(
                            "Mining Mission",
                            Style::default().fg(to_color(self.palette.text())),
                        )]))
                    })
                    .collect();

                f.render_widget(
                    List::new(missions).block(self.border(&String::from("Missions"), false)),
                    chunks[1],
                );
            }
            SelectMission(x, y, m) => {
                use SectorType::*;
                let mut map_state: ListState = ListState::default();
                let mut regions: Vec<ListItem> = vec![];
                for (coordinates, sector) in &self.game.sector.sub_sectors_map() {
                    if coordinates.x == *x && coordinates.y == *y {
                        map_state.select(Some(regions.len()));
                    }
                    let sector_name = match sector.sector_type {
                        EmptySpace => "empty space",
                        SolarSystem => "solar system",
                        GasCloud => "gas cloud",
                        StellarRift => "stellar rift",
                    };
                    regions.push(ListItem::new(Spans::from(vec![Span::styled(
                        format!("({}, {}) {}", coordinates.x, coordinates.y, sector_name),
                        Style::default().fg(to_color(self.palette.text())),
                    )])))
                }
                let chunks = Layout::default()
                    .direction(Vertical)
                    .constraints(vec![Percentage(80), Percentage(20)])
                    .split(area);

                f.render_stateful_widget(
                    List::new(regions)
                        .block(self.border(&String::from("Sector Map"), false))
                        .highlight_style(Style::default().add_modifier(Modifier::BOLD))
                        .highlight_symbol("> "),
                    chunks[0],
                    &mut map_state,
                );

                let missions: Vec<ListItem> = self
                    .game
                    .sector
                    .missions_at(*x, *y)
                    .iter()
                    .map(|_m| {
                        ListItem::new(Spans::from(vec![Span::styled(
                            "Mining Mission",
                            Style::default().fg(to_color(self.palette.text())),
                        )]))
                    })
                    .collect();

                let mut mission_state: ListState = ListState::default();
                mission_state.select(Some(*m));

                f.render_stateful_widget(
                    List::new(missions)
                        .block(self.border(&String::from("Missions"), true))
                        .highlight_style(
                            Style::default()
                                .add_modifier(Modifier::BOLD)
                                .bg(to_color(self.palette.overlay0())),
                        )
                        .highlight_symbol("> "),
                    chunks[1],
                    &mut mission_state,
                );
            }
        }
    }
}

fn print_percentage(v: i32) -> String {
    format!("{}%", v)
}
fn print_i32(v: i32) -> String {
    if v >= 0 {
        format!("+{}", v)
    } else {
        v.to_string()
    }
}
fn circular_index(index: i32, len: usize) -> usize {
    if len == 0 {
        0
    } else {
        (((index % len as i32) + len as i32) % len as i32) as usize
    }
}
fn clamp(index: i32, bounds: (i32, i32)) -> i32 {
    std::cmp::min(std::cmp::max(index, bounds.0), bounds.1)
}

fn to_color(value: Colour) -> Color {
    let (r, g, b) = value.into();
    Color::Rgb(r, g, b)
}

#[cfg(test)]
mod tests {
    use super::circular_index;

    #[test]
    fn calculate_circular_index() {
        let assert_index = |expected: usize, index: i32, arr: &Vec<i32>| {
            assert_eq!(
                expected,
                circular_index(index, arr),
                "indexing {} should create circular index {}",
                index,
                expected
            );
        };
        let arr = vec![1, 2, 3];
        assert_index(0, 0, &arr);

        // positive
        assert_index(1, 1, &arr);
        assert_index(2, 2, &arr);
        assert_index(0, 3, &arr);
        assert_index(1, 4, &arr);
        assert_index(2, 5, &arr);
        assert_index(0, 6, &arr);
        // negative
        assert_index(2, -1, &arr);
        assert_index(1, -2, &arr);
        assert_index(0, -3, &arr);
        assert_index(2, -4, &arr);
    }
}
