use std::time::Instant;

use crate::{app::App, rtt};

use tui::{
    backend::Backend,
    Frame, 
    layout::{
        Alignment,
        Constraint,
        Layout,
        Rect, 
    },
    text::{Spans, Span},
    widgets::{
        Block,
        Borders,
        BorderType,
        Paragraph,
    }, style::{Style, Modifier, Color},
};

pub(crate) fn draw<B: Backend>(f: &mut Frame<B>, app: &mut App, now: Instant) {
    let chunks = Layout::default()
        .constraints(
            [
                Constraint::Min(5),
                Constraint::Ratio(2, 3),
                Constraint::Min(5),
            ].as_ref()
        )
        .split(f.size());

    draw_train_info(f, chunks[0], app);
    draw_train_status(f, chunks[1], app);

    let since_update = now - app.last_service_update.unwrap();
    let additional_info_text = vec![
        Spans::from(" Data Sourced From Realtime Trains"),
        Spans::from(" Controls: [q]uit"),
        Spans::from(format!(" Last Update: {}s", since_update.as_secs())),
    ];

    draw_text(
        f,
        chunks[2],
        "Additional Information",
        additional_info_text,
    );
}

fn draw_train_info<B>(f: &mut Frame<B>, area: Rect, app: &mut App)
where
    B: Backend,
{
    let service = app.service.as_ref().unwrap();
    draw_text(
        f,
        area,
        "Train Info",
        vec![
            Spans::from(format!(" {}", service.atoc_name)),
            Spans::from(format!(
                " {} {} to {}",
                service.origin.first().unwrap().public_time,
                service.origin.first().unwrap().description,
                service.destination.first().unwrap().description
            )),
        ]
    );
}

fn draw_train_status<B>(f: &mut Frame<B>, area: Rect, app: &mut App)
where
    B: Backend,
{
    let service = app.service.as_ref().unwrap();
    draw_text(
        f,
        area,
        "Train Status",
        service
            .locations
            .clone()
            .into_iter()
            .map(|loc| Spans::from(location_detail_line(area, loc)))
            .collect(),
    );
}

fn location_detail_line<'a>(area: Rect, loc: rtt::ServiceLocationDetail) -> Vec<Span<'a>> {
    let mut style = Style::default();
    if loc.realtime_arrival_actual.unwrap_or_default() || loc.realtime_departure_actual.unwrap_or_default() {
        style = style.add_modifier(Modifier::BOLD);
    }

    if loc.realtime_departure_actual.unwrap_or_default() && loc.realtime_gbtt_departure_lateness.unwrap_or(0) > 0 {
        style = style.fg(Color::Red);
    }

    if loc.realtime_departure_actual.unwrap_or_default() && loc.realtime_gbtt_departure_lateness.unwrap_or(0) < 0 {
        style = style.fg(Color::Green);
    }

    let mut parts = vec![
        Span::from(" "),
        Span::styled(loc.description, style),
    ];

    // Handle the platform
    let mut platform = loc.platform.unwrap_or_default();
    if !platform.is_empty() {
        if loc.platform_changed.unwrap_or(false) {
            platform += "!";
        }
            
        parts.extend(vec![Span::styled(format!(" [{}] ", platform), style)]);
    }

    parts.extend(vec![Span::styled("(", style)]);

    // Arrival Lateness
    parts.extend(vec![Span::styled(match loc.display_as.as_str() {
        "DESTINATION" => loc.realtime_gbtt_arrival_lateness.unwrap_or(0).to_string(),
        "CALL"        => format!("{}/", loc.realtime_gbtt_arrival_lateness.unwrap_or(0)),
        _             => "".to_string(),
    }, style)]);

    // Departure Lateness
    parts.extend(vec![Span::styled(match loc.display_as.as_str() {
        "ORIGIN" | "CALL" => loc.realtime_gbtt_departure_lateness.unwrap_or(0).to_string(),
        _                 => "".to_string(),
    }, style)]);

    parts.extend(vec![Span::styled(")", style)]);

    // Current Station?
    if loc.service_location.unwrap_or_default() == "AT_PLAT" {
        parts.extend(vec![Span::styled(" <===", style)]);
    }

    // Length so far?
    let length_so_far: usize = parts.iter().map(|it| it.width()).sum();
    let to_go: usize = 12;
    let padding: usize = usize::try_from(area.width).unwrap() - length_so_far - to_go;
    (0..padding).for_each(|_| parts.extend(vec![Span::from(" ")]));

    // Calling Times
    parts.extend(vec![Span::styled(match loc.display_as.as_str() {
        "CALL" | "DESTINATION" => loc.realtime_arrival.unwrap_or_default(),
        _                      => "    ".to_string(),
    }, style)]);

    parts.extend(vec![Span::from(" ")]);

    parts.extend(vec![Span::styled(match loc.display_as.as_str() {
        "ORIGIN" | "CALL" => loc.realtime_departure.unwrap_or_default(),
        _                 => "    ".to_string(),
    }, style)]);

    parts
}

fn draw_text<B>(f: &mut Frame<B>, area: Rect, title: &str, text: Vec<Spans>)
where
    B: Backend,
{
    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .title_alignment(Alignment::Center)
        .border_type(BorderType::Rounded);


    let paragraph = Paragraph::new(text).block(block);
    f.render_widget(paragraph, area);
}