use ruff_python_ast::{self as ast, Arguments, Expr, ExprContext, Operator};
use ruff_python_literal::cformat::{CFormatError, CFormatErrorType};

use ruff_python_ast::types::Node;
use ruff_python_semantic::ScopeKind;
use ruff_python_semantic::analyze::typing;
use ruff_text_size::Ranged;

use crate::checkers::ast::Checker;
use crate::preview::{
    is_assert_raises_exception_call_enabled, is_optional_as_none_in_union_enabled,
};
use crate::registry::Rule;
use crate::rules::{
    airflow, flake8_2020, flake8_async, flake8_bandit, flake8_boolean_trap, flake8_bugbear,
    flake8_builtins, flake8_comprehensions, flake8_datetimez, flake8_debugger, flake8_django,
    flake8_future_annotations, flake8_gettext, flake8_implicit_str_concat, flake8_logging,
    flake8_logging_format, flake8_pie, flake8_print, flake8_pyi, flake8_pytest_style, flake8_self,
    flake8_simplify, flake8_tidy_imports, flake8_type_checking, flake8_use_pathlib, flynt, numpy,
    pandas_vet, pep8_naming, pycodestyle, pyflakes, pylint, pyupgrade, refurb, ruff,
};
use ruff_python_ast::PythonVersion;

/// Run lint rules over an [`Expr`] syntax node.
pub(crate) fn expression(expr: &Expr, checker: &Checker) {
    match expr {
        Expr::Subscript(subscript @ ast::ExprSubscript { value, slice, .. }) => {
            // Ex) Optional[...], Union[...]
            if checker.any_rule_enabled(&[
                Rule::FutureRewritableTypeAnnotation,
                Rule::NonPEP604AnnotationUnion,
                Rule::NonPEP604AnnotationOptional,
            ]) {
                if let Some(operator) = typing::to_pep604_operator(value, slice, &checker.semantic)
                {
                    if checker.is_rule_enabled(Rule::FutureRewritableTypeAnnotation) {
                        if !checker.semantic.future_annotations_or_stub()
                            && checker.target_version() < PythonVersion::PY310
                            && checker.target_version() >= PythonVersion::PY37
                            && checker.semantic.in_annotation()
                            && !checker.settings().pyupgrade.keep_runtime_typing
                        {
                            flake8_future_annotations::rules::future_rewritable_type_annotation(
                                checker, value,
                            );
                        }
                    }
                    if checker.any_rule_enabled(&[
                        Rule::NonPEP604AnnotationUnion,
                        Rule::NonPEP604AnnotationOptional,
                    ]) {
                        if checker.source_type.is_stub()
                            || checker.target_version() >= PythonVersion::PY310
                            || (checker.target_version() >= PythonVersion::PY37
                                && checker.semantic.future_annotations_or_stub()
                                && checker.semantic.in_annotation()
                                && !checker.settings().pyupgrade.keep_runtime_typing)
                        {
                            pyupgrade::rules::non_pep604_annotation(checker, expr, slice, operator);
                        }
                    }
                }
            }

            // Ex) list[...]
            if checker.is_rule_enabled(Rule::FutureRequiredTypeAnnotation) {
                if !checker.semantic.future_annotations_or_stub()
                    && checker.target_version() < PythonVersion::PY39
                    && checker.semantic.in_annotation()
                    && checker.semantic.in_runtime_evaluated_annotation()
                    && !checker.semantic.in_string_type_definition()
                    && typing::is_pep585_generic(value, &checker.semantic)
                {
                    flake8_future_annotations::rules::future_required_type_annotation(
                        checker,
                        expr,
                        flake8_future_annotations::rules::Reason::PEP585,
                    );
                }
            }

            // Ex) Union[...]
            if checker.any_rule_enabled(&[
                Rule::UnnecessaryLiteralUnion,
                Rule::DuplicateUnionMember,
                Rule::RedundantLiteralUnion,
                Rule::UnnecessaryTypeUnion,
                Rule::NoneNotAtEndOfUnion,
            ]) {
                // Avoid duplicate checks if the parent is a union, since these rules already
                // traverse nested unions.
                if !checker.semantic.in_nested_union() {
                    if checker.is_rule_enabled(Rule::UnnecessaryLiteralUnion) {
                        flake8_pyi::rules::unnecessary_literal_union(checker, expr);
                    }
                    if checker.is_rule_enabled(Rule::DuplicateUnionMember)
                        // Avoid duplicate checks inside `Optional`
                        && !(
                            is_optional_as_none_in_union_enabled(checker.settings())
                            && checker.semantic.inside_optional()
                        )
                    {
                        flake8_pyi::rules::duplicate_union_member(checker, expr);
                    }
                    if checker.is_rule_enabled(Rule::RedundantLiteralUnion) {
                        flake8_pyi::rules::redundant_literal_union(checker, expr);
                    }
                    if checker.is_rule_enabled(Rule::UnnecessaryTypeUnion) {
                        flake8_pyi::rules::unnecessary_type_union(checker, expr);
                    }
                    if checker.is_rule_enabled(Rule::NoneNotAtEndOfUnion) {
                        ruff::rules::none_not_at_end_of_union(checker, expr);
                    }
                }
            }

            // Ex) Literal[...]
            if checker.any_rule_enabled(&[
                Rule::DuplicateLiteralMember,
                Rule::RedundantBoolLiteral,
                Rule::RedundantNoneLiteral,
                Rule::UnnecessaryNestedLiteral,
            ]) {
                if !checker.semantic.in_nested_literal() {
                    if checker.is_rule_enabled(Rule::DuplicateLiteralMember) {
                        flake8_pyi::rules::duplicate_literal_member(checker, expr);
                    }
                    if checker.is_rule_enabled(Rule::RedundantBoolLiteral) {
                        ruff::rules::redundant_bool_literal(checker, expr);
                    }
                    if checker.is_rule_enabled(Rule::RedundantNoneLiteral) {
                        flake8_pyi::rules::redundant_none_literal(checker, expr);
                    }
                    if checker.is_rule_enabled(Rule::UnnecessaryNestedLiteral) {
                        ruff::rules::unnecessary_nested_literal(checker, expr);
                    }
                }
            }

            if checker.is_rule_enabled(Rule::NeverUnion) {
                ruff::rules::never_union(checker, expr);
            }

            if checker.is_rule_enabled(Rule::UnnecessaryDefaultTypeArgs) {
                if checker.target_version() >= PythonVersion::PY313 {
                    pyupgrade::rules::unnecessary_default_type_args(checker, expr);
                }
            }

            if checker.any_rule_enabled(&[
                Rule::SysVersionSlice3,
                Rule::SysVersion2,
                Rule::SysVersion0,
                Rule::SysVersionSlice1,
            ]) {
                flake8_2020::rules::subscript(checker, value, slice);
            }
            if checker.is_rule_enabled(Rule::UncapitalizedEnvironmentVariables) {
                flake8_simplify::rules::use_capital_environment_variables(checker, expr);
            }
            if checker.is_rule_enabled(Rule::UnnecessaryIterableAllocationForFirstElement) {
                ruff::rules::unnecessary_iterable_allocation_for_first_element(checker, expr);
            }
            if checker.is_rule_enabled(Rule::InvalidIndexType) {
                ruff::rules::invalid_index_type(checker, subscript);
            }
            if checker.is_rule_enabled(Rule::SliceCopy) {
                refurb::rules::slice_copy(checker, subscript);
            }
            if checker.is_rule_enabled(Rule::PotentialIndexError) {
                pylint::rules::potential_index_error(checker, value, slice);
            }
            if checker.is_rule_enabled(Rule::SortedMinMax) {
                refurb::rules::sorted_min_max(checker, subscript);
            }
            if checker.is_rule_enabled(Rule::FStringNumberFormat) {
                refurb::rules::fstring_number_format(checker, subscript);
            }
            if checker.is_rule_enabled(Rule::IncorrectlyParenthesizedTupleInSubscript) {
                ruff::rules::subscript_with_parenthesized_tuple(checker, subscript);
            }
            if checker.is_rule_enabled(Rule::NonPEP646Unpack) {
                pyupgrade::rules::use_pep646_unpack(checker, subscript);
            }
            if checker.is_rule_enabled(Rule::Airflow3Removal) {
                airflow::rules::airflow_3_removal_expr(checker, expr);
            }
            if checker.is_rule_enabled(Rule::MissingMaxsplitArg) {
                pylint::rules::missing_maxsplit_arg(checker, value, slice, expr);
            }
            if checker.is_rule_enabled(Rule::AccessAnnotationsFromClassDict) {
                ruff::rules::access_annotations_from_class_dict_by_key(checker, subscript);
            }
            pandas_vet::rules::subscript(checker, value, expr);
        }
        Expr::Tuple(ast::ExprTuple {
            elts,
            ctx,
            range: _,
            node_index: _,
            parenthesized: _,
        })
        | Expr::List(ast::ExprList {
            elts,
            ctx,
            range: _,
            node_index: _,
        }) => {
            if ctx.is_store() {
                let check_too_many_expressions =
                    checker.is_rule_enabled(Rule::ExpressionsInStarAssignment);
                let check_two_starred_expressions =
                    checker.is_rule_enabled(Rule::MultipleStarredExpressions);
                pyflakes::rules::starred_expressions(
                    checker,
                    elts,
                    check_too_many_expressions,
                    check_two_starred_expressions,
                    expr.range(),
                );
            }
        }
        Expr::Name(ast::ExprName {
            id,
            ctx,
            range,
            node_index: _,
        }) => {
            match ctx {
                ExprContext::Load => {
                    if checker.is_rule_enabled(Rule::TypingTextStrAlias) {
                        pyupgrade::rules::typing_text_str_alias(checker, expr);
                    }
                    if checker.is_rule_enabled(Rule::NumpyDeprecatedTypeAlias) {
                        numpy::rules::deprecated_type_alias(checker, expr);
                    }
                    if checker.is_rule_enabled(Rule::NumpyDeprecatedFunction) {
                        numpy::rules::deprecated_function(checker, expr);
                    }
                    if checker.is_rule_enabled(Rule::Numpy2Deprecation) {
                        numpy::rules::numpy_2_0_deprecation(checker, expr);
                    }
                    if checker.is_rule_enabled(Rule::CollectionsNamedTuple) {
                        flake8_pyi::rules::collections_named_tuple(checker, expr);
                    }
                    if checker.is_rule_enabled(Rule::RegexFlagAlias) {
                        refurb::rules::regex_flag_alias(checker, expr);
                    }
                    if checker.is_rule_enabled(Rule::Airflow3Removal) {
                        airflow::rules::airflow_3_removal_expr(checker, expr);
                    }
                    if checker.is_rule_enabled(Rule::Airflow3SuggestedUpdate) {
                        airflow::rules::airflow_3_0_suggested_update_expr(checker, expr);
                    }
                    if checker.is_rule_enabled(Rule::Airflow3MovedToProvider) {
                        airflow::rules::moved_to_provider_in_3(checker, expr);
                    }
                    if checker.is_rule_enabled(Rule::Airflow3SuggestedToMoveToProvider) {
                        airflow::rules::suggested_to_move_to_provider_in_3(checker, expr);
                    }
                    if checker.any_rule_enabled(&[
                        Rule::SuspiciousPickleUsage,
                        Rule::SuspiciousMarshalUsage,
                        Rule::SuspiciousInsecureHashUsage,
                        Rule::SuspiciousInsecureCipherUsage,
                        Rule::SuspiciousInsecureCipherModeUsage,
                        Rule::SuspiciousMktempUsage,
                        Rule::SuspiciousEvalUsage,
                        Rule::SuspiciousMarkSafeUsage,
                        Rule::SuspiciousURLOpenUsage,
                        Rule::SuspiciousNonCryptographicRandomUsage,
                        Rule::SuspiciousTelnetUsage,
                        Rule::SuspiciousXMLCElementTreeUsage,
                        Rule::SuspiciousXMLElementTreeUsage,
                        Rule::SuspiciousXMLExpatReaderUsage,
                        Rule::SuspiciousXMLExpatBuilderUsage,
                        Rule::SuspiciousXMLSaxUsage,
                        Rule::SuspiciousXMLMiniDOMUsage,
                        Rule::SuspiciousXMLPullDOMUsage,
                        Rule::SuspiciousXMLETreeUsage,
                        Rule::SuspiciousFTPLibUsage,
                        Rule::SuspiciousUnverifiedContextUsage,
                    ]) {
                        flake8_bandit::rules::suspicious_function_reference(checker, expr);
                    }

                    // Ex) List[...]
                    if checker.any_rule_enabled(&[
                        Rule::FutureRewritableTypeAnnotation,
                        Rule::NonPEP585Annotation,
                    ]) {
                        if let Some(replacement) =
                            typing::to_pep585_generic(expr, &checker.semantic)
                        {
                            if checker.is_rule_enabled(Rule::FutureRewritableTypeAnnotation) {
                                if !checker.semantic.future_annotations_or_stub()
                                    && checker.target_version() < PythonVersion::PY39
                                    && checker.target_version() >= PythonVersion::PY37
                                    && checker.semantic.in_annotation()
                                    && !checker.settings().pyupgrade.keep_runtime_typing
                                {
                                    flake8_future_annotations::rules::future_rewritable_type_annotation(checker, expr);
                                }
                            }
                            if checker.is_rule_enabled(Rule::NonPEP585Annotation) {
                                if checker.source_type.is_stub()
                                    || checker.target_version() >= PythonVersion::PY39
                                    || (checker.target_version() >= PythonVersion::PY37
                                        && checker.semantic.future_annotations_or_stub()
                                        && checker.semantic.in_annotation()
                                        && !checker.settings().pyupgrade.keep_runtime_typing)
                                {
                                    pyupgrade::rules::use_pep585_annotation(
                                        checker,
                                        expr,
                                        &replacement,
                                    );
                                }
                            }
                        }
                    }
                }
                ExprContext::Store => {
                    if checker.is_rule_enabled(Rule::NonLowercaseVariableInFunction) {
                        if checker.semantic.current_scope().kind.is_function() {
                            pep8_naming::rules::non_lowercase_variable_in_function(
                                checker, expr, id,
                            );
                        }
                    }
                    if checker.is_rule_enabled(Rule::MixedCaseVariableInClassScope) {
                        if let ScopeKind::Class(class_def) = &checker.semantic.current_scope().kind
                        {
                            pep8_naming::rules::mixed_case_variable_in_class_scope(
                                checker, expr, id, class_def,
                            );
                        }
                    }
                    if checker.is_rule_enabled(Rule::Airflow3Removal) {
                        airflow::rules::airflow_3_removal_expr(checker, expr);
                    }
                    if checker.is_rule_enabled(Rule::MixedCaseVariableInGlobalScope) {
                        if matches!(checker.semantic.current_scope().kind, ScopeKind::Module) {
                            pep8_naming::rules::mixed_case_variable_in_global_scope(
                                checker, expr, id,
                            );
                        }
                    }
                    if checker.is_rule_enabled(Rule::AmbiguousVariableName) {
                        pycodestyle::rules::ambiguous_variable_name(checker, id, expr.range());
                    }
                    if !checker.semantic.current_scope().kind.is_class() {
                        if checker.is_rule_enabled(Rule::BuiltinVariableShadowing) {
                            flake8_builtins::rules::builtin_variable_shadowing(checker, id, *range);
                        }
                    }
                }
                _ => {}
            }
            if checker.is_rule_enabled(Rule::SixPY3) {
                flake8_2020::rules::name_or_attribute(checker, expr);
            }
            if checker.is_rule_enabled(Rule::UndocumentedWarn) {
                flake8_logging::rules::undocumented_warn(checker, expr);
            }
        }
        Expr::Attribute(attribute) => {
            if attribute.ctx == ExprContext::Load {
                if checker.any_rule_enabled(&[
                    Rule::SuspiciousPickleUsage,
                    Rule::SuspiciousMarshalUsage,
                    Rule::SuspiciousInsecureHashUsage,
                    Rule::SuspiciousInsecureCipherUsage,
                    Rule::SuspiciousInsecureCipherModeUsage,
                    Rule::SuspiciousMktempUsage,
                    Rule::SuspiciousEvalUsage,
                    Rule::SuspiciousMarkSafeUsage,
                    Rule::SuspiciousURLOpenUsage,
                    Rule::SuspiciousNonCryptographicRandomUsage,
                    Rule::SuspiciousTelnetUsage,
                    Rule::SuspiciousXMLCElementTreeUsage,
                    Rule::SuspiciousXMLElementTreeUsage,
                    Rule::SuspiciousXMLExpatReaderUsage,
                    Rule::SuspiciousXMLExpatBuilderUsage,
                    Rule::SuspiciousXMLSaxUsage,
                    Rule::SuspiciousXMLMiniDOMUsage,
                    Rule::SuspiciousXMLPullDOMUsage,
                    Rule::SuspiciousXMLETreeUsage,
                    Rule::SuspiciousFTPLibUsage,
                    Rule::SuspiciousUnverifiedContextUsage,
                ]) {
                    flake8_bandit::rules::suspicious_function_reference(checker, expr);
                }
            }

            // Ex) typing.List[...]
            if checker.any_rule_enabled(&[
                Rule::FutureRewritableTypeAnnotation,
                Rule::NonPEP585Annotation,
            ]) {
                if let Some(replacement) = typing::to_pep585_generic(expr, &checker.semantic) {
                    if checker.is_rule_enabled(Rule::FutureRewritableTypeAnnotation) {
                        if !checker.semantic.future_annotations_or_stub()
                            && checker.target_version() < PythonVersion::PY39
                            && checker.target_version() >= PythonVersion::PY37
                            && checker.semantic.in_annotation()
                            && !checker.settings().pyupgrade.keep_runtime_typing
                        {
                            flake8_future_annotations::rules::future_rewritable_type_annotation(
                                checker, expr,
                            );
                        }
                    }
                    if checker.is_rule_enabled(Rule::NonPEP585Annotation) {
                        if checker.source_type.is_stub()
                            || checker.target_version() >= PythonVersion::PY39
                            || (checker.target_version() >= PythonVersion::PY37
                                && checker.semantic.future_annotations_or_stub()
                                && checker.semantic.in_annotation()
                                && !checker.settings().pyupgrade.keep_runtime_typing)
                        {
                            pyupgrade::rules::use_pep585_annotation(checker, expr, &replacement);
                        }
                    }
                }
            }
            if checker.is_rule_enabled(Rule::RegexFlagAlias) {
                refurb::rules::regex_flag_alias(checker, expr);
            }
            if checker.is_rule_enabled(Rule::DatetimeTimezoneUTC) {
                if checker.target_version() >= PythonVersion::PY311 {
                    pyupgrade::rules::datetime_utc_alias(checker, expr);
                }
            }
            if checker.is_rule_enabled(Rule::TypingTextStrAlias) {
                pyupgrade::rules::typing_text_str_alias(checker, expr);
            }
            if checker.is_rule_enabled(Rule::NumpyDeprecatedTypeAlias) {
                numpy::rules::deprecated_type_alias(checker, expr);
            }
            if checker.is_rule_enabled(Rule::NumpyDeprecatedFunction) {
                numpy::rules::deprecated_function(checker, expr);
            }
            if checker.is_rule_enabled(Rule::Numpy2Deprecation) {
                numpy::rules::numpy_2_0_deprecation(checker, expr);
            }
            if checker.is_rule_enabled(Rule::DeprecatedMockImport) {
                pyupgrade::rules::deprecated_mock_attribute(checker, attribute);
            }
            if checker.is_rule_enabled(Rule::SixPY3) {
                flake8_2020::rules::name_or_attribute(checker, expr);
            }
            if checker.is_rule_enabled(Rule::DatetimeMinMax) {
                flake8_datetimez::rules::datetime_min_max(checker, expr);
            }
            if checker.is_rule_enabled(Rule::BannedApi) {
                flake8_tidy_imports::rules::banned_attribute_access(checker, expr);
            }
            if checker.is_rule_enabled(Rule::PrivateMemberAccess) {
                flake8_self::rules::private_member_access(checker, expr);
            }
            if checker.is_rule_enabled(Rule::CollectionsNamedTuple) {
                flake8_pyi::rules::collections_named_tuple(checker, expr);
            }
            if checker.is_rule_enabled(Rule::UndocumentedWarn) {
                flake8_logging::rules::undocumented_warn(checker, expr);
            }
            if checker.is_rule_enabled(Rule::PandasUseOfDotValues) {
                pandas_vet::rules::attr(checker, attribute);
            }
            if checker.is_rule_enabled(Rule::ByteStringUsage) {
                flake8_pyi::rules::bytestring_attribute(checker, expr);
            }
            if checker.is_rule_enabled(Rule::Airflow3Removal) {
                airflow::rules::airflow_3_removal_expr(checker, expr);
            }
            if checker.is_rule_enabled(Rule::Airflow3SuggestedUpdate) {
                airflow::rules::airflow_3_0_suggested_update_expr(checker, expr);
            }
            if checker.is_rule_enabled(Rule::Airflow3MovedToProvider) {
                airflow::rules::moved_to_provider_in_3(checker, expr);
            }
            if checker.is_rule_enabled(Rule::Airflow3SuggestedToMoveToProvider) {
                airflow::rules::suggested_to_move_to_provider_in_3(checker, expr);
            }
        }
        Expr::Call(
            call @ ast::ExprCall {
                func,
                arguments:
                    Arguments {
                        args,
                        keywords,
                        range: _,
                        node_index: _,
                    },
                range: _,
                node_index: _,
            },
        ) => {
            if checker.any_rule_enabled(&[
                // pylint
                Rule::BadStringFormatCharacter,
                // pyflakes
                Rule::StringDotFormatInvalidFormat,
                Rule::StringDotFormatExtraNamedArguments,
                Rule::StringDotFormatExtraPositionalArguments,
                Rule::StringDotFormatMissingArguments,
                Rule::StringDotFormatMixingAutomatic,
                // pyupgrade
                Rule::FormatLiterals,
                Rule::FString,
                // flynt
                Rule::StaticJoinToFString,
                // refurb
                Rule::HashlibDigestHex,
                // flake8-simplify
                Rule::SplitStaticString,
            ]) {
                if let Expr::Attribute(ast::ExprAttribute { value, attr, .. }) = func.as_ref() {
                    let attr = attr.as_str();
                    if let Expr::StringLiteral(ast::ExprStringLiteral {
                        value: string_value,
                        ..
                    }) = value.as_ref()
                    {
                        if attr == "join" {
                            // "...".join(...) call
                            if checker.is_rule_enabled(Rule::StaticJoinToFString) {
                                flynt::rules::static_join_to_fstring(
                                    checker,
                                    expr,
                                    string_value.to_str(),
                                );
                            }
                        } else if matches!(attr, "split" | "rsplit") {
                            // "...".split(...) call
                            if checker.is_rule_enabled(Rule::SplitStaticString) {
                                flake8_simplify::rules::split_static_string(
                                    checker,
                                    attr,
                                    call,
                                    string_value,
                                );
                            }
                        } else if attr == "format" {
                            // "...".format(...) call
                            let location = expr.range();
                            match pyflakes::format::FormatSummary::try_from(string_value.to_str()) {
                                Err(e) => {
                                    // F521
                                    checker.report_diagnostic_if_enabled(
                                        pyflakes::rules::StringDotFormatInvalidFormat {
                                            message: pyflakes::format::error_to_string(&e),
                                        },
                                        location,
                                    );
                                }
                                Ok(summary) => {
                                    if checker
                                        .is_rule_enabled(Rule::StringDotFormatExtraNamedArguments)
                                    {
                                        pyflakes::rules::string_dot_format_extra_named_arguments(
                                            checker, call, &summary, keywords,
                                        );
                                    }
                                    if checker.is_rule_enabled(
                                        Rule::StringDotFormatExtraPositionalArguments,
                                    ) {
                                        pyflakes::rules::string_dot_format_extra_positional_arguments(
                                            checker, call, &summary, args,
                                        );
                                    }
                                    if checker
                                        .is_rule_enabled(Rule::StringDotFormatMissingArguments)
                                    {
                                        pyflakes::rules::string_dot_format_missing_argument(
                                            checker, call, &summary, args, keywords,
                                        );
                                    }
                                    if checker.is_rule_enabled(Rule::StringDotFormatMixingAutomatic)
                                    {
                                        pyflakes::rules::string_dot_format_mixing_automatic(
                                            checker, call, &summary,
                                        );
                                    }
                                    if checker.is_rule_enabled(Rule::FormatLiterals) {
                                        pyupgrade::rules::format_literals(checker, call, &summary);
                                    }
                                    if checker.is_rule_enabled(Rule::FString) {
                                        pyupgrade::rules::f_strings(checker, call, &summary);
                                    }
                                }
                            }

                            if checker.is_rule_enabled(Rule::BadStringFormatCharacter) {
                                pylint::rules::bad_string_format_character::call(
                                    checker,
                                    string_value.to_str(),
                                    location,
                                );
                            }
                        }
                    }
                }
            }
            if checker.is_rule_enabled(Rule::SuperWithoutBrackets) {
                pylint::rules::super_without_brackets(checker, func);
            }
            if checker.is_rule_enabled(Rule::LenTest) {
                pylint::rules::len_test(checker, call);
            }
            if checker.is_rule_enabled(Rule::BitCount) {
                refurb::rules::bit_count(checker, call);
            }
            if checker.is_rule_enabled(Rule::TypeOfPrimitive) {
                pyupgrade::rules::type_of_primitive(checker, expr, func, args);
            }
            if checker.is_rule_enabled(Rule::DeprecatedUnittestAlias) {
                pyupgrade::rules::deprecated_unittest_alias(checker, func);
            }
            if checker.is_rule_enabled(Rule::SuperCallWithParameters) {
                pyupgrade::rules::super_call_with_parameters(checker, call);
            }
            if checker.is_rule_enabled(Rule::UnnecessaryEncodeUTF8) {
                pyupgrade::rules::unnecessary_encode_utf8(checker, call);
            }
            if checker.is_rule_enabled(Rule::RedundantOpenModes) {
                pyupgrade::rules::redundant_open_modes(checker, call);
            }
            if checker.is_rule_enabled(Rule::NativeLiterals) {
                pyupgrade::rules::native_literals(
                    checker,
                    call,
                    checker.semantic().current_expression_parent(),
                );
            }
            if checker.is_rule_enabled(Rule::OpenAlias) {
                pyupgrade::rules::open_alias(checker, expr, func);
            }
            if checker.is_rule_enabled(Rule::ReplaceUniversalNewlines) {
                pyupgrade::rules::replace_universal_newlines(checker, call);
            }
            if checker.is_rule_enabled(Rule::ReplaceStdoutStderr) {
                pyupgrade::rules::replace_stdout_stderr(checker, call);
            }
            if checker.is_rule_enabled(Rule::OSErrorAlias) {
                pyupgrade::rules::os_error_alias_call(checker, func);
            }
            if checker.is_rule_enabled(Rule::TimeoutErrorAlias) {
                if checker.target_version() >= PythonVersion::PY310 {
                    pyupgrade::rules::timeout_error_alias_call(checker, func);
                }
            }
            if checker.is_rule_enabled(Rule::NonPEP604Isinstance) {
                if checker.target_version() >= PythonVersion::PY310 {
                    pyupgrade::rules::use_pep604_isinstance(checker, expr, func, args);
                }
            }
            if checker.is_rule_enabled(Rule::BlockingHttpCallInAsyncFunction) {
                flake8_async::rules::blocking_http_call(checker, call);
            }
            if checker.is_rule_enabled(Rule::BlockingOpenCallInAsyncFunction) {
                flake8_async::rules::blocking_open_call(checker, call);
            }
            if checker.any_rule_enabled(&[
                Rule::CreateSubprocessInAsyncFunction,
                Rule::RunProcessInAsyncFunction,
                Rule::WaitForProcessInAsyncFunction,
            ]) {
                flake8_async::rules::blocking_process_invocation(checker, call);
            }
            if checker.is_rule_enabled(Rule::BlockingSleepInAsyncFunction) {
                flake8_async::rules::blocking_sleep(checker, call);
            }
            if checker.is_rule_enabled(Rule::LongSleepNotForever) {
                flake8_async::rules::long_sleep_not_forever(checker, call);
            }
            if checker.any_rule_enabled(&[Rule::Print, Rule::PPrint]) {
                flake8_print::rules::print_call(checker, call);
            }
            if checker.any_rule_enabled(&[
                Rule::SuspiciousPickleUsage,
                Rule::SuspiciousMarshalUsage,
                Rule::SuspiciousInsecureHashUsage,
                Rule::SuspiciousInsecureCipherUsage,
                Rule::SuspiciousInsecureCipherModeUsage,
                Rule::SuspiciousMktempUsage,
                Rule::SuspiciousEvalUsage,
                Rule::SuspiciousMarkSafeUsage,
                Rule::SuspiciousURLOpenUsage,
                Rule::SuspiciousNonCryptographicRandomUsage,
                Rule::SuspiciousXMLCElementTreeUsage,
                Rule::SuspiciousXMLElementTreeUsage,
                Rule::SuspiciousXMLExpatReaderUsage,
                Rule::SuspiciousXMLExpatBuilderUsage,
                Rule::SuspiciousXMLSaxUsage,
                Rule::SuspiciousXMLMiniDOMUsage,
                Rule::SuspiciousXMLPullDOMUsage,
                Rule::SuspiciousXMLETreeUsage,
                Rule::SuspiciousUnverifiedContextUsage,
                Rule::SuspiciousTelnetUsage,
                Rule::SuspiciousFTPLibUsage,
            ]) {
                flake8_bandit::rules::suspicious_function_call(checker, call);
            }
            if checker.is_rule_enabled(Rule::ReSubPositionalArgs) {
                flake8_bugbear::rules::re_sub_positional_args(checker, call);
            }
            if checker.is_rule_enabled(Rule::UnreliableCallableCheck) {
                flake8_bugbear::rules::unreliable_callable_check(checker, expr, func, args);
            }
            if checker.is_rule_enabled(Rule::StripWithMultiCharacters) {
                flake8_bugbear::rules::strip_with_multi_characters(checker, expr, func, args);
            }
            if checker.is_rule_enabled(Rule::GetAttrWithConstant) {
                flake8_bugbear::rules::getattr_with_constant(checker, expr, func, args);
            }
            if checker.is_rule_enabled(Rule::SetAttrWithConstant) {
                flake8_bugbear::rules::setattr_with_constant(checker, expr, func, args);
            }
            if checker.is_rule_enabled(Rule::UselessContextlibSuppress) {
                flake8_bugbear::rules::useless_contextlib_suppress(checker, expr, func, args);
            }
            if checker.is_rule_enabled(Rule::StarArgUnpackingAfterKeywordArg) {
                flake8_bugbear::rules::star_arg_unpacking_after_keyword_arg(
                    checker, args, keywords,
                );
            }
            if checker.is_rule_enabled(Rule::ZipWithoutExplicitStrict) {
                if checker.target_version() >= PythonVersion::PY310 {
                    flake8_bugbear::rules::zip_without_explicit_strict(checker, call);
                }
            }
            if checker.is_rule_enabled(Rule::NoExplicitStacklevel) {
                flake8_bugbear::rules::no_explicit_stacklevel(checker, call);
            }
            if checker.is_rule_enabled(Rule::MutableContextvarDefault) {
                flake8_bugbear::rules::mutable_contextvar_default(checker, call);
            }
            if checker.is_rule_enabled(Rule::UnnecessaryDictKwargs) {
                flake8_pie::rules::unnecessary_dict_kwargs(checker, call);
            }
            if checker.is_rule_enabled(Rule::UnnecessaryRangeStart) {
                flake8_pie::rules::unnecessary_range_start(checker, call);
            }
            if checker.is_rule_enabled(Rule::ExecBuiltin) {
                flake8_bandit::rules::exec_used(checker, func);
            }
            if checker.is_rule_enabled(Rule::BadFilePermissions) {
                flake8_bandit::rules::bad_file_permissions(checker, call);
            }
            if checker.is_rule_enabled(Rule::RequestWithNoCertValidation) {
                flake8_bandit::rules::request_with_no_cert_validation(checker, call);
            }
            if checker.is_rule_enabled(Rule::UnsafeYAMLLoad) {
                flake8_bandit::rules::unsafe_yaml_load(checker, call);
            }
            if checker.is_rule_enabled(Rule::SnmpInsecureVersion) {
                flake8_bandit::rules::snmp_insecure_version(checker, call);
            }
            if checker.is_rule_enabled(Rule::SnmpWeakCryptography) {
                flake8_bandit::rules::snmp_weak_cryptography(checker, call);
            }
            if checker.is_rule_enabled(Rule::Jinja2AutoescapeFalse) {
                flake8_bandit::rules::jinja2_autoescape_false(checker, call);
            }
            if checker.is_rule_enabled(Rule::MakoTemplates) {
                flake8_bandit::rules::mako_templates(checker, call);
            }
            if checker.is_rule_enabled(Rule::HardcodedPasswordFuncArg) {
                flake8_bandit::rules::hardcoded_password_func_arg(checker, keywords);
            }
            if checker.is_rule_enabled(Rule::HardcodedSQLExpression) {
                flake8_bandit::rules::hardcoded_sql_expression(checker, expr);
            }
            if checker.is_rule_enabled(Rule::HashlibInsecureHashFunction) {
                flake8_bandit::rules::hashlib_insecure_hash_functions(checker, call);
            }
            if checker.is_rule_enabled(Rule::HashlibDigestHex) {
                refurb::rules::hashlib_digest_hex(checker, call);
            }
            if checker.is_rule_enabled(Rule::RequestWithoutTimeout) {
                flake8_bandit::rules::request_without_timeout(checker, call);
            }
            if checker.is_rule_enabled(Rule::ParamikoCall) {
                flake8_bandit::rules::paramiko_call(checker, func);
            }
            if checker.is_rule_enabled(Rule::SSHNoHostKeyVerification) {
                flake8_bandit::rules::ssh_no_host_key_verification(checker, call);
            }
            if checker.is_rule_enabled(Rule::LoggingConfigInsecureListen) {
                flake8_bandit::rules::logging_config_insecure_listen(checker, call);
            }
            if checker.is_rule_enabled(Rule::FlaskDebugTrue) {
                flake8_bandit::rules::flask_debug_true(checker, call);
            }
            if checker.is_rule_enabled(Rule::WeakCryptographicKey) {
                flake8_bandit::rules::weak_cryptographic_key(checker, call);
            }
            if checker.any_rule_enabled(&[
                Rule::SubprocessWithoutShellEqualsTrue,
                Rule::SubprocessPopenWithShellEqualsTrue,
                Rule::CallWithShellEqualsTrue,
                Rule::StartProcessWithAShell,
                Rule::StartProcessWithNoShell,
                Rule::StartProcessWithPartialPath,
                Rule::UnixCommandWildcardInjection,
            ]) {
                flake8_bandit::rules::shell_injection(checker, call);
            }
            if checker.is_rule_enabled(Rule::DjangoExtra) {
                flake8_bandit::rules::django_extra(checker, call);
            }
            if checker.is_rule_enabled(Rule::DjangoRawSql) {
                flake8_bandit::rules::django_raw_sql(checker, call);
            }
            if checker.is_rule_enabled(Rule::TarfileUnsafeMembers) {
                flake8_bandit::rules::tarfile_unsafe_members(checker, call);
            }
            if checker.is_rule_enabled(Rule::UnnecessaryGeneratorList) {
                flake8_comprehensions::rules::unnecessary_generator_list(checker, call);
            }
            if checker.is_rule_enabled(Rule::UnnecessaryGeneratorSet) {
                flake8_comprehensions::rules::unnecessary_generator_set(checker, call);
            }
            if checker.is_rule_enabled(Rule::UnnecessaryGeneratorDict) {
                flake8_comprehensions::rules::unnecessary_generator_dict(
                    checker, expr, func, args, keywords,
                );
            }
            if checker.is_rule_enabled(Rule::UnnecessaryListComprehensionSet) {
                flake8_comprehensions::rules::unnecessary_list_comprehension_set(checker, call);
            }
            if checker.is_rule_enabled(Rule::UnnecessaryListComprehensionDict) {
                flake8_comprehensions::rules::unnecessary_list_comprehension_dict(
                    checker, expr, func, args, keywords,
                );
            }
            if checker.is_rule_enabled(Rule::UnnecessaryLiteralSet) {
                flake8_comprehensions::rules::unnecessary_literal_set(checker, call);
            }
            if checker.is_rule_enabled(Rule::UnnecessaryLiteralDict) {
                flake8_comprehensions::rules::unnecessary_literal_dict(
                    checker, expr, func, args, keywords,
                );
            }
            if checker.is_rule_enabled(Rule::UnnecessaryCollectionCall) {
                flake8_comprehensions::rules::unnecessary_collection_call(
                    checker,
                    call,
                    &checker.settings().flake8_comprehensions,
                );
            }
            if checker.is_rule_enabled(Rule::UnnecessaryLiteralWithinTupleCall) {
                flake8_comprehensions::rules::unnecessary_literal_within_tuple_call(
                    checker, expr, call,
                );
            }
            if checker.is_rule_enabled(Rule::UnnecessaryLiteralWithinListCall) {
                flake8_comprehensions::rules::unnecessary_literal_within_list_call(checker, call);
            }
            if checker.is_rule_enabled(Rule::UnnecessaryLiteralWithinDictCall) {
                flake8_comprehensions::rules::unnecessary_literal_within_dict_call(checker, call);
            }
            if checker.is_rule_enabled(Rule::UnnecessaryListCall) {
                flake8_comprehensions::rules::unnecessary_list_call(checker, expr, call);
            }
            if checker.is_rule_enabled(Rule::UnnecessaryCallAroundSorted) {
                flake8_comprehensions::rules::unnecessary_call_around_sorted(
                    checker, expr, func, args,
                );
            }
            if checker.is_rule_enabled(Rule::UnnecessaryDoubleCastOrProcess) {
                flake8_comprehensions::rules::unnecessary_double_cast_or_process(
                    checker, expr, func, args, keywords,
                );
            }
            if checker.is_rule_enabled(Rule::UnnecessarySubscriptReversal) {
                flake8_comprehensions::rules::unnecessary_subscript_reversal(checker, call);
            }
            if checker.is_rule_enabled(Rule::UnnecessaryMap) {
                flake8_comprehensions::rules::unnecessary_map(checker, call);
            }
            if checker.is_rule_enabled(Rule::UnnecessaryComprehensionInCall) {
                flake8_comprehensions::rules::unnecessary_comprehension_in_call(
                    checker, expr, func, args, keywords,
                );
            }
            if checker.is_rule_enabled(Rule::BooleanPositionalValueInCall) {
                flake8_boolean_trap::rules::boolean_positional_value_in_call(checker, call);
            }
            if checker.is_rule_enabled(Rule::Debugger) {
                flake8_debugger::rules::debugger_call(checker, expr, func);
            }
            if checker.is_rule_enabled(Rule::PandasUseOfInplaceArgument) {
                pandas_vet::rules::inplace_argument(checker, call);
            }
            pandas_vet::rules::call(checker, func);
            if checker.is_rule_enabled(Rule::PandasUseOfDotReadTable) {
                pandas_vet::rules::use_of_read_table(checker, call);
            }
            if checker.is_rule_enabled(Rule::PandasUseOfPdMerge) {
                pandas_vet::rules::use_of_pd_merge(checker, func);
            }
            if checker.is_rule_enabled(Rule::CallDatetimeWithoutTzinfo) {
                flake8_datetimez::rules::call_datetime_without_tzinfo(checker, call);
            }
            if checker.is_rule_enabled(Rule::CallDatetimeToday) {
                flake8_datetimez::rules::call_datetime_today(checker, func, expr.range());
            }
            if checker.is_rule_enabled(Rule::CallDatetimeUtcnow) {
                flake8_datetimez::rules::call_datetime_utcnow(checker, func, expr.range());
            }
            if checker.is_rule_enabled(Rule::CallDatetimeUtcfromtimestamp) {
                flake8_datetimez::rules::call_datetime_utcfromtimestamp(
                    checker,
                    func,
                    expr.range(),
                );
            }
            if checker.is_rule_enabled(Rule::CallDatetimeNowWithoutTzinfo) {
                flake8_datetimez::rules::call_datetime_now_without_tzinfo(checker, call);
            }
            if checker.is_rule_enabled(Rule::CallDatetimeFromtimestamp) {
                flake8_datetimez::rules::call_datetime_fromtimestamp(checker, call);
            }
            if checker.is_rule_enabled(Rule::CallDatetimeStrptimeWithoutZone) {
                flake8_datetimez::rules::call_datetime_strptime_without_zone(checker, call);
            }
            if checker.is_rule_enabled(Rule::CallDateToday) {
                flake8_datetimez::rules::call_date_today(checker, func, expr.range());
            }
            if checker.is_rule_enabled(Rule::CallDateFromtimestamp) {
                flake8_datetimez::rules::call_date_fromtimestamp(checker, func, expr.range());
            }
            if checker.is_rule_enabled(Rule::UnnecessaryDirectLambdaCall) {
                pylint::rules::unnecessary_direct_lambda_call(checker, expr, func);
            }
            if checker.is_rule_enabled(Rule::SysExitAlias) {
                pylint::rules::sys_exit_alias(checker, call);
            }
            if checker.is_rule_enabled(Rule::BadOpenMode) {
                pylint::rules::bad_open_mode(checker, call);
            }
            if checker.is_rule_enabled(Rule::BadStrStripCall) {
                pylint::rules::bad_str_strip_call(checker, call);
            }
            if checker.is_rule_enabled(Rule::ShallowCopyEnviron) {
                pylint::rules::shallow_copy_environ(checker, call);
            }
            if checker.is_rule_enabled(Rule::InvalidEnvvarDefault) {
                pylint::rules::invalid_envvar_default(checker, call);
            }
            if checker.is_rule_enabled(Rule::InvalidEnvvarValue) {
                pylint::rules::invalid_envvar_value(checker, call);
            }
            if checker.is_rule_enabled(Rule::NestedMinMax) {
                pylint::rules::nested_min_max(checker, expr, func, args, keywords);
            }
            if checker.is_rule_enabled(Rule::RepeatedKeywordArgument) {
                pylint::rules::repeated_keyword_argument(checker, call);
            }
            if checker.is_rule_enabled(Rule::PytestPatchWithLambda) {
                flake8_pytest_style::rules::patch_with_lambda(checker, call);
            }
            if checker.any_rule_enabled(&[
                Rule::PytestParametrizeNamesWrongType,
                Rule::PytestParametrizeValuesWrongType,
                Rule::PytestDuplicateParametrizeTestCases,
            ]) {
                flake8_pytest_style::rules::parametrize(checker, call);
            }
            if checker.is_rule_enabled(Rule::PytestUnittestAssertion) {
                flake8_pytest_style::rules::unittest_assertion(checker, expr, func, args, keywords);
            }
            if checker.is_rule_enabled(Rule::PytestUnittestRaisesAssertion) {
                flake8_pytest_style::rules::unittest_raises_assertion_call(checker, call);
            }
            if checker.is_rule_enabled(Rule::SubprocessPopenPreexecFn) {
                pylint::rules::subprocess_popen_preexec_fn(checker, call);
            }
            if checker.is_rule_enabled(Rule::SubprocessRunWithoutCheck) {
                pylint::rules::subprocess_run_without_check(checker, call);
            }
            if checker.is_rule_enabled(Rule::UnspecifiedEncoding) {
                pylint::rules::unspecified_encoding(checker, call);
            }
            if checker.any_rule_enabled(&[
                Rule::PytestRaisesWithoutException,
                Rule::PytestRaisesTooBroad,
            ]) {
                flake8_pytest_style::rules::raises_call(checker, call);
            }
            if checker.is_rule_enabled(Rule::LegacyFormPytestRaises) {
                ruff::rules::legacy_raises_warns_deprecated_call(checker, call);
            }
            if checker
                .any_rule_enabled(&[Rule::PytestWarnsWithoutWarning, Rule::PytestWarnsTooBroad])
            {
                flake8_pytest_style::rules::warns_call(checker, call);
            }
            if checker.is_rule_enabled(Rule::PytestFailWithoutMessage) {
                flake8_pytest_style::rules::fail_call(checker, call);
            }
            if checker.is_rule_enabled(Rule::ZipInsteadOfPairwise) {
                if checker.target_version() >= PythonVersion::PY310 {
                    ruff::rules::zip_instead_of_pairwise(checker, call);
                }
            }
            if checker.any_rule_enabled(&[
                Rule::FStringInGetTextFuncCall,
                Rule::FormatInGetTextFuncCall,
                Rule::PrintfInGetTextFuncCall,
            ]) && flake8_gettext::is_gettext_func_call(
                func,
                &checker.settings().flake8_gettext.functions_names,
            ) {
                if checker.is_rule_enabled(Rule::FStringInGetTextFuncCall) {
                    flake8_gettext::rules::f_string_in_gettext_func_call(checker, args);
                }
                if checker.is_rule_enabled(Rule::FormatInGetTextFuncCall) {
                    flake8_gettext::rules::format_in_gettext_func_call(checker, args);
                }
                if checker.is_rule_enabled(Rule::PrintfInGetTextFuncCall) {
                    flake8_gettext::rules::printf_in_gettext_func_call(checker, args);
                }
            }
            if checker.is_rule_enabled(Rule::UncapitalizedEnvironmentVariables) {
                flake8_simplify::rules::use_capital_environment_variables(checker, expr);
            }
            if checker.is_rule_enabled(Rule::OpenFileWithContextHandler) {
                flake8_simplify::rules::open_file_with_context_handler(checker, call);
            }
            if checker.is_rule_enabled(Rule::DictGetWithNoneDefault) {
                flake8_simplify::rules::dict_get_with_none_default(checker, expr);
            }
            if checker.is_rule_enabled(Rule::ZipDictKeysAndValues) {
                flake8_simplify::rules::zip_dict_keys_and_values(checker, call);
            }
            if checker.any_rule_enabled(&[
                Rule::OsMkdir,
                Rule::OsMakedirs,
                Rule::OsStat,
                Rule::OsPathJoin,
                Rule::OsPathSplitext,
                Rule::BuiltinOpen,
                Rule::PyPath,
                Rule::Glob,
                Rule::OsListdir,
                Rule::OsSymlink,
            ]) {
                flake8_use_pathlib::rules::replaceable_by_pathlib(checker, call);
            }
            if let Some(qualified_name) = checker.semantic().resolve_qualified_name(&call.func) {
                let segments = qualified_name.segments();
                if checker.is_rule_enabled(Rule::OsPathGetsize) {
                    flake8_use_pathlib::rules::os_path_getsize(checker, call, segments);
                }
                if checker.is_rule_enabled(Rule::OsPathGetatime) {
                    flake8_use_pathlib::rules::os_path_getatime(checker, call, segments);
                }
                if checker.is_rule_enabled(Rule::OsPathGetctime) {
                    flake8_use_pathlib::rules::os_path_getctime(checker, call, segments);
                }
                if checker.is_rule_enabled(Rule::OsPathGetmtime) {
                    flake8_use_pathlib::rules::os_path_getmtime(checker, call, segments);
                }
                if checker.is_rule_enabled(Rule::OsPathAbspath) {
                    flake8_use_pathlib::rules::os_path_abspath(checker, call, segments);
                }
                if checker.is_rule_enabled(Rule::OsRmdir) {
                    flake8_use_pathlib::rules::os_rmdir(checker, call, segments);
                }
                if checker.is_rule_enabled(Rule::OsRemove) {
                    flake8_use_pathlib::rules::os_remove(checker, call, segments);
                }
                if checker.is_rule_enabled(Rule::OsUnlink) {
                    flake8_use_pathlib::rules::os_unlink(checker, call, segments);
                }
                if checker.is_rule_enabled(Rule::OsPathExists) {
                    flake8_use_pathlib::rules::os_path_exists(checker, call, segments);
                }
                if checker.is_rule_enabled(Rule::OsPathExpanduser) {
                    flake8_use_pathlib::rules::os_path_expanduser(checker, call, segments);
                }
                if checker.is_rule_enabled(Rule::OsPathBasename) {
                    flake8_use_pathlib::rules::os_path_basename(checker, call, segments);
                }
                if checker.is_rule_enabled(Rule::OsPathDirname) {
                    flake8_use_pathlib::rules::os_path_dirname(checker, call, segments);
                }
                if checker.is_rule_enabled(Rule::OsPathIsabs) {
                    flake8_use_pathlib::rules::os_path_isabs(checker, call, segments);
                }
                if checker.is_rule_enabled(Rule::OsPathIsdir) {
                    flake8_use_pathlib::rules::os_path_isdir(checker, call, segments);
                }
                if checker.is_rule_enabled(Rule::OsPathIsfile) {
                    flake8_use_pathlib::rules::os_path_isfile(checker, call, segments);
                }
                if checker.is_rule_enabled(Rule::OsPathIslink) {
                    flake8_use_pathlib::rules::os_path_islink(checker, call, segments);
                }
                if checker.is_rule_enabled(Rule::OsReadlink) {
                    flake8_use_pathlib::rules::os_readlink(checker, call, segments);
                }
                if checker.is_rule_enabled(Rule::OsGetcwd) {
                    flake8_use_pathlib::rules::os_getcwd(checker, call, segments);
                }
                if checker.is_rule_enabled(Rule::OsChmod) {
                    flake8_use_pathlib::rules::os_chmod(checker, call, segments);
                }
                if checker.is_rule_enabled(Rule::OsRename) {
                    flake8_use_pathlib::rules::os_rename(checker, call, segments);
                }
                if checker.is_rule_enabled(Rule::OsReplace) {
                    flake8_use_pathlib::rules::os_replace(checker, call, segments);
                }
                if checker.is_rule_enabled(Rule::OsPathSamefile) {
                    flake8_use_pathlib::rules::os_path_samefile(checker, call, segments);
                }
                if checker.is_rule_enabled(Rule::PathConstructorCurrentDirectory) {
                    flake8_use_pathlib::rules::path_constructor_current_directory(
                        checker, call, segments,
                    );
                }
            }

            if checker.is_rule_enabled(Rule::OsSepSplit) {
                flake8_use_pathlib::rules::os_sep_split(checker, call);
            }
            if checker.is_rule_enabled(Rule::NumpyLegacyRandom) {
                numpy::rules::legacy_random(checker, func);
            }
            if checker.any_rule_enabled(&[
                Rule::LoggingStringFormat,
                Rule::LoggingPercentFormat,
                Rule::LoggingStringConcat,
                Rule::LoggingFString,
                Rule::LoggingWarn,
                Rule::LoggingExtraAttrClash,
                Rule::LoggingExcInfo,
                Rule::LoggingRedundantExcInfo,
            ]) {
                flake8_logging_format::rules::logging_call(checker, call);
            }
            if checker.any_rule_enabled(&[Rule::LoggingTooFewArgs, Rule::LoggingTooManyArgs]) {
                pylint::rules::logging_call(checker, call);
            }
            if checker.is_rule_enabled(Rule::DjangoLocalsInRenderFunction) {
                flake8_django::rules::locals_in_render_function(checker, call);
            }
            if checker.is_rule_enabled(Rule::UnsupportedMethodCallOnAll) {
                flake8_pyi::rules::unsupported_method_call_on_all(checker, func);
            }
            if checker.is_rule_enabled(Rule::PrintEmptyString) {
                refurb::rules::print_empty_string(checker, call);
            }
            if checker.is_rule_enabled(Rule::RedundantLogBase) {
                refurb::rules::redundant_log_base(checker, call);
            }
            if checker.is_rule_enabled(Rule::VerboseDecimalConstructor) {
                refurb::rules::verbose_decimal_constructor(checker, call);
            }
            if checker.is_rule_enabled(Rule::UnnecessaryFromFloat) {
                refurb::rules::unnecessary_from_float(checker, call);
            }
            if checker.is_rule_enabled(Rule::QuadraticListSummation) {
                ruff::rules::quadratic_list_summation(checker, call);
            }
            if checker.is_rule_enabled(Rule::DirectLoggerInstantiation) {
                flake8_logging::rules::direct_logger_instantiation(checker, call);
            }
            if checker.is_rule_enabled(Rule::InvalidGetLoggerArgument) {
                flake8_logging::rules::invalid_get_logger_argument(checker, call);
            }
            if checker.is_rule_enabled(Rule::ExceptionWithoutExcInfo) {
                flake8_logging::rules::exception_without_exc_info(checker, call);
            }
            if checker.is_rule_enabled(Rule::RootLoggerCall) {
                flake8_logging::rules::root_logger_call(checker, call);
            }
            if checker.is_rule_enabled(Rule::IsinstanceTypeNone) {
                refurb::rules::isinstance_type_none(checker, call);
            }
            if checker.is_rule_enabled(Rule::ImplicitCwd) {
                refurb::rules::no_implicit_cwd(checker, call);
            }
            if checker.is_rule_enabled(Rule::TrioSyncCall) {
                flake8_async::rules::sync_call(checker, call);
            }
            if checker.is_rule_enabled(Rule::AsyncZeroSleep) {
                flake8_async::rules::async_zero_sleep(checker, call);
            }
            if checker.is_rule_enabled(Rule::UnnecessaryDunderCall) {
                pylint::rules::unnecessary_dunder_call(checker, call);
            }
            if checker.is_rule_enabled(Rule::SslWithNoVersion) {
                flake8_bandit::rules::ssl_with_no_version(checker, call);
            }
            if checker.is_rule_enabled(Rule::SslInsecureVersion) {
                flake8_bandit::rules::ssl_insecure_version(checker, call);
            }
            if checker.is_rule_enabled(Rule::MutableFromkeysValue) {
                ruff::rules::mutable_fromkeys_value(checker, call);
            }
            if checker.is_rule_enabled(Rule::UnsortedDunderAll) {
                ruff::rules::sort_dunder_all_extend_call(checker, call);
            }
            if checker.is_rule_enabled(Rule::DefaultFactoryKwarg) {
                ruff::rules::default_factory_kwarg(checker, call);
            }
            if checker.is_rule_enabled(Rule::UnnecessaryIterableAllocationForFirstElement) {
                ruff::rules::unnecessary_iterable_allocation_for_first_element(checker, expr);
            }
            if checker.is_rule_enabled(Rule::DecimalFromFloatLiteral) {
                ruff::rules::decimal_from_float_literal_syntax(checker, call);
            }
            if checker.is_rule_enabled(Rule::IntOnSlicedStr) {
                refurb::rules::int_on_sliced_str(checker, call);
            }
            if checker.is_rule_enabled(Rule::UnsafeMarkupUse) {
                flake8_bandit::rules::unsafe_markup_call(checker, call);
            }
            if checker.is_rule_enabled(Rule::MapIntVersionParsing) {
                ruff::rules::map_int_version_parsing(checker, call);
            }
            if checker.is_rule_enabled(Rule::UnrawRePattern) {
                ruff::rules::unraw_re_pattern(checker, call);
            }
            if checker.is_rule_enabled(Rule::AirflowDagNoScheduleArgument) {
                airflow::rules::dag_no_schedule_argument(checker, expr);
            }
            if checker.is_rule_enabled(Rule::UnnecessaryRegularExpression) {
                ruff::rules::unnecessary_regular_expression(checker, call);
            }
            if checker.is_rule_enabled(Rule::Airflow3Removal) {
                airflow::rules::airflow_3_removal_expr(checker, expr);
            }
            if checker.is_rule_enabled(Rule::Airflow3SuggestedUpdate) {
                airflow::rules::airflow_3_0_suggested_update_expr(checker, expr);
            }
            if checker.is_rule_enabled(Rule::UnnecessaryCastToInt) {
                ruff::rules::unnecessary_cast_to_int(checker, call);
            }
            if checker.is_rule_enabled(Rule::InvalidPathlibWithSuffix) {
                flake8_use_pathlib::rules::invalid_pathlib_with_suffix(checker, call);
            }
            if checker.is_rule_enabled(Rule::BatchedWithoutExplicitStrict) {
                flake8_bugbear::rules::batched_without_explicit_strict(checker, call);
            }
            if checker.is_rule_enabled(Rule::PytestRaisesAmbiguousPattern) {
                ruff::rules::pytest_raises_ambiguous_pattern(checker, call);
            }
            if checker.is_rule_enabled(Rule::FalsyDictGetFallback) {
                ruff::rules::falsy_dict_get_fallback(checker, expr);
            }
            if checker.is_rule_enabled(Rule::UnnecessaryRound) {
                ruff::rules::unnecessary_round(checker, call);
            }
            if checker.is_rule_enabled(Rule::UnnecessaryEmptyIterableWithinDequeCall) {
                ruff::rules::unnecessary_literal_within_deque_call(checker, call);
            }
            if checker.is_rule_enabled(Rule::StarmapZip) {
                ruff::rules::starmap_zip(checker, call);
            }
            if checker.is_rule_enabled(Rule::AccessAnnotationsFromClassDict) {
                ruff::rules::access_annotations_from_class_dict_with_get(checker, call);
            }
            if checker.is_rule_enabled(Rule::LogExceptionOutsideExceptHandler) {
                flake8_logging::rules::log_exception_outside_except_handler(checker, call);
            }
            if checker.is_rule_enabled(Rule::ExcInfoOutsideExceptHandler) {
                flake8_logging::rules::exc_info_outside_except_handler(checker, call);
            }
            if checker.is_rule_enabled(Rule::FromisoformatReplaceZ) {
                refurb::rules::fromisoformat_replace_z(checker, call);
            }
            if checker.is_rule_enabled(Rule::NonOctalPermissions) {
                ruff::rules::non_octal_permissions(checker, call);
            }
            if checker.is_rule_enabled(Rule::AssertRaisesException)
                && is_assert_raises_exception_call_enabled(checker.settings())
            {
                flake8_bugbear::rules::assert_raises_exception_call(checker, call);
            }
        }
        Expr::Dict(dict) => {
            if checker.any_rule_enabled(&[
                Rule::MultiValueRepeatedKeyLiteral,
                Rule::MultiValueRepeatedKeyVariable,
            ]) {
                pyflakes::rules::repeated_keys(checker, dict);
            }
            if checker.is_rule_enabled(Rule::UnnecessarySpread) {
                flake8_pie::rules::unnecessary_spread(checker, dict);
            }
        }
        Expr::Set(set) => {
            if checker.is_rule_enabled(Rule::DuplicateValue) {
                flake8_bugbear::rules::duplicate_value(checker, set);
            }
        }
        Expr::Yield(_) => {
            if checker.is_rule_enabled(Rule::YieldInInit) {
                pylint::rules::yield_in_init(checker, expr);
            }
        }
        Expr::YieldFrom(yield_from) => {
            if checker.is_rule_enabled(Rule::YieldInInit) {
                pylint::rules::yield_in_init(checker, expr);
            }
            if checker.is_rule_enabled(Rule::YieldFromInAsyncFunction) {
                pylint::rules::yield_from_in_async_function(checker, yield_from);
            }
        }
        Expr::FString(f_string_expr @ ast::ExprFString { value, .. }) => {
            if checker.is_rule_enabled(Rule::FStringMissingPlaceholders) {
                pyflakes::rules::f_string_missing_placeholders(checker, f_string_expr);
            }
            if checker.is_rule_enabled(Rule::ExplicitFStringTypeConversion) {
                for f_string in value.f_strings() {
                    ruff::rules::explicit_f_string_type_conversion(checker, f_string);
                }
            }
            if checker.is_rule_enabled(Rule::HardcodedSQLExpression) {
                flake8_bandit::rules::hardcoded_sql_expression(checker, expr);
            }
            if checker.is_rule_enabled(Rule::UnicodeKindPrefix) {
                for string_literal in value.literals() {
                    pyupgrade::rules::unicode_kind_prefix(checker, string_literal);
                }
            }
            if checker.is_rule_enabled(Rule::MissingFStringSyntax) {
                for string_literal in value.literals() {
                    ruff::rules::missing_fstring_syntax(checker, string_literal);
                }
            }
        }
        Expr::BinOp(ast::ExprBinOp {
            left,
            op: Operator::RShift,
            ..
        }) => {
            if checker.is_rule_enabled(Rule::InvalidPrintSyntax) {
                pyflakes::rules::invalid_print_syntax(checker, left);
            }
        }
        Expr::BinOp(
            bin_op @ ast::ExprBinOp {
                left,
                op: Operator::Mod,
                right,
                range: _,
                node_index: _,
            },
        ) => {
            if let Expr::StringLiteral(format_string @ ast::ExprStringLiteral { value, .. }) =
                left.as_ref()
            {
                if checker.any_rule_enabled(&[
                    Rule::PercentFormatInvalidFormat,
                    Rule::PercentFormatExpectedMapping,
                    Rule::PercentFormatExpectedSequence,
                    Rule::PercentFormatExtraNamedArguments,
                    Rule::PercentFormatMissingArgument,
                    Rule::PercentFormatMixedPositionalAndNamed,
                    Rule::PercentFormatPositionalCountMismatch,
                    Rule::PercentFormatStarRequiresSequence,
                    Rule::PercentFormatUnsupportedFormatCharacter,
                ]) {
                    let location = expr.range();
                    match pyflakes::cformat::CFormatSummary::try_from(value.to_str()) {
                        Err(CFormatError {
                            typ: CFormatErrorType::UnsupportedFormatChar(c),
                            ..
                        }) => {
                            // F509
                            checker.report_diagnostic_if_enabled(
                                pyflakes::rules::PercentFormatUnsupportedFormatCharacter {
                                    char: c,
                                },
                                location,
                            );
                        }
                        Err(e) => {
                            // F501
                            checker.report_diagnostic_if_enabled(
                                pyflakes::rules::PercentFormatInvalidFormat {
                                    message: e.to_string(),
                                },
                                location,
                            );
                        }
                        Ok(summary) => {
                            if checker.is_rule_enabled(Rule::PercentFormatExpectedMapping) {
                                pyflakes::rules::percent_format_expected_mapping(
                                    checker, &summary, right, location,
                                );
                            }
                            if checker.is_rule_enabled(Rule::PercentFormatExpectedSequence) {
                                pyflakes::rules::percent_format_expected_sequence(
                                    checker, &summary, right, location,
                                );
                            }
                            if checker.is_rule_enabled(Rule::PercentFormatExtraNamedArguments) {
                                pyflakes::rules::percent_format_extra_named_arguments(
                                    checker, &summary, right, location,
                                );
                            }
                            if checker.is_rule_enabled(Rule::PercentFormatMissingArgument) {
                                pyflakes::rules::percent_format_missing_arguments(
                                    checker, &summary, right, location,
                                );
                            }
                            if checker.is_rule_enabled(Rule::PercentFormatMixedPositionalAndNamed) {
                                pyflakes::rules::percent_format_mixed_positional_and_named(
                                    checker, &summary, location,
                                );
                            }
                            if checker.is_rule_enabled(Rule::PercentFormatPositionalCountMismatch) {
                                pyflakes::rules::percent_format_positional_count_mismatch(
                                    checker, &summary, right, location,
                                );
                            }
                            if checker.is_rule_enabled(Rule::PercentFormatStarRequiresSequence) {
                                pyflakes::rules::percent_format_star_requires_sequence(
                                    checker, &summary, right, location,
                                );
                            }
                        }
                    }
                }
                if checker.is_rule_enabled(Rule::PrintfStringFormatting) {
                    pyupgrade::rules::printf_string_formatting(checker, bin_op, format_string);
                }
                if checker.is_rule_enabled(Rule::BadStringFormatCharacter) {
                    pylint::rules::bad_string_format_character::percent(
                        checker,
                        expr,
                        format_string,
                    );
                }
                if checker.is_rule_enabled(Rule::BadStringFormatType) {
                    pylint::rules::bad_string_format_type(checker, bin_op, format_string);
                }
                if checker.is_rule_enabled(Rule::HardcodedSQLExpression) {
                    flake8_bandit::rules::hardcoded_sql_expression(checker, expr);
                }
            }
        }
        Expr::BinOp(ast::ExprBinOp {
            op: Operator::Add, ..
        }) => {
            if checker.is_rule_enabled(Rule::ExplicitStringConcatenation) {
                flake8_implicit_str_concat::rules::explicit(checker, expr);
            }
            if checker.is_rule_enabled(Rule::CollectionLiteralConcatenation) {
                ruff::rules::collection_literal_concatenation(checker, expr);
            }
            if checker.is_rule_enabled(Rule::HardcodedSQLExpression) {
                flake8_bandit::rules::hardcoded_sql_expression(checker, expr);
            }
        }
        Expr::BinOp(ast::ExprBinOp {
            op: Operator::BitOr,
            ..
        }) => {
            // Ex) `str | None`
            if checker.is_rule_enabled(Rule::FutureRequiredTypeAnnotation) {
                if !checker.semantic.future_annotations_or_stub()
                    && checker.target_version() < PythonVersion::PY310
                    && checker.semantic.in_annotation()
                    && checker.semantic.in_runtime_evaluated_annotation()
                    && !checker.semantic.in_string_type_definition()
                {
                    flake8_future_annotations::rules::future_required_type_annotation(
                        checker,
                        expr,
                        flake8_future_annotations::rules::Reason::PEP604,
                    );
                }
            }

            if checker.is_rule_enabled(Rule::NeverUnion) {
                ruff::rules::never_union(checker, expr);
            }

            // Avoid duplicate checks if the parent is a union, since these rules already
            // traverse nested unions.
            if !checker.semantic.in_nested_union() {
                if checker.is_rule_enabled(Rule::DuplicateUnionMember)
                    && checker.semantic.in_type_definition()
                    // Avoid duplicate checks inside `Optional`
                    && !(
                        is_optional_as_none_in_union_enabled(checker.settings())
                        && checker.semantic.inside_optional()
                    )
                {
                    flake8_pyi::rules::duplicate_union_member(checker, expr);
                }
                if checker.is_rule_enabled(Rule::UnnecessaryLiteralUnion) {
                    flake8_pyi::rules::unnecessary_literal_union(checker, expr);
                }
                if checker.is_rule_enabled(Rule::RedundantLiteralUnion) {
                    flake8_pyi::rules::redundant_literal_union(checker, expr);
                }
                if checker.is_rule_enabled(Rule::UnnecessaryTypeUnion) {
                    flake8_pyi::rules::unnecessary_type_union(checker, expr);
                }
                if checker.is_rule_enabled(Rule::RuntimeStringUnion) {
                    flake8_type_checking::rules::runtime_string_union(checker, expr);
                }
                if checker.is_rule_enabled(Rule::NoneNotAtEndOfUnion) {
                    ruff::rules::none_not_at_end_of_union(checker, expr);
                }
            }
        }
        Expr::UnaryOp(
            unary_op @ ast::ExprUnaryOp {
                op,
                operand,
                range: _,
                node_index: _,
            },
        ) => {
            if checker.any_rule_enabled(&[Rule::NotInTest, Rule::NotIsTest]) {
                pycodestyle::rules::not_tests(checker, unary_op);
            }
            if checker.is_rule_enabled(Rule::UnaryPrefixIncrementDecrement) {
                flake8_bugbear::rules::unary_prefix_increment_decrement(
                    checker, expr, *op, operand,
                );
            }
            if checker.is_rule_enabled(Rule::NegateEqualOp) {
                flake8_simplify::rules::negation_with_equal_op(checker, expr, *op, operand);
            }
            if checker.is_rule_enabled(Rule::NegateNotEqualOp) {
                flake8_simplify::rules::negation_with_not_equal_op(checker, expr, *op, operand);
            }
            if checker.is_rule_enabled(Rule::DoubleNegation) {
                flake8_simplify::rules::double_negation(checker, expr, *op, operand);
            }
        }
        Expr::Compare(
            compare @ ast::ExprCompare {
                left,
                ops,
                comparators,
                range: _,
                node_index: _,
            },
        ) => {
            if checker.any_rule_enabled(&[Rule::NoneComparison, Rule::TrueFalseComparison]) {
                pycodestyle::rules::literal_comparisons(checker, compare);
            }
            if checker.is_rule_enabled(Rule::IsLiteral) {
                pyflakes::rules::invalid_literal_comparison(checker, left, ops, comparators, expr);
            }
            if checker.is_rule_enabled(Rule::TypeComparison) {
                pycodestyle::rules::type_comparison(checker, compare);
            }
            if checker.any_rule_enabled(&[
                Rule::SysVersionCmpStr3,
                Rule::SysVersionInfo0Eq3,
                Rule::SysVersionInfo1CmpInt,
                Rule::SysVersionInfoMinorCmpInt,
                Rule::SysVersionCmpStr10,
            ]) {
                flake8_2020::rules::compare(checker, left, ops, comparators);
            }
            if checker.is_rule_enabled(Rule::HardcodedPasswordString) {
                flake8_bandit::rules::compare_to_hardcoded_password_string(
                    checker,
                    left,
                    comparators,
                );
            }
            if checker.is_rule_enabled(Rule::ComparisonWithItself) {
                pylint::rules::comparison_with_itself(checker, left, ops, comparators);
            }
            if checker.is_rule_enabled(Rule::LiteralMembership) {
                pylint::rules::literal_membership(checker, compare);
            }
            if checker.is_rule_enabled(Rule::ComparisonOfConstant) {
                pylint::rules::comparison_of_constant(checker, left, ops, comparators);
            }
            if checker.is_rule_enabled(Rule::CompareToEmptyString) {
                pylint::rules::compare_to_empty_string(checker, left, ops, comparators);
            }
            if checker.is_rule_enabled(Rule::MagicValueComparison) {
                pylint::rules::magic_value_comparison(checker, left, comparators);
            }
            if checker.is_rule_enabled(Rule::NanComparison) {
                pylint::rules::nan_comparison(checker, left, comparators);
            }
            if checker.is_rule_enabled(Rule::InEmptyCollection) {
                ruff::rules::in_empty_collection(checker, compare);
            }
            if checker.is_rule_enabled(Rule::InDictKeys) {
                flake8_simplify::rules::key_in_dict_compare(checker, compare);
            }
            if checker.is_rule_enabled(Rule::YodaConditions) {
                flake8_simplify::rules::yoda_conditions(checker, expr, left, ops, comparators);
            }
            if checker.is_rule_enabled(Rule::PandasNuniqueConstantSeriesCheck) {
                pandas_vet::rules::nunique_constant_series_check(
                    checker,
                    expr,
                    left,
                    ops,
                    comparators,
                );
            }
            if checker.is_rule_enabled(Rule::TypeNoneComparison) {
                refurb::rules::type_none_comparison(checker, compare);
            }
            if checker.is_rule_enabled(Rule::SingleItemMembershipTest) {
                refurb::rules::single_item_membership_test(checker, expr, left, ops, comparators);
            }
        }
        Expr::NumberLiteral(number_literal @ ast::ExprNumberLiteral { .. }) => {
            if checker.source_type.is_stub() && checker.is_rule_enabled(Rule::NumericLiteralTooLong)
            {
                flake8_pyi::rules::numeric_literal_too_long(checker, expr);
            }
            if checker.is_rule_enabled(Rule::MathConstant) {
                refurb::rules::math_constant(checker, number_literal);
            }
        }
        Expr::StringLiteral(
            string_like @ ast::ExprStringLiteral {
                value,
                range: _,
                node_index: _,
            },
        ) => {
            if checker.is_rule_enabled(Rule::UnicodeKindPrefix) {
                for string_part in value {
                    pyupgrade::rules::unicode_kind_prefix(checker, string_part);
                }
            }
            if checker.is_rule_enabled(Rule::MissingFStringSyntax) {
                for string_literal in value {
                    ruff::rules::missing_fstring_syntax(checker, string_literal);
                }
            }
            if checker.is_rule_enabled(Rule::HardcodedStringCharset) {
                refurb::rules::hardcoded_string_charset_literal(checker, string_like);
            }
        }
        Expr::If(
            if_exp @ ast::ExprIf {
                test,
                body,
                orelse,
                range: _,
                node_index: _,
            },
        ) => {
            if checker.is_rule_enabled(Rule::IfElseBlockInsteadOfDictGet) {
                flake8_simplify::rules::if_exp_instead_of_dict_get(
                    checker, expr, test, body, orelse,
                );
            }
            if checker.is_rule_enabled(Rule::IfExprWithTrueFalse) {
                flake8_simplify::rules::if_expr_with_true_false(checker, expr, test, body, orelse);
            }
            if checker.is_rule_enabled(Rule::IfExprWithFalseTrue) {
                flake8_simplify::rules::if_expr_with_false_true(checker, expr, test, body, orelse);
            }
            if checker.is_rule_enabled(Rule::IfExprWithTwistedArms) {
                flake8_simplify::rules::twisted_arms_in_ifexpr(checker, expr, test, body, orelse);
            }
            if checker.is_rule_enabled(Rule::IfExprMinMax) {
                refurb::rules::if_expr_min_max(checker, if_exp);
            }
            if checker.is_rule_enabled(Rule::IfExpInsteadOfOrOperator) {
                refurb::rules::if_exp_instead_of_or_operator(checker, if_exp);
            }
            if checker.is_rule_enabled(Rule::UselessIfElse) {
                ruff::rules::useless_if_else(checker, if_exp);
            }
            if checker.is_rule_enabled(Rule::SliceToRemovePrefixOrSuffix) {
                refurb::rules::slice_to_remove_affix_expr(checker, if_exp);
            }
        }
        Expr::ListComp(
            comp @ ast::ExprListComp {
                elt,
                generators,
                range: _,
                node_index: _,
            },
        ) => {
            if checker.is_rule_enabled(Rule::UnnecessaryListIndexLookup) {
                pylint::rules::unnecessary_list_index_lookup_comprehension(checker, expr);
            }
            if checker.is_rule_enabled(Rule::UnnecessaryDictIndexLookup) {
                pylint::rules::unnecessary_dict_index_lookup_comprehension(checker, expr);
            }
            if checker.is_rule_enabled(Rule::UnnecessaryComprehension) {
                flake8_comprehensions::rules::unnecessary_list_set_comprehension(
                    checker, expr, elt, generators,
                );
            }
            if checker.is_rule_enabled(Rule::FunctionUsesLoopVariable) {
                flake8_bugbear::rules::function_uses_loop_variable(checker, &Node::Expr(expr));
            }
            if checker.is_rule_enabled(Rule::IterationOverSet) {
                for generator in generators {
                    pylint::rules::iteration_over_set(checker, &generator.iter);
                }
            }
            if checker.is_rule_enabled(Rule::ReimplementedStarmap) {
                refurb::rules::reimplemented_starmap(checker, &comp.into());
            }
        }
        Expr::SetComp(
            comp @ ast::ExprSetComp {
                elt,
                generators,
                range: _,
                node_index: _,
            },
        ) => {
            if checker.is_rule_enabled(Rule::UnnecessaryListIndexLookup) {
                pylint::rules::unnecessary_list_index_lookup_comprehension(checker, expr);
            }
            if checker.is_rule_enabled(Rule::UnnecessaryDictIndexLookup) {
                pylint::rules::unnecessary_dict_index_lookup_comprehension(checker, expr);
            }
            if checker.is_rule_enabled(Rule::UnnecessaryComprehension) {
                flake8_comprehensions::rules::unnecessary_list_set_comprehension(
                    checker, expr, elt, generators,
                );
            }
            if checker.is_rule_enabled(Rule::FunctionUsesLoopVariable) {
                flake8_bugbear::rules::function_uses_loop_variable(checker, &Node::Expr(expr));
            }
            if checker.is_rule_enabled(Rule::IterationOverSet) {
                for generator in generators {
                    pylint::rules::iteration_over_set(checker, &generator.iter);
                }
            }
            if checker.is_rule_enabled(Rule::ReimplementedStarmap) {
                refurb::rules::reimplemented_starmap(checker, &comp.into());
            }
        }
        Expr::DictComp(
            dict_comp @ ast::ExprDictComp {
                key,
                value,
                generators,
                range: _,
                node_index: _,
            },
        ) => {
            if checker.is_rule_enabled(Rule::UnnecessaryListIndexLookup) {
                pylint::rules::unnecessary_list_index_lookup_comprehension(checker, expr);
            }
            if checker.is_rule_enabled(Rule::UnnecessaryDictIndexLookup) {
                pylint::rules::unnecessary_dict_index_lookup_comprehension(checker, expr);
            }

            if checker.is_rule_enabled(Rule::UnnecessaryComprehension) {
                flake8_comprehensions::rules::unnecessary_dict_comprehension(
                    checker, expr, key, value, generators,
                );
            }

            if checker.is_rule_enabled(Rule::UnnecessaryDictComprehensionForIterable) {
                flake8_comprehensions::rules::unnecessary_dict_comprehension_for_iterable(
                    checker, dict_comp,
                );
            }

            if checker.is_rule_enabled(Rule::FunctionUsesLoopVariable) {
                flake8_bugbear::rules::function_uses_loop_variable(checker, &Node::Expr(expr));
            }
            if checker.is_rule_enabled(Rule::IterationOverSet) {
                for generator in generators {
                    pylint::rules::iteration_over_set(checker, &generator.iter);
                }
            }
            if checker.is_rule_enabled(Rule::StaticKeyDictComprehension) {
                flake8_bugbear::rules::static_key_dict_comprehension(checker, dict_comp);
            }
        }
        Expr::Generator(
            generator @ ast::ExprGenerator {
                generators,
                elt: _,
                range: _,
                node_index: _,
                parenthesized: _,
            },
        ) => {
            if checker.is_rule_enabled(Rule::UnnecessaryListIndexLookup) {
                pylint::rules::unnecessary_list_index_lookup_comprehension(checker, expr);
            }
            if checker.is_rule_enabled(Rule::UnnecessaryDictIndexLookup) {
                pylint::rules::unnecessary_dict_index_lookup_comprehension(checker, expr);
            }
            if checker.is_rule_enabled(Rule::FunctionUsesLoopVariable) {
                flake8_bugbear::rules::function_uses_loop_variable(checker, &Node::Expr(expr));
            }
            if checker.is_rule_enabled(Rule::IterationOverSet) {
                for generator in generators {
                    pylint::rules::iteration_over_set(checker, &generator.iter);
                }
            }
            if checker.is_rule_enabled(Rule::ReimplementedStarmap) {
                refurb::rules::reimplemented_starmap(checker, &generator.into());
            }
        }
        Expr::BoolOp(bool_op) => {
            if checker.is_rule_enabled(Rule::BooleanChainedComparison) {
                pylint::rules::boolean_chained_comparison(checker, bool_op);
            }
            if checker.is_rule_enabled(Rule::MultipleStartsEndsWith) {
                flake8_pie::rules::multiple_starts_ends_with(checker, expr);
            }
            if checker.is_rule_enabled(Rule::DuplicateIsinstanceCall) {
                flake8_simplify::rules::duplicate_isinstance_call(checker, expr);
            }
            if checker.is_rule_enabled(Rule::CompareWithTuple) {
                flake8_simplify::rules::compare_with_tuple(checker, expr);
            }
            if checker.is_rule_enabled(Rule::ExprAndNotExpr) {
                flake8_simplify::rules::expr_and_not_expr(checker, expr);
            }
            if checker.is_rule_enabled(Rule::ExprOrNotExpr) {
                flake8_simplify::rules::expr_or_not_expr(checker, expr);
            }
            if checker.is_rule_enabled(Rule::ExprOrTrue) {
                flake8_simplify::rules::expr_or_true(checker, expr);
            }
            if checker.is_rule_enabled(Rule::ExprAndFalse) {
                flake8_simplify::rules::expr_and_false(checker, expr);
            }
            if checker.is_rule_enabled(Rule::RepeatedEqualityComparison) {
                pylint::rules::repeated_equality_comparison(checker, bool_op);
            }
            if checker.is_rule_enabled(Rule::UnnecessaryKeyCheck) {
                ruff::rules::unnecessary_key_check(checker, expr);
            }
            if checker.is_rule_enabled(Rule::ParenthesizeChainedOperators) {
                ruff::rules::parenthesize_chained_logical_operators(checker, bool_op);
            }
        }
        Expr::Lambda(lambda) => {
            if checker.is_rule_enabled(Rule::ReimplementedOperator) {
                refurb::rules::reimplemented_operator(checker, &lambda.into());
            }
            if checker.is_rule_enabled(Rule::InvalidArgumentName) {
                pep8_naming::rules::invalid_argument_name_lambda(checker, lambda);
            }
        }
        _ => {}
    }
}
