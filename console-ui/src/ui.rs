use ratatui::{
    layout::{Alignment, Constraint},
    style::{Color, Style},
    widgets::{Block, BorderType, Borders, Cell, Paragraph, Row, Table},
    Frame,
};

use crate::app::App;

/// Renders the user interface widgets.
pub fn render(app: &mut App, frame: &mut Frame) {
    // This is where you add new widgets.
    // See the following resources:
    // - https://docs.rs/ratatui/latest/ratatui/widgets/index.html
    // - https://github.com/ratatui-org/ratatui/tree/master/examples
    frame.render_widget(
        Paragraph::new(format!(
            "This is a tui template.\n\
                Press `Esc`, `Ctrl-C` or `q` to stop running.\n\
                Press left and right to increment and decrement the counter respectively.\n\
                Counter: {}",
            app.counter
        ))
        .block(
            Block::bordered()
                .title("Template")
                .title_alignment(Alignment::Center)
                .border_type(BorderType::Rounded),
        )
        .style(Style::default().fg(Color::Cyan).bg(Color::Black))
        .centered(),
        frame.size(),
    );

    let block = Block::default().title("System Info").borders(Borders::ALL);

    let area = frame.size();
    let inner_area = block.inner(area);

    frame.render_widget(block, area);
    frame.set_cursor(
        inner_area.x + inner_area.width / 2,
        inner_area.y + inner_area.height / 2,
    );

    let rows: Vec<Row<'static>> = vec![
        Row::new(vec![
            Cell::from("Total Cpus"),
            Cell::from(format!("{}%", app.system_info.total_cpus)),
        ]),
        Row::new(vec![
            Cell::from("Total Memory"),
            Cell::from(format!("{}%", app.system_info.total_memory)),
        ]),
        // Add more rows as needed...
    ];
    frame.render_widget(
        Table::new(rows, &[Constraint::Length(15), Constraint::Length(10)])
            .header(
                Row::new(vec![Cell::from("Metric"), Cell::from("Value")])
                    .style(Style::default().fg(Color::White))
                    .bottom_margin(1),
            )
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .style(Style::default().fg(Color::White))
                    .title("System Info"),
            ),
        inner_area,
    );
}
