use crossterm::event::{KeyCode, KeyEvent};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};
use ratatui::Frame;

use crate::app::{App, Screen};
use crate::data::season;
use crate::economy::{format_money, value_cents};
use crate::geography::SITES;
use crate::map::{self, tile};
use crate::weather;

pub fn handle_key(app: &mut App, key: KeyEvent) -> bool {
    match app.screen {
        Screen::Title => match key.code {
            KeyCode::Enter | KeyCode::Char('n') => app.screen = Screen::NewGame,
            KeyCode::Char('a') => app.screen = Screen::Almanac,
            KeyCode::Char('?') | KeyCode::Char('h') => app.screen = Screen::Help,
            KeyCode::Char('q') | KeyCode::Esc => return true,
            _ => {}
        },
        Screen::NewGame => match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                app.pick_gear = app.pick_gear.saturating_sub(1);
            }
            KeyCode::Down | KeyCode::Char('j') => {
                app.pick_gear = (app.pick_gear + 1).min(1);
            }
            KeyCode::Enter => {
                app.pick_site = 0;
                app.screen = Screen::PickSite;
            }
            KeyCode::Esc | KeyCode::Char('q') => app.screen = Screen::Title,
            _ => {}
        },
        Screen::PickSite => {
            let n = app.current_camps().len();
            match key.code {
                KeyCode::Up | KeyCode::Char('k') => {
                    app.pick_site = app.pick_site.saturating_sub(1);
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    if n > 0 {
                        app.pick_site = (app.pick_site + 1).min(n - 1);
                    }
                }
                KeyCode::Enter => app.start_game(),
                KeyCode::Esc => app.screen = Screen::NewGame,
                _ => {}
            }
        }
        Screen::Play => match key.code {
            KeyCode::Char('h') | KeyCode::Left => app.move_by(-1, 0),
            KeyCode::Char('l') | KeyCode::Right => app.move_by(1, 0),
            KeyCode::Char('k') | KeyCode::Up => app.move_by(0, -1),
            KeyCode::Char('j') | KeyCode::Down => app.move_by(0, 1),
            KeyCode::Char('H') => app.move_by(-5, 0),
            KeyCode::Char('L') => app.move_by(5, 0),
            KeyCode::Char('K') => app.move_by(0, -5),
            KeyCode::Char('J') => app.move_by(0, 5),
            KeyCode::Char('f') => app.fish(),
            KeyCode::Char('c') => app.camp_day(),
            KeyCode::Char('.') | KeyCode::Char('w') => app.wait_day(),
            KeyCode::Char('d') => app.deliver(),
            KeyCode::Char('t') => app.enter_town(),
            KeyCode::Char('m') => app.move_camp(),
            KeyCode::Char('a') => app.screen = Screen::Almanac,
            KeyCode::Char('?') => app.screen = Screen::Help,
            KeyCode::Char('q') => return true,
            _ => {}
        },
        Screen::Town => match key.code {
            KeyCode::Char('1') | KeyCode::Char('f') => app.town_food(),
            KeyCode::Char('2') | KeyCode::Char('p') => app.town_play(),
            KeyCode::Char('3') | KeyCode::Char('r') => app.town_repair(),
            KeyCode::Esc | KeyCode::Char('t') | KeyCode::Char('q') => {
                app.screen = Screen::Play;
            }
            _ => {}
        },
        Screen::Almanac => match key.code {
            KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('a') => {
                app.screen = if app.game.is_some() {
                    Screen::Play
                } else {
                    Screen::Title
                };
            }
            KeyCode::Down | KeyCode::Char('j') => app.almanac_scroll += 1,
            KeyCode::Up | KeyCode::Char('k') => {
                app.almanac_scroll = app.almanac_scroll.saturating_sub(1);
            }
            _ => {}
        },
        Screen::Help => match key.code {
            KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('?') => {
                app.screen = if app.game.is_some() {
                    Screen::Play
                } else {
                    Screen::Title
                };
            }
            _ => {}
        },
        Screen::GameOver => match key.code {
            KeyCode::Char('q') | KeyCode::Esc => return true,
            KeyCode::Char('a') => app.screen = Screen::Almanac,
            _ => {}
        },
    }
    false
}

pub fn draw(frame: &mut Frame, app: &App) {
    match app.screen {
        Screen::Title => draw_title(frame),
        Screen::NewGame => draw_new_game(frame, app),
        Screen::PickSite => draw_pick_site(frame, app),
        Screen::Play => draw_play(frame, app),
        Screen::Town => {
            draw_play(frame, app);
            draw_town(frame, app);
        }
        Screen::Almanac => draw_almanac(frame, app),
        Screen::Help => draw_help(frame),
        Screen::GameOver => draw_over(frame, app),
    }
}

fn draw_title(frame: &mut Frame) {
    let text = vec![
        Line::from(""),
        Line::from(Span::styled(
            "HARVESTER",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from("Kodiak Management Area  ·  2025  ·  commercial salmon"),
        Line::from(""),
        Line::from("A Dwarf Fortress-style TUI. You run one Kodiak permit on the"),
        Line::from("real OSM map of the archipelago. Setnet (S04K) only in the"),
        Line::from("Central Section of Northwest Kodiak — Uganik, Uyak, Amook Pass,"),
        Line::from("Terror, Zachar — and inner Alitak until 4 September. Purse seine"),
        Line::from("is mobile. Beach seine is not the default. You live at a fish"),
        Line::from("camp. Town is optional, and too much of it and the crew quit."),
        Line::from(""),
        Line::from("Openings follow 2025 ADF&G emergency orders. Run strength follows"),
        Line::from("the 2025 season summary. There is no 2025 AMR here — the game"),
        Line::from("does not invent week-by-district harvest."),
        Line::from(""),
        Line::from(Span::styled(
            "n  new season    a  almanac    ?  help    q  quit",
            Style::default().fg(Color::Cyan),
        )),
    ];
    frame.render_widget(
        Paragraph::new(text)
            .block(Block::default().borders(Borders::ALL).title(" Harvester "))
            .wrap(Wrap { trim: false }),
        frame.area(),
    );
}

fn draw_new_game(frame: &mut Frame, app: &App) {
    let mut lines = vec![
        Line::from("Choose a permit. Beach seine fished fewer than three KMA permits in 2025 and is not offered."),
        Line::from(""),
    ];
    for (i, (g, desc)) in App::gear_choices().iter().enumerate() {
        let mark = if i == app.pick_gear { ">" } else { " " };
        lines.push(Line::from(format!("{mark}  {}  {desc}", g.permit_code())));
    }
    lines.push(Line::from(""));
    lines.push(Line::from("Enter  continue   j/k  move   Esc  back"));
    frame.render_widget(
        Paragraph::new(lines)
            .block(Block::default().borders(Borders::ALL).title(" New season "))
            .wrap(Wrap { trim: false }),
        frame.area(),
    );
}

fn draw_pick_site(frame: &mut Frame, app: &App) {
    let camps = app.current_camps();
    let mut lines = vec![
        Line::from("Pick a fish camp. You are not required to live in a town or village."),
        Line::from("Setnet sites are legal Central Section beaches and inner Alitak only."),
        Line::from(""),
    ];
    for (i, s) in camps.iter().enumerate() {
        let mark = if i == app.pick_site { ">" } else { " " };
        lines.push(Line::from(format!(
            "{mark}  {:<28}  {}",
            s.osm_name,
            s.section.name()
        )));
    }
    lines.push(Line::from(""));
    lines.push(Line::from("Enter  camp here   j/k  move   Esc  back"));
    frame.render_widget(
        Paragraph::new(lines)
            .block(Block::default().borders(Borders::ALL).title(" Fish camp "))
            .wrap(Wrap { trim: false }),
        frame.area(),
    );
}

fn draw_play(frame: &mut Frame, app: &App) {
    let area = frame.area();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(4),
            Constraint::Min(8),
            Constraint::Length(6),
            Constraint::Length(2),
        ])
        .split(area);

    let mut head = Vec::new();
    if let Some(w) = app.weather_today() {
        head.push(Line::from(weather::line(w)));
    }
    for s in app.status_lines() {
        head.push(Line::from(s));
    }
    frame.render_widget(
        Paragraph::new(head).block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Harvester  ·  KMA 2025  ·  OSM names "),
        ),
        chunks[0],
    );

    frame.render_widget(
        Paragraph::new(map_lines(app, chunks[1]))
            .block(Block::default().borders(Borders::ALL).title(" Chart ")),
        chunks[1],
    );

    let log = app
        .game
        .as_ref()
        .map(|g| {
            g.log
                .iter()
                .rev()
                .take(4)
                .rev()
                .cloned()
                .collect::<Vec<_>>()
                .join("\n")
        })
        .unwrap_or_default();
    frame.render_widget(
        Paragraph::new(log)
            .wrap(Wrap { trim: true })
            .block(Block::default().borders(Borders::ALL).title(" Deck log ")),
        chunks[2],
    );

    frame.render_widget(
        Paragraph::new("hjkl move  HJKL jump  f fish  c camp/mend  . wait  d deliver  t town  m move-camp  a almanac  ? help  q quit"),
        chunks[3],
    );
}

fn map_lines(app: &App, area: Rect) -> Vec<Line<'static>> {
    let Some(g) = &app.game else {
        return Vec::new();
    };
    let vw = area.width.saturating_sub(2) as i32;
    let vh = area.height.saturating_sub(2) as i32;
    if vw < 4 || vh < 4 {
        return Vec::new();
    }
    let x0 = (g.x - vw / 2).clamp(0, (app.map.width - vw).max(0));
    let y0 = (g.y - vh / 2).clamp(0, (app.map.height - vh).max(0));

    let mut rows: Vec<Vec<(char, Color)>> = Vec::new();
    for y in y0..y0 + vh {
        let mut row = Vec::new();
        for x in x0..x0 + vw {
            let ch = tile(&app.map, x, y);
            let color = match ch {
                '~' => Color::Cyan,
                '#' => Color::Yellow,
                '.' => Color::Green,
                _ => Color::DarkGray,
            };
            row.push((ch, color));
        }
        rows.push(row);
    }

    let put = |rows: &mut Vec<Vec<(char, Color)>>, x: i32, y: i32, ch: char, color: Color| {
        let xi = x - x0;
        let yi = y - y0;
        if yi >= 0 && xi >= 0 && (yi as usize) < rows.len() && (xi as usize) < rows[0].len() {
            rows[yi as usize][xi as usize] = (ch, color);
        }
    };

    for s in SITES {
        let (sx, sy) = map::latlon_to_tile(&app.map, s.lon, s.lat);
        let ch = if s.town && !s.camp {
            '■'
        } else if s.camp {
            '▲'
        } else {
            '+'
        };
        put(&mut rows, sx, sy, ch, Color::White);
    }
    put(&mut rows, g.x, g.y, '@', Color::Magenta);

    // Labels that fall in view (OSM names).
    for lab in &app.map.labels {
        if lab.x < x0 || lab.y < y0 || lab.x >= x0 + vw || lab.y >= y0 + vh {
            continue;
        }
        let yi = (lab.y - y0) as usize;
        let mut xi = (lab.x - x0 + 1) as usize;
        for ch in lab.name.chars() {
            if yi < rows.len() && xi < rows[yi].len() {
                rows[yi][xi] = (ch, Color::Gray);
                xi += 1;
            }
        }
    }

    rows.into_iter()
        .map(|row| {
            Line::from(
                row.into_iter()
                    .map(|(ch, color)| Span::styled(ch.to_string(), Style::default().fg(color)))
                    .collect::<Vec<_>>(),
            )
        })
        .collect()
}

fn draw_town(frame: &mut Frame, app: &App) {
    let area = centered(frame.area(), 64, 14);
    let name = app
        .nearby_town()
        .map(|s| s.osm_name)
        .unwrap_or("town");
    let body = vec![
        Line::from(format!("You are in {name}. Crew would rather be at camp during a closure.")),
        Line::from(""),
        Line::from("1  f   buy a week of food   ($120)"),
        Line::from("2  p   town / playtime      ($80)   — they will quit if this piles up"),
        Line::from("3  r   repair engine        ($2,500) after a whale"),
        Line::from("Esc    leave"),
    ];
    frame.render_widget(Clear, area);
    frame.render_widget(
        Paragraph::new(body)
            .block(Block::default().borders(Borders::ALL).title(format!(" {name} ")))
            .wrap(Wrap { trim: false }),
        area,
    );
}

fn draw_almanac(frame: &mut Frame, app: &App) {
    let s = season();
    let mut lines = vec![
        Line::from(Span::styled(
            s.title.clone(),
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from(format!("Released {}  —  official KMA totals, not your boat", s.released)),
        Line::from(s.note.clone()),
        Line::from(""),
        Line::from(format!(
            "Chinook {:>10}     10-yr avg {:>10}     forecast {:>10}",
            s.harvest.chinook, s.harvest_10yr_avg.chinook, s.forecast_2025.chinook
        )),
        Line::from(format!(
            "Sockeye {:>10}     10-yr avg {:>10}     forecast {:>10}",
            s.harvest.sockeye, s.harvest_10yr_avg.sockeye, s.forecast_2025.sockeye
        )),
        Line::from(format!(
            "Coho    {:>10}     10-yr avg {:>10}     forecast {:>10}",
            s.harvest.coho, s.harvest_10yr_avg.coho, s.forecast_2025.coho
        )),
        Line::from(format!(
            "Pink    {:>10}     10-yr avg {:>10}     forecast {:>10}",
            s.harvest.pink, s.harvest_10yr_avg.pink, s.forecast_2025.pink
        )),
        Line::from(format!(
            "Chum    {:>10}     10-yr avg {:>10}     forecast {:>10}",
            s.harvest.chum, s.harvest_10yr_avg.chum, s.forecast_2025.chum
        )),
        Line::from(format!(
            "Total   {:>10}     10-yr avg {:>10}     forecast {:>10}",
            s.harvest.total, s.harvest_10yr_avg.total, s.forecast_2025.total
        )),
        Line::from(format!(
            "Common-property exvessel ≈ ${}  (season summary)",
            s.exvessel_common_property_usd
        )),
        Line::from(""),
        Line::from("2025 preliminary Kodiak prices (not invented):"),
        Line::from("  k $0.10/lb · r $1.34 · h $0.70 · p $0.30 · c $0.39"),
        Line::from(""),
    ];
    for n in &s.run_notes {
        lines.push(Line::from(n.clone()));
        lines.push(Line::from(""));
    }
    if let Some(g) = &app.game {
        lines.push(Line::from("Your permit (simulation, not ADF&G harvest):"));
        lines.push(Line::from(format!(
            "  landed {} fish  value {}  hold {}",
            g.landed.total(),
            format_money(value_cents(g.landed)),
            g.hold.total()
        )));
    }
    lines.push(Line::from(""));
    lines.push(Line::from("Esc / a  back"));
    let skip = app.almanac_scroll;
    let shown: Vec<Line> = lines.into_iter().skip(skip).collect();
    frame.render_widget(
        Paragraph::new(shown)
            .block(Block::default().borders(Borders::ALL).title(" Almanac  ·  official 2025 "))
            .wrap(Wrap { trim: false }),
        frame.area(),
    );
}

fn draw_help(frame: &mut Frame) {
    let text = vec![
        Line::from("Harvester is a local single-player commercial salmon TUI set only in the"),
        Line::from("Kodiak Management Area, year 2025."),
        Line::from(""),
        Line::from("hjkl / arrows   walk the OSM chart     HJKL  jump"),
        Line::from("f               fish if the section is open and your gear is legal"),
        Line::from("c               camp / mend  (keep crew here during closures)"),
        Line::from(".               wait a day"),
        Line::from("d               deliver the hold to a tender"),
        Line::from("t               town menu if you are on an OSM village/city"),
        Line::from("m               move fish camp to the nearest legal site"),
        Line::from("a               official 2025 season almanac"),
        Line::from(""),
        Line::from("S04K setnet is legal only in the Central Section (Uganik, Uyak, Amook"),
        Line::from("Pass, Terror, Zachar) and inner Alitak until 4 September."),
        Line::from("Weather is PADQ + NDBC 46077; CWFAER PKZ headlines from those obs."),
        Line::from("Otters foul nets. Sea lions steal fish. Whale pods kill the engine."),
        Line::from(""),
        Line::from("Development continues on Origin. This public GitHub tree is the clone."),
        Line::from("Esc  back"),
    ];
    frame.render_widget(
        Paragraph::new(text)
            .block(Block::default().borders(Borders::ALL).title(" Help "))
            .wrap(Wrap { trim: false }),
        frame.area(),
    );
}

fn draw_over(frame: &mut Frame, app: &App) {
    let g = app.game.as_ref();
    let reason = g
        .and_then(|g| g.over.clone())
        .unwrap_or_else(|| "Season over.".into());
    let landed = g.map(|g| g.landed).unwrap_or_default();
    let cash = g.map(|g| g.cash).unwrap_or(0);
    let s = season();
    let text = vec![
        Line::from(reason),
        Line::from(""),
        Line::from(format!(
            "Your permit landed {} fish (k {} r {} h {} p {} c {}) worth {}.",
            landed.total(),
            landed.chinook,
            landed.sockeye,
            landed.coho,
            landed.pink,
            landed.chum,
            format_money(value_cents(landed))
        )),
        Line::from(format!("Cash on the books: {}.", format_money(cash))),
        Line::from(""),
        Line::from(format!(
            "Official 2025 KMA commercial harvest was {} fish. That number is ADF&G's,",
            s.harvest.total
        )),
        Line::from("not yours. There is no week-by-district table in this game (no 2025 AMR)."),
        Line::from(""),
        Line::from("a  almanac    q  quit"),
    ];
    frame.render_widget(
        Paragraph::new(text)
            .block(Block::default().borders(Borders::ALL).title(" Season "))
            .wrap(Wrap { trim: false }),
        frame.area(),
    );
}

fn centered(area: Rect, w: u16, h: u16) -> Rect {
    let x = area.x + area.width.saturating_sub(w) / 2;
    let y = area.y + area.height.saturating_sub(h) / 2;
    Rect::new(x, y, w.min(area.width), h.min(area.height))
}
