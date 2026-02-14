use console::Style;

pub fn print_title(text: &str) {
    let style = Style::new().bold().cyan();
    println!("\n{}", style.apply_to(text));
    println!("{}", style.apply_to("─".repeat(text.len().max(40))));
}

pub fn print_step(step: u32, total: u32, text: &str) {
    let style = Style::new().bold();
    println!("  [{}/{}] {}", step, total, style.apply_to(text));
}

pub fn print_success(text: &str) {
    let style = Style::new().green();
    println!("  {} {}", style.apply_to("✓"), text);
}

pub fn print_warning(text: &str) {
    let style = Style::new().yellow();
    println!("  {} {}", style.apply_to("⚠"), text);
}

pub fn print_error(text: &str) {
    let style = Style::new().red();
    println!("  {} {}", style.apply_to("✗"), text);
}

pub fn print_info(text: &str) {
    let style = Style::new().dim();
    println!("  {}", style.apply_to(text));
}
