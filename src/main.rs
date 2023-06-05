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
    pub index: usize,
    pub outpost: Outpost,
    pub palette: Flavour,

    pub initial_views: Vec<View>,
    pub view: View,
}

#[derive(Clone)]
enum View {
    Outpost(usize),
    Crew(usize),
    Region(usize, usize),
}

impl std::string::ToString for View {
    fn to_string(&self) -> String {
        let s = match self {
            View::Outpost(_) => "Outpost",
            View::Crew(_) => "Crew",
            View::Region(_, _) => "Region",
        };
        s.to_string()
    }
}

impl App {
    fn new() -> App {
        App {
            index: 0,
            outpost: Outpost::new(),
            palette: Flavour::Mocha,
            initial_views: vec![View::Outpost(0), View::Crew(0), View::Region(0, 0)],
            view: View::Outpost(0),
        }
    }

    pub fn next(&mut self) {
        self.index = (self.index + 1) % self.initial_views.len();
        self.view = self.initial_views[self.index].clone()
    }

    pub fn previous(&mut self) {
        if self.index > 0 {
            self.index -= 1;
        } else {
            self.index = self.initial_views.len() - 1;
        }
        self.view = self.initial_views[self.index].clone()
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
            match key.code {
                KeyCode::Char('q') => return Ok(()),
                KeyCode::Right => app.next(),
                KeyCode::Left => app.previous(),
                _ => {}
            }
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

fn header<'a>(app: &App) -> Paragraph<'a> {
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

    Paragraph::new(text)
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true })
}

fn tabs<'a>(app: &App) -> Tabs<'a> {
    let titles = app
        .initial_views
        .iter()
        .map(|v| {
            let string = v.to_string();
            let (first, rest) = string.split_at(1);
            Spans::from(vec![
                Span::styled(
                    first.to_owned(),
                    Style::default().fg(to_color(app.palette.rosewater())),
                ),
                Span::styled(
                    rest.to_owned(),
                    Style::default().fg(to_color(app.palette.text())),
                ),
            ])
        })
        .collect();
    Tabs::new(titles)
        .block(Block::default().borders(Borders::ALL))
        .select(app.index)
        .highlight_style(
            Style::default()
                .add_modifier(Modifier::BOLD)
                .bg(to_color(app.palette.overlay0())),
        )
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
    let index = match app.view {
        View::Outpost(index) => index,
        _ => 0,
    };

    f.render_widget(
        Tabs::new(modules)
            .block(Block::default().borders(Borders::ALL))
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
    let index = match app.view {
        View::Outpost(index) => index,
        _ => 0,
    };

    f.render_widget(
        Tabs::new(modules)
            .block(Block::default().borders(Borders::ALL))
            .select(index)
            .highlight_style(
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .bg(to_color(app.palette.overlay0())),
            ),
        area,
    )
}
fn region<B: Backend>(f: &mut Frame<B>, _app: &App, area: Rect) {
    f.render_widget(Block::default(), area)
}

fn to_color(value: Colour) -> Color {
    let (r, g, b) = value.into();
    Color::Rgb(r, g, b)
}

fn ui<B: Backend>(f: &mut Frame<B>, app: &App) {
    let size = f.size();
    let chunks = Layout::default()
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
        .split(size);

    let block = Block::default().style(
        Style::default()
            .bg(to_color(app.palette.base()))
            .fg(to_color(app.palette.text())),
    );
    f.render_widget(block, size);

    f.render_widget(header(app), chunks[0]);

    match app.index {
        0 => outpost(f, app, chunks[1]),
        1 => crew(f, app, chunks[1]),
        2 => region(f, app, chunks[1]),
        _ => unreachable!(),
    }

    f.render_widget(tabs(app), chunks[2]);
}
