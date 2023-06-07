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
    widgets::{Block, Borders, Paragraph, Tabs, Wrap},
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
    Outpost(usize),
    Crew(usize),
    Research(usize),
    Region(usize, usize),
}

impl State {
    fn transitions(&self) -> Vec<StateTransition> {
        use State::*;
        use StateTransition::*;
        match *self {
            GameMenu => vec![PopState(KeyCode::Esc), QuitAndSave(KeyCode::Char('q'))],
            Outpost(_) => vec![
                PushState(KeyCode::Esc, GameMenu),
                ReplaceState(KeyCode::Tab, Crew(0)),
            ],
            Crew(_) => vec![
                PushState(KeyCode::Esc, GameMenu),
                ReplaceState(KeyCode::Tab, Research(0)),
            ],
            Research(_) => vec![
                PushState(KeyCode::Esc, GameMenu),
                ReplaceState(KeyCode::Tab, Region(0, 0)),
            ],
            Region(_, _) => vec![
                PushState(KeyCode::Esc, GameMenu),
                ReplaceState(KeyCode::Tab, Outpost(0)),
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
            Region(_, _) => String::from("Region"),
            Research(_) => String::from("Research"),
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

fn print_keycode(code: &KeyCode) -> String {
    match code {
        KeyCode::Char(c) => c.to_string(),
        KeyCode::Esc => String::from("ESC"),
        KeyCode::Tab => String::from("Tab"),
        _ => String::from("??"),
    }
}
impl std::string::ToString for StateTransition {
    fn to_string(&self) -> String {
        use StateTransition::*;
        match self {
            PushState(c, s) | ReplaceState(c, s) => {
                format!("{}({})", s.to_string(), print_keycode(&c))
            }
            PopState(c) => format!("Back({})", print_keycode(&c)),
            QuitAndSave(c) => format!("Quit&Save({})", print_keycode(&c)),
        }
    }
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

fn footer<B: Backend>(f: &mut Frame<B>, app: &App, area: Rect) {
    let transitions = app.current_state().transitions();
    let navigations: Vec<Span> = transitions
        .iter()
        .map(|t| {
            Span::styled(
                t.to_string(),
                Style::default().fg(to_color(app.palette.text())),
            )
        })
        .collect();

    f.render_widget(
        Paragraph::new(Spans::from(navigations))
            .alignment(Alignment::Left)
            .wrap(Wrap { trim: true }),
        area,
    )
}

fn border(app: &App, title: String, focused: bool) -> Block {
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
    let modules = app
        .outpost
        .modules
        .iter()
        .map(|m| {
            Spans::from(vec![Span::styled(
                m.name(),
                Style::default().fg(to_color(app.palette.text())),
            )])
        })
        .collect();

    let mut index: usize = 0;
    let mut focused = false;
    match app.current_state() {
        State::Outpost(i) => {
            index = *i;
            focused = true
        }
        _ => (),
    };

    f.render_widget(
        Tabs::new(modules)
            .block(border(app, String::from("Outpost"), focused))
            .select(index)
            .highlight_style(
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .bg(to_color(app.palette.overlay0())),
            ),
        area,
    )
}
fn crew<B: Backend>(f: &mut Frame<B>, app: &App, area: Rect) {
    let modules = app
        .outpost
        .crew
        .iter()
        .map(|m| {
            Spans::from(vec![Span::styled(
                m.name(),
                Style::default().fg(to_color(app.palette.text())),
            )])
        })
        .collect();

    let mut index: usize = 0;
    let mut focused = false;
    match app.current_state() {
        State::Crew(i) => {
            index = *i;
            focused = true
        }
        _ => (),
    };

    f.render_widget(
        Tabs::new(modules)
            .block(border(app, String::from("Crew"), focused))
            .select(index)
            .highlight_style(
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .bg(to_color(app.palette.overlay0())),
            ),
        area,
    )
}
fn region<B: Backend>(f: &mut Frame<B>, app: &App, area: Rect) {
    let mut focused = false;
    match app.current_state() {
        State::Region(_, _) => focused = true,
        _ => (),
    };
    f.render_widget(border(app, String::from("Region"), focused), area)
}
fn research<B: Backend>(f: &mut Frame<B>, app: &App, area: Rect) {
    let mut focused = false;
    match app.current_state() {
        State::Research(_) => focused = true,
        _ => (),
    };
    f.render_widget(border(app, String::from("Research"), focused), area)
}
fn focus<B: Backend>(f: &mut Frame<B>, app: &App, area: Rect) {
    f.render_widget(
        Block::default()
            .title(app.current_state().to_string())
            .borders(Borders::ALL),
        area,
    )
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
        .constraints(
            [
                Constraint::Length(1),
                Constraint::Min(0),
                Constraint::Length(3),
            ]
            .as_ref(),
        )
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
    footer(f, app, outer_layout[2]);

    outpost(f, app, left_pane[0]);
    crew(f, app, right_pane[0]);

    region(f, app, left_pane[1]);
    research(f, app, right_pane[1]);

    focus(f, app, inner_layout[1]);
}
