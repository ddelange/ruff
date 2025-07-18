//! Effective program settings, taking into account pyproject.toml and
//! command-line options. Structure is optimized for internal usage, as opposed
//! to external visibility or parsing.

use regex::Regex;
use rustc_hash::FxHashSet;
use std::fmt::{Display, Formatter};
use std::path::{Path, PathBuf};
use std::sync::LazyLock;
use types::CompiledPerFileTargetVersionList;

use crate::codes::RuleCodePrefix;
use ruff_macros::CacheKey;
use ruff_python_ast::PythonVersion;

use crate::line_width::LineLength;
use crate::registry::{Linter, Rule};
use crate::rules::{
    flake8_annotations, flake8_bandit, flake8_boolean_trap, flake8_bugbear, flake8_builtins,
    flake8_comprehensions, flake8_copyright, flake8_errmsg, flake8_gettext,
    flake8_implicit_str_concat, flake8_import_conventions, flake8_pytest_style, flake8_quotes,
    flake8_self, flake8_tidy_imports, flake8_type_checking, flake8_unused_arguments, isort, mccabe,
    pep8_naming, pycodestyle, pydoclint, pydocstyle, pyflakes, pylint, pyupgrade, ruff,
};
use crate::settings::types::{CompiledPerFileIgnoreList, ExtensionMapping, FilePatternSet};
use crate::{RuleSelector, codes, fs};

use super::line_width::IndentWidth;

use self::fix_safety_table::FixSafetyTable;
use self::rule_table::RuleTable;
use self::types::PreviewMode;
use crate::rule_selector::PreviewOptions;

pub mod fix_safety_table;
pub mod flags;
pub mod rule_table;
pub mod types;

/// `display_settings!` is a macro that can display and format struct fields in a readable,
/// namespaced format. It's particularly useful at generating `Display` implementations
/// for types used in settings.
///
/// # Example
/// ```
/// use std::fmt;
/// use ruff_linter::display_settings;
/// #[derive(Default)]
/// struct Settings {
///     option_a: bool,
///     sub_settings: SubSettings,
///     option_b: String,
/// }
///
/// struct SubSettings {
///     name: String
/// }
///
/// impl Default for SubSettings {
///     fn default() -> Self {
///         Self { name: "Default Name".into() }
///     }
///
/// }
///
/// impl fmt::Display for SubSettings {
///     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
///         display_settings! {
///             formatter = f,
///             namespace = "sub_settings",
///             fields = [
///                 self.name | quoted
///             ]
///         }
///         Ok(())
///     }
///
/// }
///
/// impl fmt::Display for Settings {
///     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
///         display_settings! {
///             formatter = f,
///             fields = [
///                 self.option_a,
///                 self.sub_settings | nested,
///                 self.option_b | quoted,
///             ]
///         }
///         Ok(())
///     }
///
/// }
///
/// const EXPECTED_OUTPUT: &str = r#"option_a = false
/// sub_settings.name = "Default Name"
/// option_b = ""
/// "#;
///
/// fn main() {
///     let settings = Settings::default();
///     assert_eq!(format!("{settings}"), EXPECTED_OUTPUT);
/// }
/// ```
#[macro_export]
macro_rules! display_settings {
    (formatter = $fmt:ident, namespace = $namespace:literal, fields = [$($settings:ident.$field:ident $(| $modifier:tt)?),* $(,)?]) => {
        {
            const _PREFIX: &str = concat!($namespace, ".");
            $(
                display_settings!(@field $fmt, _PREFIX, $settings.$field $(| $modifier)?);
            )*
        }
    };
    (formatter = $fmt:ident, fields = [$($settings:ident.$field:ident $(| $modifier:tt)?),* $(,)?]) => {
        {
            const _PREFIX: &str = "";
            $(
                display_settings!(@field $fmt, _PREFIX, $settings.$field $(| $modifier)?);
            )*
        }
    };
    (@field $fmt:ident, $prefix:ident, $settings:ident.$field:ident | debug) => {
        writeln!($fmt, "{}{} = {:?}", $prefix, stringify!($field), $settings.$field)?;
    };
    (@field $fmt:ident, $prefix:ident, $settings:ident.$field:ident | path) => {
        writeln!($fmt, "{}{} = \"{}\"", $prefix, stringify!($field), $settings.$field.display())?;
    };
    (@field $fmt:ident, $prefix:ident, $settings:ident.$field:ident | quoted) => {
        writeln!($fmt, "{}{} = \"{}\"", $prefix, stringify!($field), $settings.$field)?;
    };
    (@field $fmt:ident, $prefix:ident, $settings:ident.$field:ident | globmatcher) => {
        writeln!($fmt, "{}{} = \"{}\"", $prefix, stringify!($field), $settings.$field.glob())?;
    };
    (@field $fmt:ident, $prefix:ident, $settings:ident.$field:ident | nested) => {
        write!($fmt, "{}", $settings.$field)?;
    };
    (@field $fmt:ident, $prefix:ident, $settings:ident.$field:ident | optional) => {
        {
            write!($fmt, "{}{} = ", $prefix, stringify!($field))?;
            match &$settings.$field {
                Some(value) => writeln!($fmt, "{}", value)?,
                None        => writeln!($fmt, "none")?
            };
        }
    };
    (@field $fmt:ident, $prefix:ident, $settings:ident.$field:ident | array) => {
        {
            write!($fmt, "{}{} = ", $prefix, stringify!($field))?;
            if $settings.$field.is_empty() {
                writeln!($fmt, "[]")?;
            } else {
                writeln!($fmt, "[")?;
                for elem in &$settings.$field {
                    writeln!($fmt, "\t{elem},")?;
                }
                writeln!($fmt, "]")?;
            }
        }
    };
    (@field $fmt:ident, $prefix:ident, $settings:ident.$field:ident | map) => {
        {
            use itertools::Itertools;

            write!($fmt, "{}{} = ", $prefix, stringify!($field))?;
            if $settings.$field.is_empty() {
                writeln!($fmt, "{{}}")?;
            } else {
                writeln!($fmt, "{{")?;
                for (key, value) in $settings.$field.iter().sorted_by(|(left, _), (right, _)| left.cmp(right)) {
                    writeln!($fmt, "\t{key} = {value},")?;
                }
                writeln!($fmt, "}}")?;
            }
        }
    };
    (@field $fmt:ident, $prefix:ident, $settings:ident.$field:ident | set) => {
        {
            use itertools::Itertools;

            write!($fmt, "{}{} = ", $prefix, stringify!($field))?;
            if $settings.$field.is_empty() {
                writeln!($fmt, "[]")?;
            } else {
                writeln!($fmt, "[")?;
                for elem in $settings.$field.iter().sorted_by(|left, right| left.cmp(right)) {
                    writeln!($fmt, "\t{elem},")?;
                }
                writeln!($fmt, "]")?;
            }
        }
    };
    (@field $fmt:ident, $prefix:ident, $settings:ident.$field:ident | paths) => {
        {
            write!($fmt, "{}{} = ", $prefix, stringify!($field))?;
            if $settings.$field.is_empty() {
                writeln!($fmt, "[]")?;
            } else {
                writeln!($fmt, "[")?;
                for elem in &$settings.$field {
                    writeln!($fmt, "\t\"{}\",", elem.display())?;
                }
                writeln!($fmt, "]")?;
            }
        }
    };
    (@field $fmt:ident, $prefix:ident, $settings:ident.$field:ident) => {
        writeln!($fmt, "{}{} = {}", $prefix, stringify!($field), $settings.$field)?;
    };
}

#[derive(Debug, Clone, CacheKey)]
#[expect(clippy::struct_excessive_bools)]
pub struct LinterSettings {
    pub exclude: FilePatternSet,
    pub extension: ExtensionMapping,
    pub project_root: PathBuf,

    pub rules: RuleTable,
    pub per_file_ignores: CompiledPerFileIgnoreList,
    pub fix_safety: FixSafetyTable,

    /// The non-path-resolved Python version specified by the `target-version` input option.
    ///
    /// If you have a `Checker` available, see its `target_version` method instead.
    ///
    /// Otherwise, see [`LinterSettings::resolve_target_version`] for a way to obtain the Python
    /// version for a given file, while respecting the overrides in `per_file_target_version`.
    pub unresolved_target_version: TargetVersion,
    /// Path-specific overrides to `unresolved_target_version`.
    ///
    /// If you have a `Checker` available, see its `target_version` method instead.
    ///
    /// Otherwise, see [`LinterSettings::resolve_target_version`] for a way to check a given
    /// [`Path`] against these patterns, while falling back to `unresolved_target_version` if none
    /// of them match.
    pub per_file_target_version: CompiledPerFileTargetVersionList,
    pub preview: PreviewMode,
    pub explicit_preview_rules: bool,

    // Rule-specific settings
    pub allowed_confusables: FxHashSet<char>,
    pub builtins: Vec<String>,
    pub dummy_variable_rgx: Regex,
    pub external: Vec<String>,
    pub ignore_init_module_imports: bool,
    pub logger_objects: Vec<String>,
    pub namespace_packages: Vec<PathBuf>,
    pub src: Vec<PathBuf>,
    pub tab_size: IndentWidth,
    pub line_length: LineLength,
    pub task_tags: Vec<String>,
    pub typing_modules: Vec<String>,
    pub typing_extensions: bool,
    pub future_annotations: bool,

    // Plugins
    pub flake8_annotations: flake8_annotations::settings::Settings,
    pub flake8_bandit: flake8_bandit::settings::Settings,
    pub flake8_boolean_trap: flake8_boolean_trap::settings::Settings,
    pub flake8_bugbear: flake8_bugbear::settings::Settings,
    pub flake8_builtins: flake8_builtins::settings::Settings,
    pub flake8_comprehensions: flake8_comprehensions::settings::Settings,
    pub flake8_copyright: flake8_copyright::settings::Settings,
    pub flake8_errmsg: flake8_errmsg::settings::Settings,
    pub flake8_gettext: flake8_gettext::settings::Settings,
    pub flake8_implicit_str_concat: flake8_implicit_str_concat::settings::Settings,
    pub flake8_import_conventions: flake8_import_conventions::settings::Settings,
    pub flake8_pytest_style: flake8_pytest_style::settings::Settings,
    pub flake8_quotes: flake8_quotes::settings::Settings,
    pub flake8_self: flake8_self::settings::Settings,
    pub flake8_tidy_imports: flake8_tidy_imports::settings::Settings,
    pub flake8_type_checking: flake8_type_checking::settings::Settings,
    pub flake8_unused_arguments: flake8_unused_arguments::settings::Settings,
    pub isort: isort::settings::Settings,
    pub mccabe: mccabe::settings::Settings,
    pub pep8_naming: pep8_naming::settings::Settings,
    pub pycodestyle: pycodestyle::settings::Settings,
    pub pydoclint: pydoclint::settings::Settings,
    pub pydocstyle: pydocstyle::settings::Settings,
    pub pyflakes: pyflakes::settings::Settings,
    pub pylint: pylint::settings::Settings,
    pub pyupgrade: pyupgrade::settings::Settings,
    pub ruff: ruff::settings::Settings,
}

impl Display for LinterSettings {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "\n# Linter Settings")?;
        display_settings! {
            formatter = f,
            namespace = "linter",
            fields = [
                self.exclude,
                self.project_root | path,

                self.rules | nested,
                self.per_file_ignores,
                self.fix_safety | nested,

                self.unresolved_target_version,
                self.per_file_target_version,
                self.preview,
                self.explicit_preview_rules,
                self.extension | debug,

                self.allowed_confusables | array,
                self.builtins | array,
                self.dummy_variable_rgx,
                self.external | array,
                self.ignore_init_module_imports,
                self.logger_objects | array,
                self.namespace_packages | debug,
                self.src | paths,
                self.tab_size,
                self.line_length,
                self.task_tags | array,
                self.typing_modules | array,
                self.typing_extensions,
            ]
        }
        writeln!(f, "\n# Linter Plugins")?;
        display_settings! {
            formatter = f,
            namespace = "linter",
            fields = [
                self.flake8_annotations | nested,
                self.flake8_bandit | nested,
                self.flake8_bugbear | nested,
                self.flake8_builtins | nested,
                self.flake8_comprehensions | nested,
                self.flake8_copyright | nested,
                self.flake8_errmsg | nested,
                self.flake8_gettext | nested,
                self.flake8_implicit_str_concat | nested,
                self.flake8_import_conventions | nested,
                self.flake8_pytest_style | nested,
                self.flake8_quotes | nested,
                self.flake8_self | nested,
                self.flake8_tidy_imports | nested,
                self.flake8_type_checking | nested,
                self.flake8_unused_arguments | nested,
                self.isort | nested,
                self.mccabe | nested,
                self.pep8_naming | nested,
                self.pycodestyle | nested,
                self.pyflakes | nested,
                self.pylint | nested,
                self.pyupgrade | nested,
                self.ruff | nested,
            ]
        }
        Ok(())
    }
}

pub const DEFAULT_SELECTORS: &[RuleSelector] = &[
    RuleSelector::Linter(Linter::Pyflakes),
    // Only include pycodestyle rules that do not overlap with the formatter
    RuleSelector::Prefix {
        prefix: RuleCodePrefix::Pycodestyle(codes::Pycodestyle::E4),
        redirected_from: None,
    },
    RuleSelector::Prefix {
        prefix: RuleCodePrefix::Pycodestyle(codes::Pycodestyle::E7),
        redirected_from: None,
    },
    RuleSelector::Prefix {
        prefix: RuleCodePrefix::Pycodestyle(codes::Pycodestyle::E9),
        redirected_from: None,
    },
];

pub const TASK_TAGS: &[&str] = &["TODO", "FIXME", "XXX"];

pub static DUMMY_VARIABLE_RGX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new("^(_+|(_+[a-zA-Z0-9_]*[a-zA-Z0-9]+?))$").unwrap());

impl LinterSettings {
    pub fn for_rule(rule_code: Rule) -> Self {
        Self {
            rules: RuleTable::from_iter([rule_code]),
            unresolved_target_version: PythonVersion::latest().into(),
            ..Self::default()
        }
    }

    pub fn for_rules(rules: impl IntoIterator<Item = Rule>) -> Self {
        Self {
            rules: RuleTable::from_iter(rules),
            unresolved_target_version: PythonVersion::latest().into(),
            ..Self::default()
        }
    }

    pub fn new(project_root: &Path) -> Self {
        Self {
            exclude: FilePatternSet::default(),
            unresolved_target_version: TargetVersion(None),
            per_file_target_version: CompiledPerFileTargetVersionList::default(),
            project_root: project_root.to_path_buf(),
            rules: DEFAULT_SELECTORS
                .iter()
                .flat_map(|selector| selector.rules(&PreviewOptions::default()))
                .collect(),
            allowed_confusables: FxHashSet::from_iter([]),

            // Needs duplicating
            builtins: vec![],
            dummy_variable_rgx: DUMMY_VARIABLE_RGX.clone(),

            external: vec![],
            ignore_init_module_imports: true,
            logger_objects: vec![],
            namespace_packages: vec![],

            per_file_ignores: CompiledPerFileIgnoreList::default(),
            fix_safety: FixSafetyTable::default(),

            src: vec![fs::get_cwd().to_path_buf(), fs::get_cwd().join("src")],
            // Needs duplicating
            tab_size: IndentWidth::default(),
            line_length: LineLength::default(),

            task_tags: TASK_TAGS.iter().map(ToString::to_string).collect(),
            typing_modules: vec![],
            flake8_annotations: flake8_annotations::settings::Settings::default(),
            flake8_bandit: flake8_bandit::settings::Settings::default(),
            flake8_boolean_trap: flake8_boolean_trap::settings::Settings::default(),
            flake8_bugbear: flake8_bugbear::settings::Settings::default(),
            flake8_builtins: flake8_builtins::settings::Settings::default(),
            flake8_comprehensions: flake8_comprehensions::settings::Settings::default(),
            flake8_copyright: flake8_copyright::settings::Settings::default(),
            flake8_errmsg: flake8_errmsg::settings::Settings::default(),
            flake8_gettext: flake8_gettext::settings::Settings::default(),
            flake8_implicit_str_concat: flake8_implicit_str_concat::settings::Settings::default(),
            flake8_import_conventions: flake8_import_conventions::settings::Settings::default(),
            flake8_pytest_style: flake8_pytest_style::settings::Settings::default(),
            flake8_quotes: flake8_quotes::settings::Settings::default(),
            flake8_self: flake8_self::settings::Settings::default(),
            flake8_tidy_imports: flake8_tidy_imports::settings::Settings::default(),
            flake8_type_checking: flake8_type_checking::settings::Settings::default(),
            flake8_unused_arguments: flake8_unused_arguments::settings::Settings::default(),
            isort: isort::settings::Settings::default(),
            mccabe: mccabe::settings::Settings::default(),
            pep8_naming: pep8_naming::settings::Settings::default(),
            pycodestyle: pycodestyle::settings::Settings::default(),
            pydoclint: pydoclint::settings::Settings::default(),
            pydocstyle: pydocstyle::settings::Settings::default(),
            pyflakes: pyflakes::settings::Settings::default(),
            pylint: pylint::settings::Settings::default(),
            pyupgrade: pyupgrade::settings::Settings::default(),
            ruff: ruff::settings::Settings::default(),
            preview: PreviewMode::default(),
            explicit_preview_rules: false,
            extension: ExtensionMapping::default(),
            typing_extensions: true,
            future_annotations: false,
        }
    }

    #[must_use]
    pub fn with_target_version(mut self, target_version: PythonVersion) -> Self {
        self.unresolved_target_version = target_version.into();
        self
    }

    /// Resolve the [`TargetVersion`] to use for linting.
    ///
    /// This method respects the per-file version overrides in
    /// [`LinterSettings::per_file_target_version`] and falls back on
    /// [`LinterSettings::unresolved_target_version`] if none of the override patterns match.
    pub fn resolve_target_version(&self, path: &Path) -> TargetVersion {
        self.per_file_target_version
            .is_match(path)
            .map_or(self.unresolved_target_version, TargetVersion::from)
    }

    pub fn future_annotations(&self) -> bool {
        // TODO(brent) we can just access the field directly once this is stabilized.
        self.future_annotations && crate::preview::is_add_future_annotations_imports_enabled(self)
    }
}

impl Default for LinterSettings {
    fn default() -> Self {
        Self::new(fs::get_cwd())
    }
}

/// A thin wrapper around `Option<PythonVersion>` to clarify the reason for different `unwrap`
/// calls in various places.
///
/// For example, we want to default to `PythonVersion::latest()` for parsing and detecting semantic
/// syntax errors because this will minimize version-related diagnostics when the Python version is
/// unset. In contrast, we want to default to `PythonVersion::default()` for lint rules. These
/// correspond to the [`TargetVersion::parser_version`] and [`TargetVersion::linter_version`]
/// methods, respectively.
#[derive(Debug, Clone, Copy, CacheKey)]
pub struct TargetVersion(pub Option<PythonVersion>);

impl TargetVersion {
    /// Return the [`PythonVersion`] to use for parsing.
    ///
    /// This will be either the Python version specified by the user or the latest supported
    /// version if unset.
    pub fn parser_version(&self) -> PythonVersion {
        self.0.unwrap_or_else(PythonVersion::latest)
    }

    /// Return the [`PythonVersion`] to use for version-dependent lint rules.
    ///
    /// This will either be the Python version specified by the user or the default Python version
    /// if unset.
    pub fn linter_version(&self) -> PythonVersion {
        self.0.unwrap_or_default()
    }
}

impl From<PythonVersion> for TargetVersion {
    fn from(value: PythonVersion) -> Self {
        Self(Some(value))
    }
}

impl Display for TargetVersion {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        // manual inlining of display_settings!
        match self.0 {
            Some(value) => write!(f, "{value}"),
            None => f.write_str("none"),
        }
    }
}
