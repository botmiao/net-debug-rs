use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::Span,
    widgets::Paragraph,
    Frame,
};

use crate::app::App;

/// 状态栏组件
pub struct StatusBar {
    // 状态信息
}

impl Default for StatusBar {
    fn default() -> Self {
        Self {}
    }
}

impl StatusBar {
    /// 绘制顶部状态栏
    pub fn draw_top_bar(&self, frame: &mut Frame, area: Rect, app: &App) {
        let status_text = format!(
            " Sent: {} bytes | Received: {} bytes | Status: {} ",
            app.stats.sent_bytes,
            app.stats.received_bytes,
            if app.stats.connected {
                "Connected"
            } else {
                "Disconnected"
            }
        );

        let status_widget = Paragraph::new(Span::styled(
            status_text,
            Style::default().fg(Color::Black).bg(Color::LightCyan),
        ));

        frame.render_widget(status_widget, area);
    }

    /// 绘制底部状态栏 (快捷键提示)
    pub fn draw_bottom_bar(&self, frame: &mut Frame, area: Rect) {
        let help_text = " Ctrl+C: Quit | I: Input | Tab: Switch Recv | Shift+←/→: Switch Send ";

        let help_widget = Paragraph::new(Span::styled(
            help_text,
            Style::default().fg(Color::Black).bg(Color::LightCyan),
        ));

        frame.render_widget(help_widget, area);
    }
}
