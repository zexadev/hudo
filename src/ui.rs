use std::io::Write;
use console::{measure_text_width, pad_str, style, Alignment, Style};
use figlet_rs::FIGfont;

/// 打印 hudo 品牌 Banner
pub fn print_banner() {
    let stdout = std::io::stdout();
    let mut w = std::io::BufWriter::new(stdout.lock());
    let s = Style::new().cyan().bold();
    if let Ok(font) = FIGfont::standard() {
        if let Some(figure) = font.convert("hudo") {
            for line in figure.to_string().lines() {
                let _ = writeln!(w, "{}", s.apply_to(line));
            }
        }
    }
    let _ = writeln!(w, "  {}", style("混沌 — 开发环境一键引导工具").dim());
    let _ = writeln!(w);
}

/// 清屏
pub fn clear_screen() {
    let mut stdout = std::io::stdout().lock();
    let _ = write!(stdout, "\x1B[2J\x1B[3J\x1B[H");
    let _ = stdout.flush();
}

/// 打印标题行 + 下划线
pub fn print_title(text: &str) {
    let width = measure_text_width(text).max(40);
    let s = Style::new().bold().cyan();
    println!();
    println!("{}", s.apply_to(text));
    println!("{}", s.apply_to("─".repeat(width)));
}

/// 打印分类标题（用于 list / setup 中的分组）
pub fn print_section(text: &str) {
    println!();
    println!("  {} {}", style("■").cyan(), style(text).bold());
}

/// 打印进度步骤
pub fn print_step(step: u32, total: u32, text: &str) {
    println!(
        "  {} {}",
        style(format!("[{}/{}]", step, total)).cyan().bold(),
        style(text).bold()
    );
}

pub fn print_success(text: &str) {
    println!("  {} {}", style("✓").green().bold(), text);
}

pub fn print_warning(text: &str) {
    println!("  {} {}", style("⚠").yellow().bold(), text);
}

#[allow(dead_code)]
pub fn print_error(text: &str) {
    println!("  {} {}", style("✗").red().bold(), text);
}

pub fn print_info(text: &str) {
    println!("  {}", style(text).dim());
}

/// 打印正在进行的操作
pub fn print_action(text: &str) {
    println!("  {} {}", style("→").cyan(), text);
}

/// 将文本填充到指定显示宽度（处理中文双宽字符）
pub fn pad(text: &str, width: usize) -> String {
    pad_str(text, width, Alignment::Left, None).to_string()
}

/// 工具分类
pub enum ToolCategory {
    Tool,
    Language,
    Database,
    Ide,
}

impl ToolCategory {
    pub fn label(&self) -> &'static str {
        match self {
            ToolCategory::Tool => "工具",
            ToolCategory::Language => "语言环境",
            ToolCategory::Database => "数据库",
            ToolCategory::Ide => "编辑器 / IDE",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            ToolCategory::Tool => "🔧",
            ToolCategory::Language => "📝",
            ToolCategory::Database => "🗄️",
            ToolCategory::Ide => "💻",
        }
    }

    pub fn from_id(id: &str) -> Self {
        match id {
            "git" => ToolCategory::Tool,
            "uv" | "nodejs" | "bun" | "miniconda" | "rust" | "go" | "jdk" | "c" | "maven" | "gradle" => ToolCategory::Language,
            "mysql" | "pgsql" => ToolCategory::Database,
            "vscode" | "pycharm" => ToolCategory::Ide,
            _ => ToolCategory::Tool,
        }
    }
}

/// 页面头部：清屏 + Banner + 标题
pub fn page_header(title: &str) {
    clear_screen();
    print_banner();
    print_title(title);
}

/// 暂停等待用户按键
pub fn wait_for_key() {
    println!();
    println!("  {}", style("按任意键返回...").dim());
    let _ = console::Term::stderr().read_key();
}
