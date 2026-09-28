use crate::config::{AppConfig, expand_tilde};
use crate::view::tui::{Color, Rect, ScreenBuffer, Style};
use std::path::PathBuf;
use std::sync::LazyLock;

macro_rules! load_ascii {
    ($path:expr) => {
        LazyLock::new(|| include_str!($path).lines().collect::<Vec<&'static str>>())
    };
}

static ARCH_ASCII: LazyLock<Vec<&'static str>> = load_ascii!("../../assets/ascii/arch.txt");
static ARCH_ASCII_COMPACT: LazyLock<Vec<&'static str>> = load_ascii!("../../assets/ascii/arch_compact.txt");

static UBUNTU_ASCII: LazyLock<Vec<&'static str>> = load_ascii!("../../assets/ascii/ubuntu.txt");
static UBUNTU_ASCII_COMPACT: LazyLock<Vec<&'static str>> = load_ascii!("../../assets/ascii/ubuntu_compact.txt");

static DEBIAN_ASCII: LazyLock<Vec<&'static str>> = load_ascii!("../../assets/ascii/debian.txt");
static DEBIAN_ASCII_COMPACT: LazyLock<Vec<&'static str>> = load_ascii!("../../assets/ascii/debian_compact.txt");

static FEDORA_ASCII: LazyLock<Vec<&'static str>> = load_ascii!("../../assets/ascii/fedora.txt");
static FEDORA_ASCII_COMPACT: LazyLock<Vec<&'static str>> = load_ascii!("../../assets/ascii/fedora_compact.txt");

static VOID_ASCII: LazyLock<Vec<&'static str>> = load_ascii!("../../assets/ascii/void.txt");
static VOID_ASCII_COMPACT: LazyLock<Vec<&'static str>> = load_ascii!("../../assets/ascii/void_compact.txt");

static LINUX_ASCII: LazyLock<Vec<&'static str>> = load_ascii!("../../assets/ascii/linux.txt");
static LINUX_ASCII_COMPACT: LazyLock<Vec<&'static str>> = load_ascii!("../../assets/ascii/linux_compact.txt");

static MACOS_ASCII: LazyLock<Vec<&'static str>> = load_ascii!("../../assets/ascii/macos.txt");
static MACOS_ASCII_COMPACT: LazyLock<Vec<&'static str>> = load_ascii!("../../assets/ascii/macos_compact.txt");

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AsciiMode {
    Full,
    Compact,
    None,
}

pub struct ResolvedAscii {
    pub title: String,
    pub lines: Vec<String>,
    pub palette: Vec<Color>,
    pub mode: AsciiMode,
}

pub fn resolve_ascii(
    config: &AppConfig,
    is_wide: bool,
    term_height: usize,
) -> ResolvedAscii {
    // 1. Resolve Mode
    let mode = if let Some(m_str) = config.logo.as_ref().and_then(|l| l.mode.as_ref()) {
        match m_str.trim().to_lowercase().as_str() {
            "full" => AsciiMode::Full,
            "compact" => AsciiMode::Compact,
            "none" | "off" | "disable" => AsciiMode::None,
            _ => adaptive_mode(is_wide, term_height),
        }
    } else {
        adaptive_mode(is_wide, term_height)
    };

    // 2. Custom Source (file path) check
    if let Some(src) = config.logo.as_ref().and_then(|l| l.source.as_ref()) {
        let expanded = expand_tilde(src);
        if let Ok(content) = std::fs::read_to_string(&expanded) {
            let lines: Vec<String> = if mode == AsciiMode::None {
                Vec::new()
            } else {
                content.lines().map(String::from).collect()
            };
            let title = expanded
                .file_stem()
                .map(|s| format!(" 󰌽 {} ", s.to_string_lossy()))
                .unwrap_or_else(|| " 󰌽 Custom ".to_string());
            let palette = resolve_palette(config, &[Color::Cyan, Color::Blue]);
            return ResolvedAscii {
                title,
                lines,
                palette,
                mode,
            };
        }
    }

    // 3. Resolve distro name / key
    let os_name = if let Some(src) = config.logo.as_ref().and_then(|l| l.source.as_ref()) {
        if src.eq_ignore_ascii_case("auto") {
            detect_distro_id()
        } else {
            src.to_lowercase()
        }
    } else {
        detect_distro_id()
    };

    let (distro_key, title, default_palette, full_built, compact_built) = if os_name.contains("arch") {
        ("arch", "  Arch Linux ", vec![Color::Cyan, Color::Blue], &ARCH_ASCII[..], &ARCH_ASCII_COMPACT[..])
    } else if os_name.contains("ubuntu") {
        ("ubuntu", "  Ubuntu ", vec![Color::Yellow, Color::Red], &UBUNTU_ASCII[..], &UBUNTU_ASCII_COMPACT[..])
    } else if os_name.contains("debian") {
        ("debian", "  Debian ", vec![Color::Red, Color::White], &DEBIAN_ASCII[..], &DEBIAN_ASCII_COMPACT[..])
    } else if os_name.contains("fedora") {
        ("fedora", "  Fedora ", vec![Color::Blue, Color::White], &FEDORA_ASCII[..], &FEDORA_ASCII_COMPACT[..])
    } else if os_name.contains("void") {
        ("void", "  Void Linux ", vec![Color::Green, Color::White], &VOID_ASCII[..], &VOID_ASCII_COMPACT[..])
    } else if os_name.contains("darwin") || os_name.contains("mac") {
        ("macos", "  macOS ", vec![Color::White, Color::DarkGrey], &MACOS_ASCII[..], &MACOS_ASCII_COMPACT[..])
    } else {
        ("linux", " 󰌽 Linux ", vec![Color::Cyan, Color::Yellow, Color::White], &LINUX_ASCII[..], &LINUX_ASCII_COMPACT[..])
    };

    let palette = resolve_palette(config, &default_palette);

    // 4. Check ~/.config/easyfetch/ascii/<distro>.txt override
    if mode != AsciiMode::None {
        let home = std::env::var("HOME").unwrap_or_default();
        if !home.is_empty() {
            let file_name = if mode == AsciiMode::Compact {
                format!("{}_compact.txt", distro_key)
            } else {
                format!("{}.txt", distro_key)
            };
            let user_file = PathBuf::from(home).join(".config/easyfetch/ascii").join(file_name);
            if let Ok(content) = std::fs::read_to_string(user_file) {
                let lines: Vec<String> = content.lines().map(String::from).collect();
                return ResolvedAscii {
                    title: title.to_string(),
                    lines,
                    palette,
                    mode,
                };
            }
        }
    }

    // 5. Built-in embedded fallback
    let lines: Vec<String> = match mode {
        AsciiMode::Full => full_built.iter().map(|s| s.to_string()).collect(),
        AsciiMode::Compact => compact_built.iter().map(|s| s.to_string()).collect(),
        AsciiMode::None => Vec::new(),
    };

    ResolvedAscii {
        title: title.to_string(),
        lines,
        palette,
        mode,
    }
}

fn adaptive_mode(is_wide: bool, term_height: usize) -> AsciiMode {
    if is_wide {
        if term_height >= 26 {
            AsciiMode::Full
        } else if term_height >= 18 {
            AsciiMode::Compact
        } else {
            AsciiMode::None
        }
    } else if term_height >= 32 {
        AsciiMode::Full
    } else if term_height >= 20 {
        AsciiMode::Compact
    } else {
        AsciiMode::None
    }
}

pub fn detect_distro_id() -> String {
    if let Ok(content) = std::fs::read_to_string("/etc/os-release") {
        for line in content.lines() {
            if let Some(id) = line.strip_prefix("ID=") {
                return id.trim_matches('"').to_lowercase();
            }
        }
    }
    if let Ok(ostype) = std::fs::read_to_string("/proc/sys/kernel/ostype") {
        return ostype.trim().to_lowercase();
    }
    "linux".to_string()
}

fn resolve_palette(config: &AppConfig, default_colors: &[Color]) -> Vec<Color> {
    if let Some(color_strs) = config.logo.as_ref().and_then(|l| l.colors.as_ref()) {
        let parsed: Vec<Color> = color_strs.iter().filter_map(|s| Color::parse(s)).collect();
        if !parsed.is_empty() {
            return parsed;
        }
    }
    default_colors.to_vec()
}

pub fn visible_art_len(line: &str) -> usize {
    let mut count = 0;
    let mut chars = line.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '$' {
            match chars.peek() {
                Some(next) if next.is_ascii_digit() => {
                    chars.next();
                    continue;
                }
                Some('$') => {
                    chars.next();
                    count += 1;
                    continue;
                }
                _ => {}
            }
        }
        count += 1;
    }
    count
}

pub fn put_art_line(
    buffer: &mut ScreenBuffer,
    x: usize,
    y: usize,
    line: &str,
    palette: &[Color],
    max_width: usize,
) -> usize {
    let mut current_color = palette.first().copied().unwrap_or(Color::Cyan);
    let mut drawn = 0;
    let mut chars = line.chars().peekable();

    while let Some(ch) = chars.next() {
        if drawn >= max_width {
            break;
        }
        if ch == '$' {
            match chars.peek() {
                Some(next) if next.is_ascii_digit() => {
                    let digit = next.to_digit(10).unwrap_or(0);
                    chars.next();
                    let idx = (digit as usize).saturating_sub(1);
                    current_color = palette.get(idx).copied().unwrap_or(Color::White);
                    continue;
                }
                Some('$') => {
                    chars.next();
                    buffer.put_char(x + drawn, y, '$', Style::fg(current_color).bold());
                    drawn += 1;
                    continue;
                }
                _ => {}
            }
        }
        buffer.put_char(x + drawn, y, ch, Style::fg(current_color).bold());
        drawn += 1;
    }
    drawn
}

pub fn draw_ascii_box(
    buffer: &mut ScreenBuffer,
    area: Rect,
    title: &str,
    lines: &[String],
    palette: &[Color],
) -> usize {
    if area.width < 10 || area.height < 4 || lines.is_empty() {
        return area.y;
    }

    let border_style = Style::fg(Color::DarkGrey);
    let title_style = Style::fg(Color::Cyan).bold();

    let inner_width = area.width.saturating_sub(2);
    let start_x = area.x;
    let start_y = area.y;

    // Top border: ╭── Arch Linux ────╮
    buffer.put_char(start_x, start_y, '╭', border_style);

    let title_len = title.chars().count();
    if !title.is_empty() && inner_width > title_len + 4 {
        let left_pad = 2;
        for i in 0..left_pad {
            buffer.put_char(start_x + 1 + i, start_y, '─', border_style);
        }
        buffer.put_str(
            start_x + 1 + left_pad,
            start_y,
            title,
            title_style,
            inner_width.saturating_sub(left_pad),
        );
        let right_start = 1 + left_pad + title_len;
        for x in right_start..area.width - 1 {
            buffer.put_char(start_x + x, start_y, '─', border_style);
        }
    } else {
        for x in 1..area.width - 1 {
            buffer.put_char(start_x + x, start_y, '─', border_style);
        }
    }
    buffer.put_char(start_x + area.width - 1, start_y, '╮', border_style);

    let art_height = lines.len();
    let content_height = area.height.saturating_sub(2);
    let v_padding = content_height.saturating_sub(art_height) / 2;

    let mut current_y = start_y + 1;
    let max_y = area.y + area.height - 1;

    for row in 0..content_height {
        if current_y >= max_y {
            break;
        }

        buffer.put_char(start_x, current_y, '│', border_style);

        if row >= v_padding && row < v_padding + art_height {
            let art_idx = row - v_padding;
            let art_line = &lines[art_idx];
            let art_len = visible_art_len(art_line);
            let h_padding = inner_width.saturating_sub(art_len) / 2;

            put_art_line(
                buffer,
                start_x + 1 + h_padding,
                current_y,
                art_line,
                palette,
                inner_width.saturating_sub(h_padding),
            );
        }

        buffer.put_char(start_x + area.width - 1, current_y, '│', border_style);
        current_y += 1;
    }

    // Bottom border: ╰────────────╯
    buffer.put_char(start_x, current_y, '╰', border_style);
    for x in 1..area.width - 1 {
        buffer.put_char(start_x + x, current_y, '─', border_style);
    }
    buffer.put_char(start_x + area.width - 1, current_y, '╯', border_style);

    current_y + 1
}

pub fn draw_sys_ascii_box(
    buffer: &mut ScreenBuffer,
    area: Rect,
    title: &str,
    lines: &[String],
    palette: &[Color],
    sys_rows: &[crate::view::tui::TuiRow],
) -> usize {
    if area.width < 10 || area.height < 3 {
        return area.y;
    }

    let border_style = Style::fg(Color::DarkGrey);
    let title_style = Style::fg(Color::Cyan).bold();
    let label_style = Style::fg(Color::Green);
    let val_style = Style::fg(Color::Cyan);

    let inner_width = area.width.saturating_sub(2);
    let start_x = area.x;
    let start_y = area.y;

    // Top border: ╭── Title ────╮
    buffer.put_char(start_x, start_y, '╭', border_style);

    let title_len = title.chars().count();
    if !title.is_empty() && inner_width > title_len + 4 {
        let left_pad = 2;
        for i in 0..left_pad {
            buffer.put_char(start_x + 1 + i, start_y, '─', border_style);
        }
        buffer.put_str(
            start_x + 1 + left_pad,
            start_y,
            title,
            title_style,
            inner_width.saturating_sub(left_pad),
        );
        let right_start = 1 + left_pad + title_len;
        for x in right_start..area.width - 1 {
            buffer.put_char(start_x + x, start_y, '─', border_style);
        }
    } else {
        for x in 1..area.width - 1 {
            buffer.put_char(start_x + x, start_y, '─', border_style);
        }
    }
    buffer.put_char(start_x + area.width - 1, start_y, '╮', border_style);

    let max_art_w = lines.iter().map(|l| visible_art_len(l)).max().unwrap_or(0);
    let has_art = !lines.is_empty() && inner_width > max_art_w + 24;

    let art_x = start_x + 3;
    let sys_start_x = if has_art { art_x + max_art_w + 3 } else { start_x + 2 };
    let sys_width = (start_x + area.width - 2).saturating_sub(sys_start_x);

    let num_rows = sys_rows.len().max(if has_art { lines.len() } else { 0 });
    let content_height = area.height.saturating_sub(2).min(num_rows);

    let art_v_pad = content_height.saturating_sub(lines.len()) / 2;
    let sys_v_pad = content_height.saturating_sub(sys_rows.len()) / 2;

    let mut current_y = start_y + 1;
    let max_y = area.y + area.height - 1;

    let sys_max_label_len = sys_rows
        .iter()
        .map(|r| r.label.chars().count())
        .max()
        .unwrap_or(0);

    for row_idx in 0..content_height {
        if current_y >= max_y {
            break;
        }

        buffer.put_char(start_x, current_y, '│', border_style);

        // Draw art line (vertically centered) with token colors
        if has_art && row_idx >= art_v_pad && row_idx < art_v_pad + lines.len() {
            let art_line = &lines[row_idx - art_v_pad];
            put_art_line(buffer, art_x, current_y, art_line, palette, max_art_w);
        }

        // Draw sys row (vertically centered) with Fastfetch-style close alignment
        if row_idx >= sys_v_pad && row_idx < sys_v_pad + sys_rows.len() {
            let row = &sys_rows[row_idx - sys_v_pad];
            let label_len = row.label.chars().count();
            let val_len = row.value.chars().count();
            let val_col_x = sys_start_x + sys_max_label_len + 2;

            if val_col_x + val_len <= sys_start_x + sys_width {
                buffer.put_str(sys_start_x, current_y, &row.label, label_style, label_len);
                buffer.put_str(val_col_x, current_y, &row.value, val_style, val_len);
            } else if label_len + 1 + val_len <= sys_width {
                buffer.put_str(sys_start_x, current_y, &row.label, label_style, label_len);
                buffer.put_str(sys_start_x + label_len + 1, current_y, &row.value, val_style, val_len);
            } else {
                buffer.put_str(sys_start_x, current_y, &row.label, label_style, label_len);
                let allowed = sys_width.saturating_sub(label_len + 4);
                if allowed > 0 {
                    let truncated: String = row.value.chars().take(allowed).collect();
                    buffer.put_str(sys_start_x + label_len + 1, current_y, &truncated, val_style, allowed);
                    buffer.put_str(sys_start_x + label_len + 1 + allowed, current_y, "...", val_style, 3);
                }
            }
        }

        buffer.put_char(start_x + area.width - 1, current_y, '│', border_style);
        current_y += 1;
    }

    buffer.put_char(start_x, current_y, '╰', border_style);
    for x in 1..area.width - 1 {
        buffer.put_char(start_x + x, current_y, '─', border_style);
    }
    buffer.put_char(start_x + area.width - 1, current_y, '╯', border_style);

    current_y + 1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_visible_art_len() {
        assert_eq!(visible_art_len("$1/\\$2_"), 3);
        assert_eq!(visible_art_len("$$100"), 4); // $$ is literal '$', then '100'
        assert_eq!(visible_art_len("Hello $1World$2!"), 12);
    }
}

