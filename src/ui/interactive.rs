use std::io;
use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::ExecutableCommand;
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph};
use ratatui::Terminal;

use crate::ui::RunSummary;

pub fn run_status<F>(mut fetch: F) -> anyhow::Result<Option<RunSummary>>
where
    F: FnMut() -> anyhow::Result<Vec<RunSummary>>,
{
    let mut summaries = fetch()?;
    if summaries.is_empty() {
        return Ok(None);
    }

    let mut stdout = io::stdout();
    enable_raw_mode()?;
    stdout.execute(EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut selected: usize = 0;
    let result = run_loop(&mut terminal, &mut fetch, &mut summaries, &mut selected);
    let cleanup_result = restore_terminal(&mut terminal);

    match (result, cleanup_result) {
        (Ok(value), Ok(())) => Ok(value),
        (Err(err), _) => Err(err),
        (Ok(_), Err(err)) => Err(err),
    }
}

fn run_loop<F>(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    fetch: &mut F,
    summaries: &mut Vec<RunSummary>,
    selected: &mut usize,
) -> anyhow::Result<Option<RunSummary>>
where
    F: FnMut() -> anyhow::Result<Vec<RunSummary>>,
{
    loop {
        terminal.draw(|frame| draw(frame, summaries, *selected))?;

        if event::poll(Duration::from_millis(250))? {
            match event::read()? {
                Event::Key(key) => match key {
                    KeyEvent {
                        code: KeyCode::Char('q'),
                        ..
                    }
                    | KeyEvent {
                        code: KeyCode::Esc, ..
                    } => return Ok(None),
                    KeyEvent {
                        code: KeyCode::Char('r'),
                        modifiers: KeyModifiers::NONE,
                        ..
                    } => {
                        *summaries = fetch()?;
                        if summaries.is_empty() {
                            return Ok(None);
                        }
                        if *selected >= summaries.len() {
                            *selected = summaries.len() - 1;
                        }
                    }
                    KeyEvent {
                        code: KeyCode::Up, ..
                    } => {
                        if *selected > 0 {
                            *selected -= 1;
                        }
                    }
                    KeyEvent {
                        code: KeyCode::Down,
                        ..
                    } => {
                        if *selected + 1 < summaries.len() {
                            *selected += 1;
                        }
                    }
                    KeyEvent {
                        code: KeyCode::Enter,
                        ..
                    } => return Ok(Some(summaries[*selected].clone())),
                    _ => {}
                },
                Event::Resize(_, _) => {}
                _ => {}
            }
        }
    }
}

fn draw(frame: &mut ratatui::Frame, summaries: &[RunSummary], selected: usize) {
    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(1)].as_ref())
        .split(frame.size());

    let body_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(35), Constraint::Percentage(65)].as_ref())
        .split(main_layout[0]);

    let items: Vec<ListItem> = summaries
        .iter()
        .map(|summary| ListItem::new(summary.replay_id.clone()))
        .collect();
    let mut state = ListState::default();
    state.select(Some(selected));

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("runs"))
        .highlight_style(Style::default().add_modifier(Modifier::BOLD));
    frame.render_stateful_widget(list, body_layout[0], &mut state);

    let detail = summaries.get(selected);
    let detail_lines = match detail {
        Some(summary) => vec![
            Line::from(vec![Span::raw(format!("replay_id: {}", summary.replay_id))]),
            Line::from(vec![Span::raw(format!(
                "tasks: {}/{}",
                summary.done, summary.total
            ))]),
            Line::from(vec![Span::raw(format!("last: {}", summary.last))]),
        ],
        None => vec![Line::from(Span::raw("no selection"))],
    };

    let detail_widget =
        Paragraph::new(detail_lines).block(Block::default().borders(Borders::ALL).title("details"));
    frame.render_widget(detail_widget, body_layout[1]);

    let help = Paragraph::new("q quit  r refresh  enter print selected and exit")
        .block(Block::default().borders(Borders::ALL));
    frame.render_widget(help, main_layout[1]);
}

fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> anyhow::Result<()> {
    disable_raw_mode()?;
    terminal.backend_mut().execute(LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}
