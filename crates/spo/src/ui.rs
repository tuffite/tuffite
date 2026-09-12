//! Shared English terminal UI; explicit arguments never require terminal input.
use anyhow::{Result, bail, ensure};
use console::{Key, Style, Term, measure_text_width, truncate_str};
use std::io::{self, IsTerminal};

#[must_use]
pub fn interactive(disabled: bool) -> bool {
    !disabled && io::stdin().is_terminal() && io::stderr().is_terminal()
}

pub fn input(prompt: &str, disabled: bool) -> Result<String> {
    ensure!(
        interactive(disabled),
        "{prompt}: provide an explicit argument in non-interactive mode"
    );
    let term = Term::stderr();
    term.write_str(&format!("{prompt}: "))?;
    let mut line = String::new();
    loop {
        match term.read_key_raw()? {
            Key::Enter => {
                term.write_line("")?;
                return Ok(line.trim().to_owned());
            }
            Key::Escape | Key::CtrlC => {
                term.write_line("")?;
                bail!("operation cancelled");
            }
            Key::Backspace => {
                if let Some(letter) = line.pop() {
                    term.clear_chars(measure_text_width(&letter.to_string()))?;
                }
            }
            Key::Char(letter) if !letter.is_control() => {
                line.push(letter);
                term.write_str(&letter.to_string())?;
            }
            _ => {}
        }
    }
}

struct Picker<'a> {
    choices: &'a [String],
    query: String,
    filtering: bool,
    cursor: usize,
    selected: Vec<bool>,
    multiple: bool,
}
impl<'a> Picker<'a> {
    fn new(choices: &'a [String], multiple: bool) -> Self {
        Self {
            choices,
            query: String::new(),
            filtering: false,
            cursor: 0,
            selected: vec![false; choices.len()],
            multiple,
        }
    }

    fn visible(&self) -> Vec<usize> {
        let query = self.query.to_lowercase();
        self.choices
            .iter()
            .enumerate()
            .filter_map(|(index, choice)| {
                let mut candidate = choice
                    .to_lowercase()
                    .chars()
                    .collect::<Vec<_>>()
                    .into_iter();
                query
                    .chars()
                    .all(|needle| candidate.any(|letter| letter == needle))
                    .then_some(index)
            })
            .collect()
    }

    fn key(&mut self, key: &Key) -> Result<Option<Vec<usize>>> {
        let visible = self.visible();
        match key {
            Key::Escape if self.filtering => {
                self.query.clear();
                self.filtering = false;
                self.cursor = 0;
            }
            Key::Escape | Key::CtrlC => bail!("operation cancelled"),
            Key::ArrowUp if !visible.is_empty() => {
                self.cursor = (self.cursor + visible.len() - 1) % visible.len();
            }
            Key::ArrowDown if !visible.is_empty() => {
                self.cursor = (self.cursor + 1) % visible.len();
            }
            Key::Home => self.cursor = 0,
            Key::End if !visible.is_empty() => self.cursor = visible.len() - 1,
            Key::Char(' ') if self.multiple => {
                if let Some(index) = visible.get(self.cursor) {
                    self.selected[*index] = !self.selected[*index];
                }
            }
            Key::Enter if self.multiple => {
                let selected: Vec<_> = self
                    .selected
                    .iter()
                    .enumerate()
                    .filter_map(|(index, selected)| selected.then_some(index))
                    .collect();
                if !selected.is_empty() {
                    return Ok(Some(selected));
                }
            }
            Key::Enter if !visible.is_empty() => return Ok(Some(vec![visible[self.cursor]])),
            Key::Backspace if self.filtering => {
                self.query.pop();
                self.cursor = 0;
            }
            Key::Char('/') if !self.filtering => self.filtering = true,
            Key::Char(letter) if self.filtering && !letter.is_control() => {
                self.query.push(*letter);
                self.cursor = 0;
            }
            _ => {}
        }
        Ok(None)
    }
}

struct Cursor<'a>(&'a Term);
impl Drop for Cursor<'_> {
    fn drop(&mut self) {
        let _ = self.0.show_cursor();
    }
}

fn plain(text: &str) -> String {
    text.chars().filter(|letter| !letter.is_control()).collect()
}

fn choice_label(
    choice: &str,
    focused: bool,
    selected: Option<bool>,
    command_width: usize,
) -> String {
    let accent = Style::new().for_stderr().magenta();
    let command = Style::new().for_stderr().cyan();
    let muted = Style::new().for_stderr().dim();
    let focus = accent.apply_to(if focused { ">" } else { " " });
    let mark = match selected {
        Some(true) => "[x] ",
        Some(false) => "[ ] ",
        None => "",
    };
    let (name, description) = choice.split_once('\t').unwrap_or((choice, ""));
    let name = plain(name);
    let padding = " ".repeat(command_width.saturating_sub(measure_text_width(&name)) + 2);
    let name = if focused {
        command.clone().bold().apply_to(name)
    } else {
        command.apply_to(name)
    };
    if description.is_empty() {
        format!("{focus} {mark}{name}")
    } else {
        format!(
            "{focus} {mark}{name}{padding}{}",
            muted.apply_to(plain(description))
        )
    }
}

fn render(term: &Term, prompt: &str, picker: &Picker<'_>) -> Result<usize> {
    let choices = picker.choices;
    let multiple = picker.multiple;
    let (height, width) = term.size();
    let width = usize::from(width).saturating_sub(1).max(1);
    let capacity = usize::from(height).saturating_sub(5).clamp(1, 12);
    let visible = picker.visible();
    let start = picker
        .cursor
        .saturating_sub(capacity / 2)
        .min(visible.len().saturating_sub(capacity));
    let muted = Style::new().for_stderr().dim();
    let accent = Style::new().for_stderr().magenta();
    let command_width = choices
        .iter()
        .map(|choice| {
            measure_text_width(&plain(
                choice
                    .split_once('\t')
                    .map_or(choice.as_str(), |(name, _)| name),
            ))
        })
        .max()
        .unwrap_or(0)
        .min(width / 2);
    term.write_line(&truncate_str(
        &accent.clone().bold().apply_to(prompt).to_string(),
        width,
        "…",
    ))?;
    let mut lines = 1;
    for (position, index) in visible.iter().enumerate().skip(start).take(capacity) {
        let label = choice_label(
            &choices[*index],
            position == picker.cursor,
            multiple.then_some(picker.selected[*index]),
            command_width,
        );
        term.write_line(&truncate_str(&label, width, "…"))?;
        lines += 1;
    }
    if visible.is_empty() {
        term.write_line(&muted.apply_to("No matches").to_string())?;
        lines += 1;
    }
    if picker.filtering {
        term.write_line(&truncate_str(
            &format!(
                "{}{}{}",
                Style::new().for_stderr().green().apply_to("/"),
                picker.query,
                Style::new().for_stderr().reverse().apply_to(" ")
            ),
            width,
            "…",
        ))?;
        lines += 1;
    }
    let hint = if picker.filtering {
        "esc clear filter · enter confirm"
    } else {
        "/ filter · ↑↓ move · enter confirm · esc cancel"
    };
    let hint = if multiple {
        format!(
            "{hint} · space toggle ({} selected)",
            picker.selected.iter().filter(|selected| **selected).count()
        )
    } else {
        hint.into()
    };
    term.write_line(&truncate_str(&muted.apply_to(hint).to_string(), width, "…"))?;
    lines += 1;
    Ok(lines)
}

fn picker(prompt: &str, choices: &[String], disabled: bool, multiple: bool) -> Result<Vec<usize>> {
    ensure!(!choices.is_empty(), "no available choices");
    ensure!(
        interactive(disabled),
        "{prompt}: select an explicit subcommand or target in non-interactive mode"
    );
    let term = Term::stderr();
    term.hide_cursor()?;
    let _cursor = Cursor(&term);
    let mut picker = Picker::new(choices, multiple);
    let mut lines = 0;
    loop {
        term.clear_last_lines(lines)?;
        lines = render(&term, prompt, &picker)?;
        match picker.key(&term.read_key_raw()?) {
            Ok(Some(selected)) => {
                term.clear_last_lines(lines)?;
                term.write_line(&format!(
                    "{prompt}: {}",
                    selected
                        .iter()
                        .map(|index| choices[*index]
                            .split_once('\t')
                            .map_or(choices[*index].as_str(), |(name, _)| name))
                        .collect::<Vec<_>>()
                        .join(", ")
                ))?;
                return Ok(selected);
            }
            Err(error) => {
                term.clear_last_lines(lines)?;
                return Err(error);
            }
            Ok(None) => {}
        }
    }
}

/// A tab separates a command/name from its optional muted description.
pub fn select(prompt: &str, choices: &[String], disabled: bool) -> Result<usize> {
    Ok(picker(prompt, choices, disabled, false)?[0])
}

pub fn multi_select(prompt: &str, choices: &[String], disabled: bool) -> Result<Vec<usize>> {
    picker(prompt, choices, disabled, true)
}

pub fn confirm(prompt: &str, yes: bool, disabled: bool, default: bool) -> Result<()> {
    if yes {
        return Ok(());
    }
    let answer = input(
        &format!("{prompt} {}", if default { "[Y/n]" } else { "[y/N]" }),
        disabled,
    )?;
    if (answer.is_empty() && default)
        || answer.eq_ignore_ascii_case("y")
        || answer.eq_ignore_ascii_case("yes")
    {
        Ok(())
    } else {
        bail!("operation cancelled")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn filtering_navigation_and_cancellation() {
        let choices = vec![
            "Build Playground".into(),
            "Roll dependencies".into(),
            "Package Framework distribution".into(),
        ];
        let mut picker = Picker::new(&choices, false);
        picker.key(&Key::Char('z')).unwrap();
        assert!(picker.query.is_empty());
        picker.key(&Key::Char('/')).unwrap();
        for letter in "d".chars() {
            picker.key(&Key::Char(letter)).unwrap();
        }
        assert_eq!(picker.visible(), vec![0, 1, 2]);
        picker.key(&Key::ArrowUp).unwrap();
        assert_eq!(picker.key(&Key::Enter).unwrap(), Some(vec![2]));
        picker.key(&Key::Char('z')).unwrap();
        assert!(picker.visible().is_empty());
        assert!(picker.key(&Key::Enter).unwrap().is_none());
        picker.key(&Key::Backspace).unwrap();
        assert_eq!(picker.cursor, 0);
        picker.key(&Key::Escape).unwrap();
        assert!(picker.query.is_empty());
        assert!(!picker.filtering);
        assert!(picker.key(&Key::Escape).is_err());
    }
    #[test]
    fn multi_selection_survives_filter_changes() {
        let choices = vec!["alpha.txt".into(), "beta.txt".into(), "gamma.txt".into()];
        let mut picker = Picker::new(&choices, true);
        assert!(picker.key(&Key::Enter).unwrap().is_none());
        picker.key(&Key::Char(' ')).unwrap();
        picker.key(&Key::Char('/')).unwrap();
        picker.key(&Key::Char('b')).unwrap();
        picker.key(&Key::Char(' ')).unwrap();
        assert_eq!(picker.key(&Key::Enter).unwrap(), Some(vec![0, 1]));
        picker.key(&Key::Escape).unwrap();
        picker.key(&Key::Char(' ')).unwrap();
        assert_eq!(picker.key(&Key::Enter).unwrap(), Some(vec![1]));
    }
}
