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

use crate::model::{modules::Module, outpost::Outpost};

pub struct App {
    pub outpost: Outpost,
    pub palette: Flavour,

    pub state: Vec<State>,
}

#[derive(Clone)]
pub enum State {
    GameMenu,
    // module states
    Outpost(usize),
    // crew states
    Crew(usize),
    AssignToModule(usize, usize),
    // research states
    Research,
    // region states
    Region,
}

impl State {
    fn transitions(&self, app: &App) -> Vec<StateTransition> {
        use DomainEvent::*;
        use State::*;
        use StateTransition::*;
        match *self {
            GameMenu => vec![PopState(KeyCode::Esc), QuitAndSave(KeyCode::Char('q'))],
            Outpost(i) => vec![
                PushState(KeyCode::Esc, GameMenu),
                ReplaceState(KeyCode::Tab, Crew(0)),
                ReplaceState(KeyCode::BackTab, Region),
                ReplaceState(
                    KeyCode::Char('j'),
                    Outpost(circular_index((i as i32) + 1, &app.outpost.modules)),
                ),
                ReplaceState(
                    KeyCode::Char('k'),
                    Outpost(circular_index((i as i32) - 1, &app.outpost.modules)),
                ),
                ApplyDomainEvent(KeyCode::Char('+'), IncrementModuleEnergyLevel, false),
                ApplyDomainEvent(KeyCode::Char('-'), DecrementModuleEnergyLevel, false),
            ],
            Crew(i) => vec![
                PushState(KeyCode::Esc, GameMenu),
                ReplaceState(KeyCode::Tab, Research),
                ReplaceState(KeyCode::BackTab, Outpost(0)),
                ReplaceState(
                    KeyCode::Char('j'),
                    Crew(circular_index((i as i32) + 1, &app.outpost.crew)),
                ),
                ReplaceState(
                    KeyCode::Char('k'),
                    Crew(circular_index((i as i32) - 1, &app.outpost.crew)),
                ),
                PushState(KeyCode::Char('a'), AssignToModule(i, 0)),
            ],
            Research => vec![
                PushState(KeyCode::Esc, GameMenu),
                ReplaceState(KeyCode::Tab, Region),
                ReplaceState(KeyCode::BackTab, Crew(0)),
            ],
            Region => vec![
                PushState(KeyCode::Esc, GameMenu),
                ReplaceState(KeyCode::Tab, Outpost(0)),
                ReplaceState(KeyCode::BackTab, Research),
            ],
            AssignToModule(c, m) => vec![
                PopState(KeyCode::Esc),
                ReplaceState(
                    KeyCode::Char('j'),
                    AssignToModule(c, circular_index((m as i32) + 1, &app.outpost.modules)),
                ),
                ReplaceState(
                    KeyCode::Char('k'),
                    AssignToModule(c, circular_index((m as i32) - 1, &app.outpost.modules)),
                ),
                ApplyDomainEvent(KeyCode::Enter, AssignCrewMemberToModule, true),
            ],
        }
    }
}

impl std::string::ToString for State {
    fn to_string(&self) -> String {
        use State::*;
        match *self {
            GameMenu => String::from("Game Menu"),
            Outpost(_) => String::from("Outpost"),
            Crew(_) => String::from("Crew"),
            Region => String::from("Region"),
            Research => String::from("Research"),
            AssignToModule(_, _) => String::from("Assign Crew Member to Module"),
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
}

impl App {
    pub fn new() -> App {
        let input_path = "./saves/current.json";

        let outpost: Outpost = std::fs::File::open(input_path)
            .ok()
            .and_then(|data| serde_json::from_reader(data).ok())
            .unwrap_or_else(Outpost::new);

        App {
            outpost,
            palette: Flavour::Mocha,
            state: vec![State::Outpost(0)],
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
        return match transition {
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
                    let data = serde_json::to_string(&self.outpost).unwrap();
                    let output_path = "./saves/current.json";
                    let _ = std::fs::create_dir_all("./saves");
                    let _ = std::fs::write(output_path, data);
                    Some(Ok(()))
                }
                ApplyDomainEvent(_, e, and_pop) => {
                    match e {
                        IncrementModuleEnergyLevel => {
                            self.current_module().map(|m| m.increment_energy_level());
                        }
                        DecrementModuleEnergyLevel => {
                            self.current_module().map(|m| m.decrement_energy_level());
                        }
                        AssignCrewMemberToModule => match self.current_state() {
                            AssignToModule(c, m) => {
                                self.outpost.assign_crew_member_to_module(*c, *m)
                            }
                            _ => (),
                        },
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
        };
    }

    pub fn render<B: Backend>(&self, f: &mut Frame<B>) {
        let window = f.size();
        let block = Block::default().style(
            Style::default()
                .bg(to_color(self.palette.base()))
                .fg(to_color(self.palette.text())),
        );
        f.render_widget(block, window);

        let outer_layout = Layout::default()
            .direction(Direction::Vertical)
            .margin(5)
            .constraints([Constraint::Length(1), Constraint::Min(0)].as_ref())
            .split(window);

        let inner_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(
                [
                    Constraint::Percentage(20),
                    Constraint::Min(0),
                    Constraint::Percentage(20),
                ]
                .as_ref(),
            )
            .split(outer_layout[1]);

        let left_pane = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
            .split(inner_layout[0]);

        let right_pane = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                    Constraint::Percentage(60),
                    Constraint::Percentage(20),
                    Constraint::Percentage(20),
                ]
                .as_ref(),
            )
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

    fn current_module(&mut self) -> Option<&mut Box<dyn Module>> {
        match self.state.last().unwrap() {
            State::Outpost(i) => Some(&mut self.outpost.modules[*i]),
            _ => None,
        }
    }

    fn header<B: Backend>(&self, f: &mut Frame<B>, area: Rect) {
        let consumption = self.outpost.consumption();
        let production = self.outpost.production();
        let text = vec![Spans::from(vec![
            Span::styled(
                format!(
                    "{}/{}",
                    consumption.energy.to_string(),
                    self.outpost.resources.energy.to_string(),
                ),
                Style::default().fg(to_color(self.palette.yellow())),
            ),
            Span::raw(" | "),
            Span::styled(
                format!(
                    "{}/{}",
                    consumption.living_space.to_string(),
                    self.outpost.resources.living_space.to_string(),
                ),
                Style::default().fg(to_color(self.palette.peach())),
            ),
            Span::raw(" | "),
            Span::styled(
                format!(
                    "{}({})",
                    self.outpost.resources.minerals.to_string(),
                    print_i32(production.minerals - consumption.minerals),
                ),
                Style::default().fg(to_color(self.palette.sapphire())),
            ),
            Span::raw(" | "),
            Span::styled(
                format!(
                    "{}({})",
                    self.outpost.resources.food.to_string(),
                    print_i32(production.food - consumption.food),
                ),
                Style::default().fg(to_color(self.palette.green())),
            ),
            Span::raw(" | "),
            Span::styled(
                format!(
                    "{}({})",
                    self.outpost.resources.water.to_string(),
                    print_i32(production.water - consumption.water),
                ),
                Style::default().fg(to_color(self.palette.blue())),
            ),
        ])];

        f.render_widget(
            Paragraph::new(text)
                .alignment(Alignment::Center)
                .wrap(Wrap { trim: true }),
            area,
        )
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
            State::Outpost(s) => {
                state.select(Some(*s));
                focused = true
            }
            _ => (),
        };
        self.modules_list(f, area, "Outpost", &mut state, focused)
    }

    fn modules_list_assign_to_module<B: Backend>(&self, f: &mut Frame<B>, area: Rect) {
        let mut state: ListState = ListState::default();
        let mut focused = false;
        match self.current_state() {
            State::AssignToModule(_, m) => {
                state.select(Some(*m));
                focused = true
            }
            _ => (),
        };
        self.modules_list(f, area, "Assign To Module", &mut state, focused)
    }

    fn modules_list<B: Backend>(
        &self,
        f: &mut Frame<B>,
        area: Rect,
        title: &str,
        state: &mut ListState,
        focused: bool,
    ) {
        let modules: Vec<ListItem> = self
            .outpost
            .modules
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
                .block(self.border(title, focused))
                .highlight_style(
                    Style::default()
                        .add_modifier(Modifier::BOLD)
                        .bg(to_color(self.palette.overlay0())),
                )
                .highlight_symbol("> "),
            area,
            state,
        )
    }

    fn crew_list<B: Backend>(&self, f: &mut Frame<B>, area: Rect) {
        let crew: Vec<ListItem> = self
            .outpost
            .crew
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
                .block(self.border(&String::from("Crew"), focused))
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
            State::Region => focused = true,
            _ => (),
        };
        f.render_widget(self.border(&String::from("Current Mission"), focused), area)
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
        f.render_widget(
            self.border(&String::from("Current Research"), focused),
            area,
        )
    }

    fn focus<B: Backend>(&self, f: &mut Frame<B>, area: Rect) {
        match self.current_state() {
            State::Crew(i) => {
                let crew = &self.outpost.crew[*i];
                let description = self.outpost.describe_crew_member(&crew);

                let mood = print_percentage(description.mood);
                let biology = print_percentage(description.stats.biology);
                let chemistry = print_percentage(description.stats.chemistry);
                let engineering = print_percentage(description.stats.engineering);
                let geology = print_percentage(description.stats.geology);
                let astrophysics = print_percentage(description.stats.astrophysics);
                let military = print_percentage(description.stats.military);

                let data: Vec<Vec<&str>> = vec![
                    vec!["Mood", &mood],
                    vec!["Biology", &biology],
                    vec!["Chemistry", &chemistry],
                    vec!["Engineering", &engineering],
                    vec!["Geology", &geology],
                    vec!["Astrophysics", &astrophysics],
                    vec!["Military", &military],
                ];
                let rows = data
                    .iter()
                    .map(|row| Row::new(row.iter().map(|c| Cell::from(*c))));
                f.render_widget(
                    Table::new(rows)
                        .block(self.border("Stats", false))
                        .widths(&[Constraint::Percentage(70), Constraint::Percentage(30)]),
                    area,
                )
            }
            State::Outpost(i) => {
                let module = &self.outpost.modules[*i];

                f.render_widget(self.border(module.name(), false), area);

                let description = self.outpost.describe_module(&module);

                let energy_consumption = description.consumption.energy.to_string();
                let energy_production = description.production.energy.to_string();
                let living_space_consumption = description.consumption.living_space.to_string();
                let living_space_production = description.production.living_space.to_string();
                let minerals_consumption = description.consumption.minerals.to_string();
                let minerals_production = description.production.minerals.to_string();
                let food_consumption = description.consumption.food.to_string();
                let food_production = description.production.food.to_string();
                let water_consumption = description.consumption.water.to_string();
                let water_production = description.production.water.to_string();

                let header_data = vec!["Resource", "In", "Out"];
                let header_cells = header_data.iter().map(|h| {
                    Cell::from(*h).style(Style::default().fg(to_color(self.palette.subtext0())))
                });
                let header = Row::new(header_cells);

                let mut data: Vec<Vec<&str>> = vec![];

                if description.consumption.energy != 0 || description.production.energy != 0 {
                    data.push(vec!["Energy", &energy_consumption, &energy_production])
                }
                if description.consumption.living_space != 0
                    || description.production.living_space != 0
                {
                    data.push(vec![
                        "Living Space",
                        &living_space_consumption,
                        &living_space_production,
                    ])
                }
                if description.consumption.minerals != 0 || description.production.minerals != 0 {
                    data.push(vec![
                        "Minerals",
                        &minerals_consumption,
                        &minerals_production,
                    ])
                }
                if description.consumption.food != 0 || description.production.food != 0 {
                    data.push(vec!["Food", &food_consumption, &food_production])
                }
                if description.consumption.water != 0 || description.production.water != 0 {
                    data.push(vec!["Water", &water_consumption, &water_production])
                }

                let rows = data
                    .iter()
                    .map(|row| Row::new(row.iter().map(|c| Cell::from(*c))));

                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints(
                        [
                            Constraint::Length((data.len() + 3) as u16),
                            Constraint::Min(0),
                        ]
                        .as_ref(),
                    )
                    .split(area);

                f.render_widget(
                    Table::new(rows)
                        .header(header)
                        .block(self.border("Production", false))
                        .widths(&[
                            Constraint::Percentage(70),
                            Constraint::Percentage(15),
                            Constraint::Percentage(15),
                        ]),
                    chunks[0],
                );

                let header_data = vec!["Active", "In", "Out"];
                let header_cells = header_data.iter().map(|h| {
                    Cell::from(*h).style(Style::default().fg(to_color(self.palette.subtext0())))
                });
                let header = Row::new(header_cells);

                let data: Vec<Vec<&str>> = description
                    .energy_levels
                    .iter()
                    .map(|l| {
                        let is_active = if l.is_active { "X" } else { "" };
                        vec![is_active, "TODO", "TODO"]
                    })
                    .collect();

                let rows = data
                    .iter()
                    .map(|row| Row::new(row.iter().map(|c| Cell::from(*c))));
                f.render_widget(
                    Table::new(rows)
                        .header(header)
                        .block(self.border("Energy Levels", false))
                        .widths(&[
                            Constraint::Percentage(70),
                            Constraint::Percentage(15),
                            Constraint::Percentage(15),
                        ]),
                    chunks[1],
                )
            }
            State::GameMenu => {
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
            State::AssignToModule(_, _) => self.modules_list_assign_to_module(f, area),
            State::Research => {
                f.render_widget(self.border(&self.current_state().to_string(), false), area)
            }
            State::Region => {
                f.render_widget(self.border(&self.current_state().to_string(), false), area)
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
fn circular_index<T>(index: i32, arr: &Vec<T>) -> usize {
    (((index % arr.len() as i32) + arr.len() as i32) % arr.len() as i32) as usize
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
