use std::io;
use std::path::{Path, PathBuf};
use std::process::{self, Command};
use std::sync::OnceLock;

use chrono::{DateTime, NaiveDate, Utc};
use mdbook_preprocessor::book::{Book, BookItem};
use mdbook_preprocessor::errors::{Error, Result};
use mdbook_preprocessor::{parse_input, Preprocessor, PreprocessorContext};
use regex::{Captures, Regex};
use semver::{Version, VersionReq};

const TOKEN_RE: &str = r"\{\{\s*relative_to\s*:\s*([^}]+?)\s*\}\}";

fn main() {
    let preprocessor = RelativeDatePreprocessor;

    if let Err(err) = run(&preprocessor) {
        eprintln!("relative-date preprocessor failed: {err}");
        process::exit(1);
    }
}

fn run(preprocessor: &dyn Preprocessor) -> Result<()> {
    let mut args = std::env::args();
    let _program = args.next();

    if let Some(command) = args.next() {
        match command.as_str() {
            "supports" => {
                let renderer = args.next().unwrap_or_default();
                process::exit(if preprocessor.supports_renderer(&renderer)? {
                    0
                } else {
                    1
                });
            }
            unknown => {
                return Err(Error::msg(format!("unknown command: {unknown}")));
            }
        }
    }

    let (ctx, book) = parse_input(io::stdin())?;
    check_mdbook_version(&ctx)?;
    let processed_book = preprocessor.run(&ctx, book)?;
    serde_json::to_writer(io::stdout(), &processed_book)?;

    Ok(())
}

#[derive(Default)]
struct RelativeDatePreprocessor;

impl Preprocessor for RelativeDatePreprocessor {
    fn name(&self) -> &str {
        "relative-date"
    }

    fn run(&self, ctx: &PreprocessorContext, mut book: Book) -> Result<Book> {
        let today = Utc::now().date_naive();
        let book_src = ctx.config.book.src.clone();

        book.for_each_mut(|item| {
            if let BookItem::Chapter(chapter) = item {
                let chapter_path = chapter
                    .source_path
                    .clone()
                    .or_else(|| chapter.path.clone());
                chapter.content = render_content(
                    &chapter.content,
                    chapter.name.as_str(),
                    chapter_path.as_deref(),
                    &ctx.root,
                    &book_src,
                    today,
                );
            }
        });

        Ok(book)
    }

    fn supports_renderer(&self, _renderer: &str) -> Result<bool> {
        Ok(true)
    }
}

fn check_mdbook_version(ctx: &PreprocessorContext) -> Result<()> {
    let Some(mdbook_version) = Version::parse(&ctx.mdbook_version).ok() else {
        return Ok(());
    };

    let version_req = VersionReq::parse("^0.5")?;
    if !version_req.matches(&mdbook_version) {
        eprintln!(
            "warning: mdbook-relative-date was built for mdBook 0.5.x, but mdBook {} is running",
            ctx.mdbook_version
        );
    }

    Ok(())
}

fn render_content(
    content: &str,
    chapter_name: &str,
    chapter_path: Option<&Path>,
    book_root: &Path,
    book_src: &Path,
    today: NaiveDate,
) -> String {
    token_regex()
        .replace_all(content, |captures: &Captures<'_>| {
            let raw_value = captures
                .get(1)
                .map(|m| m.as_str().trim())
                .unwrap_or_default();

            let target_date = match raw_value {
                "current" => page_git_date(book_root, book_src, chapter_path).unwrap_or_else(|| {
                    warn(chapter_name, raw_value, "could not read git commit date; using system date");
                    today
                }),
                value => match NaiveDate::parse_from_str(value, "%Y_%m_%d") {
                    Ok(date) => date,
                    Err(_) => {
                        warn(
                            chapter_name,
                            raw_value,
                            "invalid date token; expected YYYY_MM_DD or current; removing token",
                        );
                        return String::new();
                    }
                },
            };

            format!("({})", relative_date_text(today, target_date))
        })
        .into_owned()
}

fn token_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(TOKEN_RE).expect("token regex should compile"))
}

fn page_git_date(book_root: &Path, book_src: &Path, chapter_path: Option<&Path>) -> Option<NaiveDate> {
    let chapter_path = chapter_path?;
    let git_path = normalize_git_path(book_src.join(chapter_path));

    let output = Command::new("git")
        .arg("-C")
        .arg(book_root)
        .arg("log")
        .arg("-1")
        .arg("--format=%cI")
        .arg("--")
        .arg(git_path)
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8(output.stdout).ok()?;
    let commit_date = stdout.trim();
    if commit_date.is_empty() {
        return None;
    }

    DateTime::parse_from_rfc3339(commit_date)
        .ok()
        .map(|date_time| date_time.with_timezone(&Utc).date_naive())
}

fn normalize_git_path(path: PathBuf) -> String {
    path.components()
        .map(|component| component.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/")
}

fn relative_date_text(today: NaiveDate, target: NaiveDate) -> String {
    let days = (today - target).num_days();
    let abs_days = days.abs();

    if abs_days == 0 {
        return "today".to_string();
    }

    let unit_text = coarse_unit(abs_days);
    if days > 0 {
        format!("{unit_text} ago")
    } else {
        format!("in {unit_text}")
    }
}

fn coarse_unit(abs_days: i64) -> String {
    if abs_days < 14 {
        plural(abs_days, "day")
    } else if abs_days < 60 {
        plural(abs_days / 7, "week")
    } else if abs_days < 365 {
        plural(abs_days / 30, "month")
    } else {
        plural(abs_days / 365, "year")
    }
}

fn plural(value: i64, unit: &str) -> String {
    if value == 1 {
        format!("1 {unit}")
    } else {
        format!("{value} {unit}s")
    }
}

fn warn(chapter_name: &str, token: &str, message: &str) {
    eprintln!("warning: chapter '{chapter_name}', relative_to:{token}: {message}");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_explicit_date_tokens() {
        let today = NaiveDate::from_ymd_opt(2026, 5, 13).unwrap();
        let rendered = render_content(
            "Published {{relative_to:2026_05_01}} and {{ relative_to: 2026_05_16 }}.",
            "Test",
            None,
            Path::new("."),
            Path::new("src"),
            today,
        );

        assert_eq!(rendered, "Published (12 days ago) and (in 3 days).");
    }

    #[test]
    fn removes_invalid_date_tokens() {
        let today = NaiveDate::from_ymd_opt(2026, 5, 13).unwrap();
        let rendered = render_content(
            "Bad {{ relative_to: 2026-05-01 }} token.",
            "Test",
            None,
            Path::new("."),
            Path::new("src"),
            today,
        );

        assert_eq!(rendered, "Bad  token.");
    }

    #[test]
    fn renders_coarse_units() {
        let today = NaiveDate::from_ymd_opt(2026, 5, 13).unwrap();

        assert_eq!(
            relative_date_text(today, NaiveDate::from_ymd_opt(2026, 5, 12).unwrap()),
            "1 day ago"
        );
        assert_eq!(
            relative_date_text(today, NaiveDate::from_ymd_opt(2026, 4, 15).unwrap()),
            "4 weeks ago"
        );
        assert_eq!(
            relative_date_text(today, NaiveDate::from_ymd_opt(2025, 5, 13).unwrap()),
            "1 year ago"
        );
    }
}
