use std::{
    env,
    fmt::Write as _,
    panic::{self, PanicHookInfo},
    path::Path,
    sync::OnceLock,
};

use m32_domain::BuildInfo;

pub const CRASH_TEST_ENV: &str = "M32_CRASH_TEST";
pub const CRASH_REPORT_SCHEMA_VERSION: u32 = 1;
const MAX_PANIC_MESSAGE_CHARS: usize = 2_048;

static HOOK_INSTALLED: OnceLock<()> = OnceLock::new();

pub fn install() {
    if HOOK_INSTALLED.set(()).is_err() {
        return;
    }

    panic::set_hook(Box::new(|panic_info| {
        handle_panic(panic_info);
    }));
}

pub fn trigger_smoke_test_if_requested() {
    if cfg!(debug_assertions) && should_trigger_smoke_test(env::var(CRASH_TEST_ENV).ok().as_deref()) {
        panic!("M32 intentional crash smoke test");
    }
}

fn handle_panic(panic_info: &PanicHookInfo<'_>) {
    let build_info = BuildInfo::current();
    let current_thread = std::thread::current();
    let thread_name = current_thread.name().unwrap_or("<unnamed>");
    let message = panic_message(panic_info);
    let (source_file, source_line, source_column) = panic_location(panic_info);

    tracing::error!(
        target: "m32::crash",
        event = "panic",
        schema_version = CRASH_REPORT_SCHEMA_VERSION,
        app_version = build_info.app_version,
        git_commit = build_info.git_commit,
        thread = thread_name,
        "M32 panic captured"
    );

    let report = render_report(
        build_info,
        thread_name,
        &message,
        source_file.as_deref(),
        source_line,
        source_column,
    );

    eprintln!("{report}");
}

fn panic_message(panic_info: &PanicHookInfo<'_>) -> String {
    let raw = if let Some(message) = panic_info.payload().downcast_ref::<&str>() {
        *message
    } else if let Some(message) = panic_info.payload().downcast_ref::<String>() {
        message.as_str()
    } else {
        "<non-string panic payload>"
    };

    sanitize_message(raw)
}

fn panic_location(panic_info: &PanicHookInfo<'_>) -> (Option<String>, Option<u32>, Option<u32>) {
    let Some(location) = panic_info.location() else {
        return (None, None, None);
    };

    (
        Some(sanitize_source_file(location.file())),
        Some(location.line()),
        Some(location.column()),
    )
}

fn sanitize_source_file(raw: &str) -> String {
    let path = Path::new(raw);

    if path.is_absolute() {
        path.file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("<unknown>")
            .to_owned()
    } else {
        raw.replace('\\', "/")
    }
}

fn sanitize_message(raw: &str) -> String {
    let mut output = String::new();

    for (index, character) in raw.chars().enumerate() {
        if index >= MAX_PANIC_MESSAGE_CHARS {
            output.push('…');
            break;
        }

        match character {
            '\r' | '\n' | '\t' => output.push(' '),
            character if character.is_control() => output.push('�'),
            character => output.push(character),
        }
    }

    output
}

fn render_report(
    build_info: BuildInfo,
    thread_name: &str,
    panic_message: &str,
    source_file: Option<&str>,
    source_line: Option<u32>,
    source_column: Option<u32>,
) -> String {
    let mut report = String::new();

    writeln!(report, "M32 Crash Report").expect("writing to String cannot fail");
    writeln!(report, "schema_version={CRASH_REPORT_SCHEMA_VERSION}").expect("writing to String cannot fail");
    writeln!(report, "app_version={}", build_info.app_version).expect("writing to String cannot fail");
    writeln!(report, "product_spec_version={}", build_info.product_spec_version)
        .expect("writing to String cannot fail");
    writeln!(report, "spec_bundle_version={}", build_info.spec_bundle_version).expect("writing to String cannot fail");
    writeln!(report, "git_commit={}", build_info.git_commit).expect("writing to String cannot fail");
    writeln!(report, "wie_commit={}", build_info.wie_commit).expect("writing to String cannot fail");
    writeln!(report, "rust_version={}", build_info.rust_version).expect("writing to String cannot fail");
    writeln!(report, "target={}", build_info.target).expect("writing to String cannot fail");
    writeln!(report, "build_profile={}", build_info.build_profile).expect("writing to String cannot fail");
    writeln!(report, "thread={thread_name}").expect("writing to String cannot fail");
    writeln!(report, "panic_message={panic_message}").expect("writing to String cannot fail");
    writeln!(report, "source_file={}", source_file.unwrap_or("<unknown>")).expect("writing to String cannot fail");
    writeln!(
        report,
        "source_line={}",
        source_line
            .map(|value| value.to_string())
            .as_deref()
            .unwrap_or("<unknown>")
    )
    .expect("writing to String cannot fail");
    writeln!(
        report,
        "source_column={}",
        source_column
            .map(|value| value.to_string())
            .as_deref()
            .unwrap_or("<unknown>")
    )
    .expect("writing to String cannot fail");

    report
}

fn should_trigger_smoke_test(raw: Option<&str>) -> bool {
    raw.is_some_and(|value| value.trim().eq_ignore_ascii_case("panic"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn panic_message_is_forced_to_one_line() {
        assert_eq!(sanitize_message("first\r\nsecond\tthird"), "first  second third");
    }

    #[test]
    fn long_panic_message_is_bounded() {
        let input = "x".repeat(MAX_PANIC_MESSAGE_CHARS + 10);
        let sanitized = sanitize_message(&input);

        assert_eq!(sanitized.chars().count(), MAX_PANIC_MESSAGE_CHARS + 1);
        assert!(sanitized.ends_with('…'));
    }

    #[test]
    fn crash_smoke_test_requires_exact_panic_value() {
        assert!(should_trigger_smoke_test(Some("panic")));
        assert!(should_trigger_smoke_test(Some(" PANIC ")));
        assert!(!should_trigger_smoke_test(None));
        assert!(!should_trigger_smoke_test(Some("1")));
        assert!(!should_trigger_smoke_test(Some("true")));
    }

    #[test]
    fn rendered_report_contains_locked_build_identity() {
        let report = render_report(
            BuildInfo::current(),
            "test-thread",
            "synthetic panic",
            Some("src/test.rs"),
            Some(10),
            Some(20),
        );

        assert!(report.contains("M32 Crash Report"));
        assert!(report.contains("schema_version=1"));
        assert!(report.contains("product_spec_version=1.0.0"));
        assert!(report.contains("spec_bundle_version=1.0.1"));
        assert!(report.contains("wie_commit=f0513eb758c02736981f545ad030eed937d55f3e"));
        assert!(report.contains("thread=test-thread"));
        assert!(report.contains("panic_message=synthetic panic"));
        assert!(report.contains("source_file=src/test.rs"));
        assert!(report.contains("source_line=10"));
        assert!(report.contains("source_column=20"));
    }
}
