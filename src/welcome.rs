//! Static welcome art; shell launch and command execution live in shell.rs.
use std::io::IsTerminal;

const LOGO: [[&str; 5]; 9] = [
    ["#### ", "#   #", "#### ", "#  # ", "#   #"], // R
    ["#####", "#    ", "#### ", "#    ", "#####"], // E
    ["#### ", "#   #", "#### ", "#    ", "#    "], // P
    [" ### ", "#   #", "#   #", "#   #", " ### "], // O
    ["#### ", "#   #", "#### ", "#    ", "#    "], // P
    ["#####", "  #  ", "  #  ", "  #  ", "#####"], // I
    ["#    ", "#    ", "#    ", "#    ", "#####"], // L
    [" ### ", "#   #", "#   #", "#   #", " ### "], // O
    ["#####", "  #  ", "  #  ", "  #  ", "  #  "], // T
];
const COMMANDS: [(&str, &str); 5] = [
    ("claude / codex", "Run an installed AI coding tool"),
    ("repopilot doctor .", "Inspect this project locally"),
    ("repopilot chat", "Chat with your saved local model"),
    ("repopilot guide", "Browse offline workflow guides"),
    ("repopilot --help", "Explore all commands"),
];

pub fn render(terminal: bool) -> String {
    let color = terminal
        && std::env::var_os("NO_COLOR").is_none()
        && std::env::var("TERM").is_ok_and(|v| v != "dumb");
    let columns = std::env::var("COLUMNS")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(80);
    screen(color, columns)
}

fn wrapped(out: &mut String, text: &str, width: usize) {
    let mut line = String::from("  ");
    for word in text.split_whitespace() {
        if line.len() > 2 && line.len() + 1 + word.len() > width {
            out.push_str(&line);
            out.push('\n');
            line = String::from("  ");
        }
        if line.len() > 2 {
            line.push(' ');
        }
        line.push_str(word);
    }
    out.push_str(&line);
    out.push('\n');
}

fn screen(color: bool, columns: usize) -> String {
    // Bound allocation even when an inherited COLUMNS value is malformed or huge.
    let width = columns.clamp(32, 80);
    let (accent, reset) = if color {
        ("\x1b[1;36m", "\x1b[0m")
    } else {
        ("", "")
    };
    let mut out = String::new();
    if width >= 56 {
        for row in 0..5 {
            let shade = if color {
                [
                    "\x1b[1;96m",
                    "\x1b[1;96m",
                    "\x1b[1;36m",
                    "\x1b[1;36m",
                    "\x1b[1;34m",
                ][row]
            } else {
                ""
            };
            let art = LOGO
                .iter()
                .map(|letter| letter[row])
                .collect::<Vec<_>>()
                .join(" ");
            out.push_str(&format!("  {shade}{art}{reset}\n"));
        }
    }
    out.push_str(&format!(
        "  {accent}REPOPILOT{reset} / v{}\n",
        env!("CARGO_PKG_VERSION")
    ));
    wrapped(&mut out, "Your shell. Your tools. Your workflow.", width);
    out.push('\n');
    let inner = width - 6;
    let border = format!("  {accent}+{}+{reset}\n", "-".repeat(inner));
    out.push_str(&border);
    let box_row = |out: &mut String, text: &str| {
        out.push_str(&format!(
            "  {accent}|{reset} {text:<padding$} {accent}|{reset}\n",
            padding = inner - 2
        ));
    };
    box_row(&mut out, "QUICK START / HOW TO USE");
    for (command, description) in COMMANDS {
        if width >= 74 {
            box_row(&mut out, &format!("{command:<22} {description}"));
        } else {
            box_row(&mut out, command);
        }
    }
    out.push_str(&border);
    out.push_str(&format!("\n  {accent}WORKFLOWS TO TRY{reset}\n"));
    wrapped(&mut out, "repopilot guide init", width);
    wrapped(&mut out, "repopilot guide \"create a branch\"", width);
    wrapped(&mut out, "repopilot guide \"undo last commit\"", width);
    out.push('\n');
    wrapped(
        &mut out,
        "RepoPilot shell: installed commands run normally.",
        width,
    );
    wrapped(
        &mut out,
        "Guides explain commands; you choose what to run.",
        width,
    );
    wrapped(
        &mut out,
        "Local AI chat is separate. Type exit to leave the shell.",
        width,
    );
    out
}

pub fn stdout_is_terminal() -> bool {
    std::io::stdout().is_terminal()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn color_is_explicit_and_plain_layouts_fit() {
        let colored = screen(true, 80);
        assert!(colored.contains("\x1b[1;96m"));
        assert!(colored.contains("\x1b[1;34m"));
        for width in [32, 40, 55, 56, 73, 74, 80, usize::MAX] {
            let plain = screen(false, width);
            assert!(!plain.contains('\u{1b}'));
            assert!(
                plain.lines().all(|line| line.len() <= width.clamp(32, 80)),
                "width {width}: {plain}"
            );
        }
    }
}
