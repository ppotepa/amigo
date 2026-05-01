use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};

use super::{DiagnosticSeverity, LauncherProfile, LauncherTuiState, ProfileDiagnostics};

pub(super) fn active_profile_health_line(state: &LauncherTuiState) -> Line<'static> {
    profile_health_line(state.active_profile_diagnostics())
}

pub(super) fn profile_tab_title(profile: &LauncherProfile, state: &LauncherTuiState) -> Line<'static> {
    let diagnostics = state.profile_diagnostics.get(&profile.id);
    let mut spans = vec![
        Span::styled(
            if profile.id == state.config.active_profile {
                "* "
            } else {
                "  "
            },
            if profile.id == state.config.active_profile {
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::DarkGray)
            },
        ),
        Span::styled(
            profile.display_label().to_owned(),
            Style::default().add_modifier(Modifier::BOLD),
        ),
    ];

    if let Some(report) = diagnostics {
        if report.error_count() > 0 {
            spans.push(Span::raw(" "));
            spans.push(Span::styled(
                format!("E{}", report.error_count()),
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            ));
        }

        if report.warning_count() > 0 {
            spans.push(Span::raw(" "));
            spans.push(Span::styled(
                format!("W{}", report.warning_count()),
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ));
        }
    }

    Line::from(spans)
}

pub(super) fn profile_health_line(diagnostics: Option<&ProfileDiagnostics>) -> Line<'static> {
    let (label, style) = match diagnostics {
        Some(report) if report.has_errors() => (
            format!("health: blocked ({} error(s))", report.error_count()),
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        ),
        Some(report) if report.warning_count() > 0 => (
            format!("health: warnings ({} warning(s))", report.warning_count()),
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Some(_) => (
            "health: ready".to_owned(),
            Style::default().fg(Color::Green),
        ),
        None => (
            "health: diagnostics unavailable".to_owned(),
            Style::default().fg(Color::Yellow),
        ),
    };

    Line::from(Span::styled(label, style))
}

pub(super) fn primary_diagnostic_line(diagnostics: Option<&ProfileDiagnostics>) -> Line<'static> {
    let Some(report) = diagnostics else {
        return Line::from(Span::styled(
            "diagnostics: unavailable",
            Style::default().fg(Color::Yellow),
        ));
    };

    let Some(diagnostic) = report.diagnostics.first() else {
        return Line::from(Span::styled(
            "diagnostics: none",
            Style::default().fg(Color::Green),
        ));
    };

    let label = match diagnostic.severity {
        DiagnosticSeverity::Warning => "diagnostics: warning",
        DiagnosticSeverity::Error => "diagnostics: error",
    };
    let style = match diagnostic.severity {
        DiagnosticSeverity::Warning => Style::default().fg(Color::Yellow),
        DiagnosticSeverity::Error => Style::default().fg(Color::Red),
    };

    Line::from(vec![
        Span::styled(format!("{label} "), style.add_modifier(Modifier::BOLD)),
        Span::raw(diagnostic.message.clone()),
    ])
}

