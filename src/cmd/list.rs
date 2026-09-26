use std::borrow::Cow;
use std::io::Write;

use anyhow::Result;
use unicode_width::UnicodeWidthChar;

use crate::repo::Repo;

/// Same shape as `git worktree list`: path, short HEAD, `[branch]` or `(detached HEAD)`, then
/// ` locked` and ` prunable` where they apply.
pub fn run() -> Result<()> {
    let repo = Repo::discover()?;
    let worktrees = repo.worktrees()?;
    let fully = repo.quotes_paths_fully();
    let paths: Vec<_> = worktrees
        .iter()
        .map(|w| quote_c_style(w.path.as_os_str().as_encoded_bytes(), fully))
        .collect();
    let widths: Vec<_> = paths.iter().map(|p| display_width(p)).collect();
    let heads: Vec<_> = worktrees
        .iter()
        .map(|w| w.head_short.as_deref().unwrap_or("0000000"))
        .collect();
    let path_width = widths.iter().copied().max().unwrap_or(0);
    let head_width = heads.iter().map(|h| h.len()).max().unwrap_or(0);

    let mut out = std::io::stdout().lock();
    for (((wt, path), width), head) in worktrees.iter().zip(&paths).zip(widths).zip(heads) {
        let branch = match &wt.branch {
            Some(b) => format!("[{b}]"),
            None => "(detached HEAD)".to_string(),
        };
        let locked = if wt.locked { " locked" } else { "" };
        let prunable = if wt.prunable { " prunable" } else { "" };
        let pad = path_width - width;
        out.write_all(path)?;
        writeln!(
            out,
            "{:pad$} {head:<head_width$} {branch}{locked}{prunable}",
            ""
        )?;
    }
    Ok(())
}

/// git's `utf8_strwidth`: the sum of each character's column width, or the byte count when
/// `text` isn't valid UTF-8.
fn display_width(text: &[u8]) -> usize {
    match std::str::from_utf8(text) {
        Ok(text) => text.chars().map(|c| c.width().unwrap_or(0)).sum(),
        Err(_) => text.len(),
    }
}

/// git's `quote_c_style`: wrap in double quotes and backslash-escape control characters, `"`
/// and `\`, plus every byte above 0x7f when `fully` (`core.quotePath`). Otherwise unchanged.
fn quote_c_style(name: &[u8], fully: bool) -> Cow<'_, [u8]> {
    let must_quote =
        |b: u8| b < 0x20 || b == b'"' || b == b'\\' || b == 0x7f || (fully && b >= 0x80);
    if !name.iter().copied().any(must_quote) {
        return Cow::Borrowed(name);
    }
    let mut quoted = vec![b'"'];
    for &b in name {
        if !must_quote(b) {
            quoted.push(b);
            continue;
        }
        quoted.push(b'\\');
        match b {
            0x07 => quoted.push(b'a'),
            0x08 => quoted.push(b'b'),
            b'\t' => quoted.push(b't'),
            b'\n' => quoted.push(b'n'),
            0x0b => quoted.push(b'v'),
            0x0c => quoted.push(b'f'),
            b'\r' => quoted.push(b'r'),
            b'"' | b'\\' => quoted.push(b),
            _ => quoted.extend(format!("{b:03o}").bytes()),
        }
    }
    quoted.push(b'"');
    Cow::Owned(quoted)
}
