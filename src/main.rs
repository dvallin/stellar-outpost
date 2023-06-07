use catppuccin::{Colour, Flavour};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use model::outpost::Outpost;
use std::{error::Error, io};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Cell, List, ListItem, ListState, Paragraph, Row, Table, Wrap},
    Frame, Terminal,
};

mod model;

struct App {
    pub outpost: Outpost,
    pub palette: Flavour,

    pub state: Vec<State>,
}

#[derive(Clone)]
enum State {
    GameMenu,
    Outpost(i32),
    Crew(i32),
    Research,
    Region,
}

impl State {
    fn transitions(&self) -> Vec<StateTransition> {
        use State::*;
        use StateTransition::*;
        match *self {
            GameMenu => vec![PopState(KeyCode::Esc), QuitAndSave(KeyCode::Char('q'))],
            Outpost(i) => vec![
                PushState(KeyCode::Esc, GameMenu),
                ReplaceState(KeyCode::Tab, Crew(0)),
                ReplaceState(KeyCode::BackTab, Region),
                ReplaceState(KeyCode::Char('j'), Outpost(i + 1)),
                ReplaceState(KeyCode::Char('k'), Outpost(i - 1)),
            ],
            Crew(i) => vec![
                PushState(KeyCode::Esc, GameMenu),
                ReplaceState(KeyCode::Tab, Research),
                ReplaceState(KeyCode::BackTab, Outpost(0)),
                ReplaceState(KeyCode::Char('j'), Crew(i + 1)),
                ReplaceState(KeyCode::Char('k'), Crew(i - 1)),
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
        }
    }
}

#[derive(Clone)]
enum StateTransition {
    PushState(KeyCode, State),
    PopState(KeyCode),
    ReplaceState(KeyCode, State),
    QuitAndSave(KeyCode),
}

impl App {
    fn new() -> App {
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

    fn current_state(&self) -> &State {
        self.state.last().unwrap()
    }

    fn input(&mut self, code: KeyCode) -> Option<io::Result<()>> {
        use StateTransition::*;
        let transitions = self.current_state().transitions();
        let transition = transitions.iter().find(|t| match t {
            PopState(c) | QuitAndSave(c) => c.eq(&code),
            PushState(c, _) | ReplaceState(c, _) => c.eq(&code),
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
                    std::fs::create_dir_all("./saves");
                    std::fs::write(output_path, data);
                    Some(Ok(()))
                }
            },
            None => None,
        };
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // create app and run it
    let app = App::new();
    let res = run_app(&mut terminal, app);

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err)
    }

    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, &app))?;

        if let Event::Key(key) = event::read()? {
            match app.input(key.code) {
                Some(r) => return r,
                None => (),
            };
        }
    }
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

fn header<B: Backend>(f: &mut Frame<B>, app: &App, area: Rect) {
    let consumption = app.outpost.consumption();
    let production = app.outpost.production();
    let text = vec![Spans::from(vec![
        Span::styled(
            format!(
                "{}/{}",
                consumption.energy.to_string(),
                app.outpost.resources.energy.to_string(),
            ),
            Style::default().fg(to_color(app.palette.yellow())),
        ),
        Span::raw(" | "),
        Span::styled(
            format!(
                "{}/{}",
                consumption.living_space.to_string(),
                app.outpost.resources.living_space.to_string(),
            ),
            Style::default().fg(to_color(app.palette.peach())),
        ),
        Span::raw(" | "),
        Span::styled(
            format!(
                "{}({})",
                app.outpost.resources.minerals.to_string(),
                print_i32(production.minerals - consumption.minerals),
            ),
            Style::default().fg(to_color(app.palette.sapphire())),
        ),
        Span::raw(" | "),
        Span::styled(
            format!(
                "{}({})",
                app.outpost.resources.food.to_string(),
                print_i32(production.food - consumption.food),
            ),
            Style::default().fg(to_color(app.palette.green())),
        ),
        Span::raw(" | "),
        Span::styled(
            format!(
                "{}({})",
                app.outpost.resources.water.to_string(),
                print_i32(production.water - consumption.water),
            ),
            Style::default().fg(to_color(app.palette.blue())),
        ),
    ])];

    f.render_widget(
        Paragraph::new(text)
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true }),
        area,
    )
}

fn border<'a>(app: &'a App, title: &'a str, focused: bool) -> Block<'a> {
    let fg: Colour = if focused {
        app.palette.lavender()
    } else {
        app.palette.overlay0()
    };
    Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(to_color(fg)))
}

fn outpost<B: Backend>(f: &mut Frame<B>, app: &App, area: Rect) {
    let modules: Vec<ListItem> = app
        .outpost
        .modules
        .iter()
        .map(|m| {
            ListItem::new(Spans::from(vec![Span::styled(
                m.name(),
                Style::default().fg(to_color(app.palette.text())),
            )]))
        })
        .collect();

    let mut state: ListState = ListState::default();
    let mut focused = false;
    match app.current_state() {
        State::Outpost(s) => {
            state.select(Some(circular_index(*s, &modules)));
            focused = true
        }
        _ => (),
    };

    f.render_stateful_widget(
        List::new(modules)
            .block(border(app, &String::from("Outpost"), focused))
            .highlight_style(
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .bg(to_color(app.palette.overlay0())),
            )
            .highlight_symbol("> "),
        area,
        &mut state,
    )
}
fn crew<B: Backend>(f: &mut Frame<B>, app: &App, area: Rect) {
    let crew: Vec<ListItem> = app
        .outpost
        .crew
        .iter()
        .map(|m| {
            ListItem::new(Spans::from(vec![Span::styled(
                m.name(),
                Style::default().fg(to_color(app.palette.text())),
            )]))
        })
        .collect();

    let mut state: ListState = ListState::default();
    let mut focused = false;
    match app.current_state() {
        State::Crew(s) => {
            state.select(Some(circular_index(*s, &crew)));
            focused = true
        }
        _ => (),
    };

    f.render_stateful_widget(
        List::new(crew)
            .block(border(app, &String::from("Crew"), focused))
            .highlight_style(
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .bg(to_color(app.palette.overlay0())),
            )
            .highlight_symbol("> "),
        area,
        &mut state,
    )
}
fn region<B: Backend>(f: &mut Frame<B>, app: &App, area: Rect) {
    let mut focused = false;
    match app.current_state() {
        State::Region => focused = true,
        _ => (),
    };
    f.render_widget(border(app, &String::from("Region"), focused), area)
}
fn research<B: Backend>(f: &mut Frame<B>, app: &App, area: Rect) {
    let mut focused = false;
    match app.current_state() {
        State::Research => focused = true,
        _ => (),
    };
    f.render_widget(border(app, &String::from("Research"), focused), area)
}
fn focus<B: Backend>(f: &mut Frame<B>, app: &App, area: Rect) {
    match app.current_state() {
        State::Crew(i) => {
            let index = circular_index(*i, &app.outpost.crew);
            let crew = &app.outpost.crew[index];

            f.render_widget(border(app, crew.name(), false), area)
        }
        State::Outpost(i) => {
            let index = circular_index(*i, &app.outpost.modules);
            let module = &app.outpost.modules[index];

            f.render_widget(border(app, module.name(), false), area)
        }
        State::GameMenu => {
            let header_data = vec!["Action", "Key"];
            let data: Vec<Vec<&str>> = vec![
                vec!["Toggle Game Menu", "Esc"],
                vec!["Quit (in game menu)", "q"],
                vec!["next pane", "Tab"],
                vec!["previous pane", "Shift+Tab"],
                vec!["up (e.g. in lists)", "k"],
                vec!["down (e.g. in lists)", "j"],
            ];

            let header_cells = header_data.iter().map(|h| {
                Cell::from(*h).style(Style::default().fg(to_color(app.palette.subtext0())))
            });
            let header = Row::new(header_cells).height(1).bottom_margin(1);
            let rows = data
                .iter()
                .map(|row| Row::new(row.iter().map(|c| Cell::from(*c))));
            f.render_widget(
                Table::new(rows)
                    .header(header)
                    .block(border(app, &String::from("Game Menu"), false))
                    .widths(&[Constraint::Percentage(70), Constraint::Percentage(30)]),
                area,
            )
        }
        _ => f.render_widget(border(app, &app.current_state().to_string(), false), area),
    }
}

fn to_color(value: Colour) -> Color {
    let (r, g, b) = value.into();
    Color::Rgb(r, g, b)
}

fn ui<B: Backend>(f: &mut Frame<B>, app: &App) {
    let window = f.size();
    let block = Block::default().style(
        Style::default()
            .bg(to_color(app.palette.base()))
            .fg(to_color(app.palette.text())),
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
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(inner_layout[2]);

    header(f, app, outer_layout[0]);

    outpost(f, app, left_pane[0]);
    crew(f, app, right_pane[0]);

    region(f, app, left_pane[1]);
    research(f, app, right_pane[1]);

    focus(f, app, inner_layout[1]);
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
