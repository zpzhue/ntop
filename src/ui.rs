use ratatui::{
    backend::Backend,
    layout::Alignment,
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::Modifier;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Cell, List, ListItem, Row, Sparkline, Table, TableState};
use crate::system::Size;
use crate::system::AppSystemInfo;

use crate::app::App;


const TOTAL_RENDER_LENGTH: usize = 72;


fn format_usage_string(prefix: &str, usage: f32) -> String {
    let usage_length = TOTAL_RENDER_LENGTH as f32 * (usage / 100.0);

    let usage_string = String::from_iter(vec!['|'; usage_length as usize]);
    let free_string = String::from_iter(vec![' '; TOTAL_RENDER_LENGTH - usage_length as usize]);
    format!("{}[{}{} {:.2}%]", prefix, usage_string, free_string, usage)
}


fn render_usage_info<B: Backend>(frame: &mut Frame<B>, area: Rect, app_sys: &mut AppSystemInfo,) {
    // 1. 渲染CPU使用率
    let cpu_usage_str = format_usage_string("CPU ", app_sys.global_cpu_usage);

    // 2. 渲染内存使用率
    let mem_usage_str = format_usage_string("MEM ", app_sys.memory_info.memory_usage);

    // 3. 渲染Swap使用率
    let swap_usage_str = format_usage_string("SWAP", app_sys.memory_info.swap_usage);

    // 4. 创建渲染组件
    let usage_info = vec![
        Line::from(""),
        Line::from(cpu_usage_str),
        Line::from(""),
        Line::from(mem_usage_str),
        Line::from(""),
        Line::from(swap_usage_str),
        Line::from(""),
    ];

    let create_block = || {
        Block::default()
            .borders(Borders::NONE)
            .style(Style::default().fg(Color::Gray))
    };

    // 5 渲染到指定区域
    let paragraph = Paragraph::new(usage_info)
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::Gray))
        .block(create_block());
    frame.render_widget(paragraph, area);
}


fn render_network_info<B: Backend>(frame: &mut Frame<B>, area: Rect, app_sys: &mut AppSystemInfo) {

    let chuck = Layout::default()
        .vertical_margin(3)
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(area);

    fn create_sparkline(data: &Vec<u64>, color: Color) -> Sparkline {
         Sparkline::default()
            .block(Block::default().borders(Borders::NONE), )
            .data(data)
            .style(Style::default().fg(color))
    }

    let data = &app_sys.network_info;
    let received_sparkline = create_sparkline(&data.received_list, Color::Green);
    frame.render_widget(received_sparkline, chuck[0]);

    let transmitted_sparkline = create_sparkline(&data.transmitted_list, Color::Blue);
    frame.render_widget(transmitted_sparkline, chuck[1]);
}


fn render_process_table<B: Backend>(frame: &mut Frame<B>, area: Rect, app_sys: &mut AppSystemInfo, state: &mut TableState) {
    let (process_column, process_data) = app_sys.get_process_info();

    let header_cells = process_column
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().add_modifier(Modifier::BOLD)));
    let header = Row::new(header_cells)
        .style(Style::default().bg(Color::Green))
        .height(1)
        .bottom_margin(1);
    let rows = process_data.iter().map(|item| {
        let cells = item.iter().map(|c| Cell::from(c.as_str()));
        Row::new(cells).height(1_u16).bottom_margin(1)
    });

    let table = Table::new(rows)
        .header(header)
        .block(Block::default().borders(Borders::ALL).title("Table"))
        .widths(&[
            Constraint::Length(10),
            Constraint::Length(10),
            Constraint::Length(10),
            Constraint::Length(10),
            Constraint::Length(10),
            Constraint::Length(10),
            Constraint::Length(10),
            Constraint::Length(10),
            Constraint::Length(10),
            Constraint::Percentage(30),
        ]);
    frame.render_stateful_widget(table, area, state);
}


fn render_content<B: Backend>(frame: &mut Frame<B>, area: Rect,  app_sys: &mut AppSystemInfo, state: &mut TableState) {
    let right_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(30), Constraint::Percentage(70)].as_ref())
        .split(area);


    let top_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(right_chunks[0]);

    render_usage_info(frame, top_chunks[0], app_sys);

    render_network_info(frame, top_chunks[1], app_sys);


    // frame.render_widget(
    //     Block::default()
    //         .title_alignment(Alignment::Center)
    //         .title("    Process Info    ")
    //         .borders(Borders::ALL)
    //         .border_type(BorderType::Plain),
    //     right_chunks[1]
    // );
    render_process_table(frame, right_chunks[1], app_sys, state);



}


fn render_sider<B: Backend>(frame: &mut Frame<'_, B>, area: Rect, app_sys: &AppSystemInfo) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(3)
        .constraints([
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25)
        ].as_ref())
        .split(area);

    // 网络信息
    let memory_info = &app_sys.network_info;
    let mut memory_text = vec![
        ListItem::new(vec![
            Line::from(vec![
                Span::styled(format!("{:<12}", "Network"), Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" "),
                Span::styled(format!("{:<12}", "R/s"), Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" "),
                Span::styled(format!("{:<8}", "W/s"), Style::default().add_modifier(Modifier::BOLD))
            ])
        ])
    ];

    for (name, item) in &memory_info.detail {
        let list_item = ListItem::new(vec![
            Line::from(vec![
                Span::styled(format!("{:<12}", name), Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" "),
                Span::styled(format!("{:<12}", Size::convent_from(item[0]).format()), Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" "),
                Span::styled(format!("{:<8}", Size::convent_from(item[1]).format()), Style::default().add_modifier(Modifier::BOLD))
            ])
        ]);
        memory_text.push(list_item)
    }

    let list = List::new(memory_text).block(Block::default().borders(Borders::NONE));
    frame.render_widget(list, chunks[1])

}


/// Renders the user interface widgets.
pub fn render<B: Backend>(app: &mut App, frame: &mut Frame<'_, B>) {

    app.app_sys_info.refresh_all();

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .margin(1)
        .constraints([Constraint::Percentage(20), Constraint::Percentage(80)].as_ref())
        .split(frame.size());

    // frame.render_widget(
    //     Block::default()
    //         .title_alignment(Alignment::Center)
    //         .title("    Sider    ")
    //         .borders(Borders::ALL)
    //         .border_type(BorderType::Plain),
    //     chunks[0]
    // );

    render_sider(frame, chunks[0], &app.app_sys_info);

    render_content(frame, chunks[1],&mut app.app_sys_info, &mut app.state)

}