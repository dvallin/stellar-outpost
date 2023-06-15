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

use crate::model::{crew::CrewMember, modules::Module, outpost::Outpost, resources::Resources};

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
    AssignCrew(usize, usize),
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
        use KeyCode::*;
        use State::*;
        use StateTransition::*;
        match *self {
            GameMenu => vec![PopState(Esc), QuitAndSave(Char('q'))],
            Outpost(i) => vec![
                PushState(Esc, GameMenu),
                ReplaceState(Tab, Crew(0)),
                ReplaceState(BackTab, Region),
                ReplaceState(
                    Char('j'),
                    Outpost(circular_index((i as i32) + 1, &app.outpost.modules)),
                ),
                ReplaceState(
                    Char('k'),
                    Outpost(circular_index((i as i32) - 1, &app.outpost.modules)),
                ),
                ApplyDomainEvent(Char('+'), IncrementModuleEnergyLevel, false),
                ApplyDomainEvent(Char('-'), DecrementModuleEnergyLevel, false),
                PushState(Char('a'), AssignCrew(0, i)),
                ApplyDomainEvent(Enter, FinishTurn, false),
            ],
            Crew(i) => vec![
                PushState(Esc, GameMenu),
                ReplaceState(Tab, Research),
                ReplaceState(BackTab, Outpost(0)),
                ReplaceState(
                    Char('j'),
                    Crew(circular_index((i as i32) + 1, &app.outpost.crew)),
                ),
                ReplaceState(
                    Char('k'),
                    Crew(circular_index((i as i32) - 1, &app.outpost.crew)),
                ),
                PushState(Char('a'), AssignToModule(i, 0)),
                ApplyDomainEvent(Enter, FinishTurn, false),
            ],
            Research => vec![
                PushState(Esc, GameMenu),
                ReplaceState(Tab, Region),
                ReplaceState(BackTab, Crew(0)),
                ApplyDomainEvent(Enter, FinishTurn, false),
            ],
            Region => vec![
                PushState(Esc, GameMenu),
                ReplaceState(Tab, Outpost(0)),
                ReplaceState(BackTab, Research),
                ApplyDomainEvent(Enter, FinishTurn, false),
            ],
            AssignToModule(c, m) => vec![
                PopState(Esc),
                ReplaceState(
                    Char('j'),
                    AssignToModule(c, circular_index((m as i32) + 1, &app.outpost.modules)),
                ),
                ReplaceState(
                    Char('k'),
                    AssignToModule(c, circular_index((m as i32) - 1, &app.outpost.modules)),
                ),
                ApplyDomainEvent(Enter, AssignCrewMemberToModule, true),
            ],
            AssignCrew(c, m) => vec![
                PopState(Esc),
                ReplaceState(
                    Char('j'),
                    AssignCrew(circular_index((c as i32) + 1, &app.outpost.crew), m),
                ),
                ReplaceState(
                    Char('k'),
                    AssignCrew(circular_index((c as i32) - 1, &app.outpost.crew), m),
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
            Outpost(_) => String::from("Outpost"),
            Crew(_) => String::from("Crew"),
            Region => String::from("Region"),
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
                            AssignToModule(c, m) | AssignCrew(c, m) => {
                                self.outpost.assign_crew_member_to_module(*c, *m);
                            }
                            _ => (),
                        },
                        FinishTurn => {
                            self.outpost.finish_turn();
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

    fn current_module(&mut self) -> Option<&mut Box<dyn Module>> {
        match self.state.last().unwrap() {
            State::Outpost(i) => Some(&mut self.outpost.modules[*i]),
            _ => None,
        }
    }

    fn header<B: Backend>(&self, f: &mut Frame<B>, area: Rect) {
        let consumption = self.outpost.consumption() + self.outpost.crew_upkeep();
        let production = self.outpost.production();
        let text = vec![Spans::from(vec![
            Span::styled(
                format!("turn {}", self.outpost.current_turn),
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
            State::Outpost(s) => {
                state.select(Some(*s));
                focused = true
            }
            _ => (),
        };

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
                .block(self.border("Outpost", focused))
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
            .outpost
            .modules
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
            .outpost
            .crew
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
            Crew(i) => {
                if self.outpost.crew.len() <= *i {
                    return;
                }

                let crew = &self.outpost.crew[*i];
                let description = self.outpost.describe_crew_member(&crew);

                let age = vec![Span::raw("mood: "), Span::raw(print_i32(crew.mood()))];
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
                    .constraints([Length(5), Length(8), Min(0)].as_ref())
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
                        Spans::from(age),
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
            Outpost(i) => {
                if self.outpost.modules.len() <= *i {
                    return;
                }

                let module = &self.outpost.modules[*i];
                let description = self.outpost.describe_module(&module);

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
                let crew_member = &self.outpost.crew[*c];
                self.modules_list_assign_to_module(f, crew_member, area);
            }
            AssignCrew(_, m) => {
                let module = &self.outpost.modules[*m];
                self.crew_list_assign_to_module(f, module, area);
            }
            Research => {
                f.render_widget(self.border(&self.current_state().to_string(), false), area)
            }
            Region => f.render_widget(self.border(&self.current_state().to_string(), false), area),
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
    if arr.len() == 0 {
        0
    } else {
        (((index % arr.len() as i32) + arr.len() as i32) % arr.len() as i32) as usize
    }
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
