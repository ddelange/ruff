use ruff_python_ast::StmtImportFrom;

use ruff_diagnostics::{Diagnostic, Fix, FixAvailability, Violation};
use ruff_macros::{ViolationMetadata, derive_message_formats};

use crate::{checkers::ast::Checker, fix, preview::is_fix_future_annotations_in_stub_enabled};

/// ## What it does
/// Checks for the presence of the `from __future__ import annotations` import
/// statement in stub files.
///
/// ## Why is this bad?
/// Stub files natively support forward references in all contexts, as stubs are
/// never executed at runtime. (They should be thought of as "data files" for
/// type checkers.) As such, the `from __future__ import annotations` import
/// statement has no effect and should be omitted.
///
/// ## References
/// - [Static Typing with Python: Type Stubs](https://typing.python.org/en/latest/source/stubs.html)
#[derive(ViolationMetadata)]
pub(crate) struct FutureAnnotationsInStub;

impl Violation for FutureAnnotationsInStub {
    const FIX_AVAILABILITY: FixAvailability = FixAvailability::Sometimes;

    #[derive_message_formats]
    fn message(&self) -> String {
        "`from __future__ import annotations` has no effect in stub files, since type checkers automatically treat stubs as having those semantics".to_string()
    }

    fn fix_title(&self) -> Option<String> {
        Some("Remove `from __future__ import annotations`".to_string())
    }
}

/// PYI044
pub(crate) fn from_future_import(checker: &Checker, target: &StmtImportFrom) {
    let StmtImportFrom {
        range,
        module: Some(module_name),
        names,
        ..
    } = target
    else {
        return;
    };

    if module_name != "__future__" {
        return;
    }

    if names.iter().all(|alias| &*alias.name != "annotations") {
        return;
    }

    let mut diagnostic = Diagnostic::new(FutureAnnotationsInStub, *range);

    if is_fix_future_annotations_in_stub_enabled(checker.settings) {
        let stmt = checker.semantic().current_statement();

        diagnostic.try_set_fix(|| {
            let edit = fix::edits::remove_unused_imports(
                std::iter::once("annotations"),
                stmt,
                None,
                checker.locator(),
                checker.stylist(),
                checker.indexer(),
            )?;

            Ok(Fix::safe_edit(edit))
        });
    }

    checker.report_diagnostic(diagnostic);
}
