use anyhow::Result;
use itertools::Itertools;

use ruff_macros::{ViolationMetadata, derive_message_formats};
use ruff_python_ast::parenthesize::parenthesized_range;
use ruff_python_ast::{self as ast, Arguments, Expr};
use ruff_python_semantic::SemanticModel;
use ruff_text_size::Ranged;

use crate::checkers::ast::Checker;
use crate::importer::ImportRequest;
use crate::{AlwaysFixableViolation, Edit, Fix};

/// ## What it does
/// Checks for the use of `sum()` to flatten lists of lists, which has
/// quadratic complexity.
///
/// ## Why is this bad?
/// The use of `sum()` to flatten lists of lists is quadratic in the number of
/// lists, as `sum()` creates a new list for each element in the summation.
///
/// Instead, consider using another method of flattening lists to avoid
/// quadratic complexity. The following methods are all linear in the number of
/// lists:
///
/// - `functools.reduce(operator.iadd, lists, [])`
/// - `list(itertools.chain.from_iterable(lists))`
/// - `[item for sublist in lists for item in sublist]`
///
/// When fixing relevant violations, Ruff defaults to the `functools.reduce`
/// form, which outperforms the other methods in [microbenchmarks].
///
/// ## Example
/// ```python
/// lists = [[1, 2, 3], [4, 5, 6], [7, 8, 9]]
/// joined = sum(lists, [])
/// ```
///
/// Use instead:
/// ```python
/// import functools
/// import operator
///
///
/// lists = [[1, 2, 3], [4, 5, 6], [7, 8, 9]]
/// functools.reduce(operator.iadd, lists, [])
/// ```
///
/// ## Fix safety
///
/// This fix is always marked as unsafe because `sum` uses the `__add__` magic method while
/// `operator.iadd` uses the `__iadd__` magic method, and these behave differently on lists.
/// The former requires the right summand to be a list, whereas the latter allows for any iterable.
/// Therefore, the fix could inadvertently cause code that previously raised an error to silently
/// succeed. Moreover, the fix could remove comments from the original code.
///
/// ## References
/// - [_How Not to Flatten a List of Lists in Python_](https://mathieularose.com/how-not-to-flatten-a-list-of-lists-in-python)
/// - [_How do I make a flat list out of a list of lists?_](https://stackoverflow.com/questions/952914/how-do-i-make-a-flat-list-out-of-a-list-of-lists/953097#953097)
///
/// [microbenchmarks]: https://github.com/astral-sh/ruff/issues/5073#issuecomment-1591836349
#[derive(ViolationMetadata)]
pub(crate) struct QuadraticListSummation;

impl AlwaysFixableViolation for QuadraticListSummation {
    #[derive_message_formats]
    fn message(&self) -> String {
        "Avoid quadratic list summation".to_string()
    }

    fn fix_title(&self) -> String {
        "Replace with `functools.reduce`".to_string()
    }
}

/// RUF017
pub(crate) fn quadratic_list_summation(checker: &Checker, call: &ast::ExprCall) {
    let ast::ExprCall {
        func,
        arguments,
        range,
        node_index: _,
    } = call;

    let Some(iterable) = arguments.args.first() else {
        return;
    };

    let semantic = checker.semantic();

    if !semantic.match_builtin_expr(func, "sum") {
        return;
    }

    if !start_is_empty_list(arguments, semantic) {
        return;
    }

    let mut diagnostic = checker.report_diagnostic(QuadraticListSummation, *range);
    diagnostic.try_set_fix(|| convert_to_reduce(iterable, call, checker));
}

/// Generate a [`Fix`] to convert a `sum()` call to a `functools.reduce()` call.
fn convert_to_reduce(iterable: &Expr, call: &ast::ExprCall, checker: &Checker) -> Result<Fix> {
    let (reduce_edit, reduce_binding) = checker.importer().get_or_import_symbol(
        &ImportRequest::import("functools", "reduce"),
        call.start(),
        checker.semantic(),
    )?;

    let (iadd_edit, iadd_binding) = checker.importer().get_or_import_symbol(
        &ImportRequest::import("operator", "iadd"),
        iterable.start(),
        checker.semantic(),
    )?;

    let iterable = checker.locator().slice(
        parenthesized_range(
            iterable.into(),
            (&call.arguments).into(),
            checker.comment_ranges(),
            checker.locator().contents(),
        )
        .unwrap_or(iterable.range()),
    );

    Ok(Fix::unsafe_edits(
        Edit::range_replacement(
            format!("{reduce_binding}({iadd_binding}, {iterable}, [])"),
            call.range(),
        ),
        [reduce_edit, iadd_edit].into_iter().dedup(),
    ))
}

/// Returns `true` if the `start` argument to a `sum()` call is an empty list.
fn start_is_empty_list(arguments: &Arguments, semantic: &SemanticModel) -> bool {
    let Some(start_arg) = arguments.find_argument_value("start", 1) else {
        return false;
    };

    match start_arg {
        Expr::Call(ast::ExprCall {
            func, arguments, ..
        }) => arguments.is_empty() && semantic.match_builtin_expr(func, "list"),
        Expr::List(list) => list.is_empty() && list.ctx.is_load(),
        _ => false,
    }
}
