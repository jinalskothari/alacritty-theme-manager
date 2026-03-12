use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, List, ListItem, Paragraph},
    Frame,
};

use crate::app::App;

pub fn draw(f: &mut Frame, app: &App) {
    let area = f.area();

    let v_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(3), Constraint::Length(3)])
        .split(area);

    let h_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
        .split(v_chunks[0]);

    // ── left: theme list ──────────────────────────────────────────────────────
    let items: Vec<ListItem> = app
        .themes
        .iter()
        .map(|t| ListItem::new(Line::from(Span::raw(t.clone()))))
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .title(" Themes ")
                .title_alignment(Alignment::Center)
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::Indexed(4))),
        )
        .highlight_style(
            Style::default()
                .fg(Color::Indexed(0))
                .bg(Color::Indexed(4))
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("> ");

    f.render_stateful_widget(list, h_chunks[0], &mut app.list_state.clone());

    // ── right: preview ────────────────────────────────────────────────────────
    let preview = Paragraph::new(preview_lines()).block(
        Block::default()
            .title(" Preview ")
            .title_alignment(Alignment::Center)
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::Indexed(4))),
    );

    f.render_widget(preview, h_chunks[1]);

    // ── footer ────────────────────────────────────────────────────────────────
    let footer_line = if let Some(msg) = &app.status_msg {
        // Show git status; colour green for success, red for errors
        let (text, color) = if msg.starts_with("git error") || msg.starts_with("failed") {
            (msg.as_str(), 1u8)
        } else {
            (msg.as_str(), 2u8)
        };
        Line::from(Span::styled(text, Style::default().fg(Color::Indexed(color))))
    } else {
        let mut spans = vec![
            Span::styled(" ↑↓ jk ", Style::default().fg(Color::Indexed(3))),
            Span::raw("navigate   "),
            Span::styled("Enter ", Style::default().fg(Color::Indexed(10))),
            Span::raw("keep   "),
            Span::styled("Esc/q ", Style::default().fg(Color::Indexed(1))),
            Span::raw("restore & exit"),
        ];
        if app.is_git_repo {
            spans.push(Span::raw("   "));
            spans.push(Span::styled("u ", Style::default().fg(Color::Indexed(14))));
            spans.push(Span::raw("update themes"));
        }
        Line::from(spans)
    };

    let footer = Paragraph::new(footer_line)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::Indexed(8))),
        )
        .alignment(Alignment::Center);

    f.render_widget(footer, v_chunks[1]);
}

fn preview_lines() -> Vec<Line<'static>> {
    fn c(text: &'static str, idx: u8) -> Span<'static> {
        Span::styled(text, Style::default().fg(Color::Indexed(idx)))
    }
    fn b(text: &'static str, idx: u8) -> Span<'static> {
        Span::styled(
            text,
            Style::default()
                .fg(Color::Indexed(idx))
                .add_modifier(Modifier::BOLD),
        )
    }
    fn p(text: &'static str) -> Span<'static> {
        Span::raw(text)
    }

    // Each name padded to 7 chars (length of "magenta") for right-column alignment.
    // pad = 7 - len(name)
    fn pal_row(
        ni: u8, nl: &'static str, nl_pad: &'static str,
        bi: u8, bl: &'static str,
    ) -> Line<'static> {
        Line::from(vec![
            p("  "),
            c("██", ni), p(" "), c(nl, ni), p(nl_pad),
            p("    "),
            c("██", bi), p(" "), c(bl, bi),
        ])
    }

    vec![
        // ── palette ──────────────────────────────────────────────────────────
        Line::from(c("  ── palette ─────────────────────────────────────────────", 8)),
        //              name      pad(7-len)   bright idx  bright name
        pal_row(0, "black",   "  ",  8,  "bright black"),   // 5+2=7
        pal_row(1, "red",     "    ", 9,  "bright red"),    // 3+4=7
        pal_row(2, "green",   "  ",  10, "bright green"),   // 5+2=7
        pal_row(3, "yellow",  " ",   11, "bright yellow"),  // 6+1=7
        pal_row(4, "blue",    "   ", 12, "bright blue"),    // 4+3=7
        pal_row(5, "magenta", "",    13, "bright magenta"), // 7+0=7
        pal_row(6, "cyan",    "   ", 14, "bright cyan"),    // 4+3=7
        pal_row(7, "white",   "  ",  15, "bright white"),   // 5+2=7
        Line::from(""),

        // ── code ─────────────────────────────────────────────────────────────
        Line::from(c("  ── code ────────────────────────────────────────────────", 8)),
        Line::from(vec![
            p("  "), c("use ", 4), p("tokio::net::"), c("TcpListener", 13), p(";"),
            p("  "), c("use ", 4), p("std::io::"), c("Result", 13), p(";"),
        ]),
        Line::from(""),
        Line::from(vec![
            p("  "), c("struct ", 4), c("Server", 13), p(" {"),
            p("  "), c("// holds runtime config", 8),
        ]),
        Line::from(vec![p("      host: "), c("String", 13), p(",")]),
        Line::from(vec![
            p("      port: "), c("u16", 13), p(","),
            p("        "), c("// default: ", 8), c("8080", 3),
        ]),
        Line::from(p("  }")),
        Line::from(""),
        Line::from(vec![
            p("  "), c("async fn ", 4), c("connect", 12),
            p("(srv: "), c("Server", 13), p(") -> "), c("Result", 13), p("<()> {"),
        ]),
        Line::from(vec![
            p("      "), c("let ", 4), p("addr = "),
            c("format!", 6), p("("), c("\"{}:{}\"", 2),
            p(", srv.host, srv."), c("port", 14), p(");"),
        ]),
        Line::from(vec![
            p("      "), c("println!", 6), p("("), c("\"listening on {addr}\"", 2), p(");"),
        ]),
        Line::from(vec![p("      "), c("Ok", 10), p("(())")]),
        Line::from(p("  }")),
        Line::from(""),

        // ── terminal ─────────────────────────────────────────────────────────
        Line::from(c("  ── terminal ────────────────────────────────────────────", 8)),
        Line::from(vec![
            c("  ~/projects/server", 12), p(" "),
            c("(main)", 11), p(" $ cargo build"),
        ]),
        Line::from(vec![
            p("       "), c("Compiling", 10), p(" server v"), c("0.1.0", 3),
        ]),
        Line::from(vec![
            p("  "), c("warning", 11), p(": unused variable `"), c("x", 3), p("`"),
        ]),
        Line::from(vec![
            p("  "), c("error", 9), p("[E0502]: cannot borrow `data` as mutable"),
        ]),
        Line::from(vec![
            p("       "), c("Finished", 2), p(" dev [unoptimized] in "), c("1.24", 3), p("s"),
        ]),
        Line::from(""),
        Line::from(vec![
            p("  "),
            b("[✓]", 10), p(" tests passed   "),
            b("[!]", 11), p(" 1 warning   "),
            b("[✗]", 9),  p(" 2 errors"),
        ]),
    ]
}
