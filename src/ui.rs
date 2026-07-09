use crate::app::App;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Sparkline};
use ratatui::Frame;

pub fn draw(f: &mut Frame, app: &App) {
    let outer = Layout::vertical([Constraint::Min(1), Constraint::Length(1)]).split(f.size());
    let cols = Layout::horizontal([Constraint::Length(28), Constraint::Min(10)]).split(outer[0]);
    draw_summaries(f, cols[0], app);
    draw_graphs(f, cols[1], app);

    let status = format!(
        " q quit | s stress | e external stress [{}] | 1-{} toggle pane | h help ",
        app.stress.mode(),
        app.panes.len()
    );
    f.render_widget(
        Paragraph::new(status).style(Style::default().add_modifier(Modifier::REVERSED)),
        outer[1],
    );

    if app.show_help {
        draw_help(f);
    }
}

fn temp_alert(app: &App) -> bool {
    app.panes.iter().any(|p| {
        p.name == "Temp"
            && p.latest
                .iter()
                .any(|r| r.value >= f64::from(app.cfg.temp_threshold))
    })
}

fn draw_summaries(f: &mut Frame, area: Rect, app: &App) {
    let alert = temp_alert(app);
    let mut items: Vec<ListItem> = Vec::new();
    for (i, p) in app.panes.iter().enumerate() {
        let marker = if p.visible { "" } else { " (hidden)" };
        items.push(ListItem::new(Line::styled(
            format!("[{}] {} ({}){}", i + 1, p.name, p.unit, marker),
            Style::default().add_modifier(Modifier::BOLD),
        )));
        let hot = p.name == "Temp" && alert;
        for r in &p.latest {
            let style = if hot && r.value >= f64::from(app.cfg.temp_threshold) {
                Style::default().fg(Color::Red)
            } else {
                Style::default()
            };
            items.push(ListItem::new(Line::styled(
                format!("  {} {:.1}", r.label, r.value),
                style,
            )));
        }
    }
    let list = List::new(items).block(Block::default().borders(Borders::ALL).title("Sensors"));
    f.render_widget(list, area);
}

fn draw_graphs(f: &mut Frame, area: Rect, app: &App) {
    let visible: Vec<&crate::app::Pane> = app.panes.iter().filter(|p| p.visible).collect();
    if visible.is_empty() {
        f.render_widget(
            Paragraph::new("all panes hidden — press 1-9 to show")
                .block(Block::default().borders(Borders::ALL)),
            area,
        );
        return;
    }
    let rows = Layout::vertical(vec![
        Constraint::Ratio(1, visible.len() as u32);
        visible.len()
    ])
    .split(area);
    for (pane, row) in visible.iter().zip(rows.iter()) {
        let width = row.width.saturating_sub(2) as usize;
        let data: Vec<u64> = pane
            .history
            .iter()
            .rev()
            .take(width)
            .rev()
            .map(|v| v.max(0.0) as u64)
            .collect();
        let latest = pane.history.back().copied().unwrap_or(0.0);
        let title = format!("{} {:.1}{}", pane.name, latest, pane.unit);
        let hot = pane.name == "Temp" && latest >= f64::from(app.cfg.temp_threshold);
        let style = if hot {
            Style::default().fg(Color::Red)
        } else {
            Style::default().fg(Color::Green)
        };
        let spark = Sparkline::default()
            .block(Block::default().borders(Borders::ALL).title(title))
            .style(style)
            .data(&data);
        f.render_widget(spark, *row);
    }
}

fn draw_help(f: &mut Frame) {
    let area = f.size();
    let w = 44.min(area.width);
    let h = 10.min(area.height);
    let rect = Rect::new(
        area.x + (area.width.saturating_sub(w)) / 2,
        area.y + (area.height.saturating_sub(h)) / 2,
        w,
        h,
    );
    let text = "s-tui-rs\n\n\
        q / Esc   quit\n\
        s         toggle built-in stress\n\
        e         toggle external stress(-ng)\n\
        1-9       show/hide sensor pane\n\
        h / F1    toggle this help\n\n\
        config: ~/.config/s-tui-rs/config.toml";
    f.render_widget(Clear, rect);
    f.render_widget(
        Paragraph::new(text).block(Block::default().borders(Borders::ALL).title("Help")),
        rect,
    );
}
