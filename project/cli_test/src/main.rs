use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Terminal,
};
use std::io;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 设置终端
    let stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    enable_raw_mode()?;

    // 组件状态
    let mut checkbox_states = vec![false, false, false]; // 复选框状态
    let mut input_1 = String::new(); // 输入框 1
    let mut input_2 = String::new(); // 输入框 2
    let mut active_input = 0; // 当前激活的输入框
    let mut selected_button = 0; // 当前选中的按钮
    let buttons = vec!["OK", "Cancel", "Apply"];

    loop {
        // 渲染界面
        terminal.draw(|f| {
            // 主布局
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints(
                    [
                        Constraint::Percentage(30), // 复选框
                        Constraint::Percentage(40), // 输入框
                        Constraint::Percentage(30), // 按钮
                    ]
                    .as_ref(),
                )
                .split(f.area());

            // 复选框区域
            let checkbox_items: Vec<ListItem> = checkbox_states
                .iter()
                .enumerate()
                .map(|(i, &checked)| {
                    let checkbox_text = if checked {
                        format!("[X] Checkbox {}", i + 1)
                    } else {
                        format!("[ ] Checkbox {}", i + 1)
                    };
                    // 使用 Line::from
                    ListItem::new(Line::from(Span::raw(checkbox_text)))
                })
                .collect();

            let checkbox_list = List::new(checkbox_items)
                .block(Block::default().borders(Borders::ALL).title("Checkboxes"));
            f.render_widget(checkbox_list, chunks[0]);

            // 输入框区域
            let input_area = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                .split(chunks[1]);

            let input_1_widget = Paragraph::new(input_1.clone())
                .block(Block::default().borders(Borders::ALL).title("Input 1"))
                .style(if active_input == 0 {
                    Style::default().fg(Color::Yellow)
                } else {
                    Style::default()
                });
            let input_2_widget = Paragraph::new(input_2.clone())
                .block(Block::default().borders(Borders::ALL).title("Input 2"))
                .style(if active_input == 1 {
                    Style::default().fg(Color::Yellow)
                } else {
                    Style::default()
                });

            f.render_widget(input_1_widget, input_area[0]);
            f.render_widget(input_2_widget, input_area[1]);

            // 按钮区域
            let button_lines: Vec<Line> = buttons
                .iter()
                .enumerate()
                .map(|(i, &btn)| {
                    if i == selected_button {
                        Line::from(Span::styled(
                            format!("[{}] ", btn),
                            Style::default().fg(Color::Yellow),
                        ))
                    } else {
                        Line::from(Span::raw(format!("[{}] ", btn)))
                    }
                })
                .collect();

            let button_widget = Paragraph::new(Text::from(button_lines))
                .block(Block::default().borders(Borders::ALL).title("Buttons"));
        })?;

        // 事件处理
        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') => break, // 退出
                KeyCode::Tab => {
                    // 切换激活区域：复选框 -> 输入框 -> 按钮
                    active_input = (active_input + 1) % 3;
                }
                KeyCode::Char(' ') => {
                    // 切换复选框状态
                    if active_input == 0 {
                        checkbox_states[selected_button] = !checkbox_states[selected_button];
                    }
                }
                KeyCode::Down => {
                    if active_input == 0 && selected_button < checkbox_states.len() - 1 {
                        selected_button += 1;
                    } else if active_input == 1 {
                        active_input = 2;
                    }
                }
                KeyCode::Up => {
                    if active_input == 0 && selected_button > 0 {
                        selected_button -= 1;
                    } else if active_input == 2 {
                        active_input = 1;
                    }
                }
                KeyCode::Left => {
                    if active_input == 2 && selected_button > 0 {
                        selected_button -= 1;
                    }
                }
                KeyCode::Right => {
                    if active_input == 2 && selected_button < buttons.len() - 1 {
                        selected_button += 1;
                    }
                }
                KeyCode::Enter => {
                    if active_input == 2 {
                        println!("Button {} pressed!", buttons[selected_button]);
                    }
                }
                KeyCode::Char(c) => {
                    if active_input == 1 {
                        input_1.push(c);
                    } else if active_input == 2 {
                        input_2.push(c);
                    }
                }
                KeyCode::Backspace => {
                    if active_input == 1 && !input_1.is_empty() {
                        input_1.pop();
                    } else if active_input == 2 && !input_2.is_empty() {
                        input_2.pop();
                    }
                }
                _ => {}
            }
        }
    }

    disable_raw_mode()?;
    Ok(())
}
