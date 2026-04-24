use std::io;
use std::time::{Duration, Instant};

use anyhow::Result;
use crossterm::event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyEventKind};
use crossterm::execute;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use ratatui::backend::{Backend, CrosstermBackend};
use ratatui::Terminal;

use crate::app::App;
use crate::cli::args::Args;
use crate::ui::ui;

pub async fn run(tick_rate: Duration, _enhanced_graphics: bool, args: Args) -> Result<()> {
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // create app and run it
    let mut app = App::new(args).await.unwrap();

    let app_result = run_app(&mut terminal, &mut app, tick_rate).await;

    // restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
    terminal.show_cursor()?;

    if let Err(err) = app_result {
        println!("{err:?}");
    }

    Ok(())
}

async fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
    tick_rate: Duration,
) -> Result<()> {
    let mut last_tick = Instant::now();
    loop {
        // 检查应用是否需要退出
        if app.should_quit {
            return Ok(());
        }

        // 接收服务端消息
        app.receive_message();

        // 绘制界面
        terminal.draw(|frame| ui::draw(frame, app))?;

        // 处理输入事件
        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    app.handle_key_event(key.code, key.modifiers)?;

                    if app.should_quit {
                        return Ok(());
                    }
                }
            }
        }

        if last_tick.elapsed() >= tick_rate {
            // 处理定时任务
            last_tick = Instant::now();
        }
    }
}
