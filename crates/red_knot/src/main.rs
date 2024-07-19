use std::sync::Mutex;

use clap::Parser;
use crossbeam::channel as crossbeam_channel;
use rustc_hash::FxHashSet;
use salsa::ParallelDatabase;
use tracing::subscriber::Interest;
use tracing::{info, Level, Metadata};
use tracing_subscriber::filter::LevelFilter;
use tracing_subscriber::layer::{Context, Filter, SubscriberExt};
use tracing_subscriber::{Layer, Registry};
use tracing_tree::time::Uptime;

use red_knot::db::RootDatabase;
use red_knot::watch;
use red_knot::watch::Watcher;
use red_knot::workspace::WorkspaceMetadata;
use ruff_db::program::{ProgramSettings, SearchPathSettings};
use ruff_db::system::{OsSystem, System, SystemPathBuf};

use cli::target_version::TargetVersion;
use cli::verbosity::{Verbosity, VerbosityLevel};

mod cli;

#[derive(Debug, Parser)]
#[command(
    author,
    name = "red-knot",
    about = "An experimental multifile analysis backend for Ruff"
)]
#[command(version)]
struct Args {
    #[arg(
        long,
        help = "Changes the current working directory.",
        long_help = "Changes the current working directory before any specified operations. This affects the workspace and configuration discovery.",
        value_name = "PATH"
    )]
    current_directory: Option<SystemPathBuf>,

    #[arg(
        long,
        value_name = "DIRECTORY",
        help = "Custom directory to use for stdlib typeshed stubs"
    )]
    custom_typeshed_dir: Option<SystemPathBuf>,

    #[arg(
        long,
        value_name = "PATH",
        help = "Additional path to use as a module-resolution source (can be passed multiple times)"
    )]
    extra_search_path: Vec<SystemPathBuf>,

    #[arg(long, help = "Python version to assume when resolving types", default_value_t = TargetVersion::default(), value_name="VERSION")]
    target_version: TargetVersion,

    #[clap(flatten)]
    verbosity: Verbosity,

    #[arg(
        long,
        help = "Run in watch mode by re-running whenever files change",
        short = 'W'
    )]
    watch: bool,
}

#[allow(
    clippy::print_stdout,
    clippy::unnecessary_wraps,
    clippy::print_stderr,
    clippy::dbg_macro
)]
pub fn main() -> anyhow::Result<()> {
    let Args {
        current_directory,
        custom_typeshed_dir,
        extra_search_path: extra_paths,
        target_version,
        verbosity,
        watch,
    } = Args::parse_from(std::env::args().collect::<Vec<_>>());

    let verbosity = verbosity.level();
    countme::enable(verbosity == Some(VerbosityLevel::Trace));
    setup_tracing(verbosity);

    let cwd = if let Some(cwd) = current_directory {
        let canonicalized = cwd.as_utf8_path().canonicalize_utf8().unwrap();
        SystemPathBuf::from_utf8_path_buf(canonicalized)
    } else {
        let cwd = std::env::current_dir().unwrap();
        SystemPathBuf::from_path_buf(cwd).unwrap()
    };

    let system = OsSystem::new(cwd.clone());
    let workspace_metadata =
        WorkspaceMetadata::from_path(system.current_directory(), &system).unwrap();

    // TODO: Respect the settings from the workspace metadata. when resolving the program settings.
    let program_settings = ProgramSettings {
        target_version: target_version.into(),
        search_paths: SearchPathSettings {
            extra_paths,
            workspace_root: workspace_metadata.root().to_path_buf(),
            custom_typeshed: custom_typeshed_dir,
            site_packages: None,
        },
    };

    // TODO: Use the `program_settings` to compute the key for the database's persistent
    //   cache and load the cache if it exists.
    let mut db = RootDatabase::new(workspace_metadata, program_settings, system);

    let (main_loop, main_loop_cancellation_token) = MainLoop::new(verbosity);

    // Listen to Ctrl+C and abort the watch mode.
    let main_loop_cancellation_token = Mutex::new(Some(main_loop_cancellation_token));
    ctrlc::set_handler(move || {
        let mut lock = main_loop_cancellation_token.lock().unwrap();

        if let Some(token) = lock.take() {
            token.stop();
        }
    })?;

    if watch {
        main_loop.watch(&mut db)?;
    } else {
        main_loop.run(&mut db);
    }

    Ok(())
}

struct MainLoop {
    /// Sender that can be used to send messages to the main loop.
    sender: crossbeam_channel::Sender<MainLoopMessage>,

    /// Receiver for the messages sent **to** the main loop.
    receiver: crossbeam_channel::Receiver<MainLoopMessage>,

    /// The file system watcher, if running in watch mode.
    watcher: Option<Watcher>,

    /// The paths that are being watched.
    watch_paths: FxHashSet<SystemPathBuf>,

    verbosity: Option<VerbosityLevel>,
}

impl MainLoop {
    fn new(verbosity: Option<VerbosityLevel>) -> (Self, MainLoopCancellationToken) {
        let (sender, receiver) = crossbeam_channel::bounded(10);

        (
            Self {
                sender: sender.clone(),
                receiver,
                watcher: None,
                watch_paths: FxHashSet::default(),
                verbosity,
            },
            MainLoopCancellationToken { sender },
        )
    }

    fn watch(mut self, db: &mut RootDatabase) -> anyhow::Result<()> {
        let sender = self.sender.clone();
        let watcher = watch::directory_watcher(move |event| {
            sender.send(MainLoopMessage::ApplyChanges(event)).unwrap();
        })?;

        self.watcher = Some(watcher);
        self.update_watched_folders(db);

        self.run(db);

        Ok(())
    }

    #[allow(clippy::print_stderr)]
    fn run(mut self, db: &mut RootDatabase) {
        // Schedule the first check.
        self.sender.send(MainLoopMessage::CheckWorkspace).unwrap();
        let mut revision = 0usize;

        while let Ok(message) = self.receiver.recv() {
            tracing::trace!("Main Loop: Tick");

            match message {
                MainLoopMessage::CheckWorkspace => {
                    let db = db.snapshot();
                    let sender = self.sender.clone();

                    // Spawn a new task that checks the workspace. This needs to be done in a separate thread
                    // to prevent blocking the main loop here.
                    rayon::spawn(move || {
                        if let Ok(result) = db.check() {
                            // Send the result back to the main loop for printing.
                            sender
                                .send(MainLoopMessage::CheckCompleted { result, revision })
                                .ok();
                        }
                    });
                }

                MainLoopMessage::CheckCompleted {
                    result,
                    revision: check_revision,
                } => {
                    if check_revision == revision {
                        eprintln!("{}", result.join("\n"));

                        if self.verbosity == Some(VerbosityLevel::Trace) {
                            eprintln!("{}", countme::get_all());
                        }
                    }

                    if self.watcher.is_none() {
                        return self.exit();
                    }
                }

                MainLoopMessage::ApplyChanges(changes) => {
                    revision += 1;
                    // Automatically cancels any pending queries and waits for them to complete.
                    db.apply_changes(changes);
                    self.update_watched_folders(db);
                    self.sender.send(MainLoopMessage::CheckWorkspace).unwrap();
                }
                MainLoopMessage::Exit => {
                    return self.exit();
                }
            }
        }
    }

    fn update_watched_folders(&mut self, db: &RootDatabase) {
        let Some(watcher) = &mut self.watcher else {
            return;
        };

        let new_watch_paths = db.workspace().watch_paths(db);

        let mut added_folders = new_watch_paths.difference(&self.watch_paths).peekable();
        let mut removed_folders = self.watch_paths.difference(&new_watch_paths).peekable();

        if added_folders.peek().is_none() && removed_folders.peek().is_none() {
            return;
        }

        for added_folder in added_folders {
            // TODO How to surface errors here? Just use logging?
            // There's really not much we can do about it and aborting seems kind of bad as well.
            // TOOD: Consider storing a cache_key over all search paths that is used to determine if they've changed.
            //  OR store both the supposed watch paths and the registered watch paths.
            if let Err(error) = watcher.watch(&added_folder) {
                // We'll pretend that setting up the new folders was successful to avoid that we'll try to
                // set up the same folder over-and-over again which is likely to fail again.
                tracing::error!("Failed to watch path {:?}: {:?}", added_folder, error);
            }
        }

        for removed_path in removed_folders {
            if let Err(error) = watcher.unwatch(removed_path) {
                // This doesn't feel worth raising and could be a concequence of having failed to
                // register the path before.
                tracing::info!("Failed to unwatch path {:?}: {:?}", removed_path, error);
            }
            watcher.unwatch(removed_path).unwrap();
        }

        info!("Watching paths {:?}", new_watch_paths);

        self.watch_paths = new_watch_paths;
    }

    #[allow(clippy::print_stderr, clippy::unused_self)]
    fn exit(self) {
        if self.verbosity == Some(VerbosityLevel::Trace) {
            eprintln!("Exit");
            eprintln!("{}", countme::get_all());
        }
    }
}

#[derive(Debug)]
struct MainLoopCancellationToken {
    sender: crossbeam_channel::Sender<MainLoopMessage>,
}

impl MainLoopCancellationToken {
    fn stop(self) {
        self.sender.send(MainLoopMessage::Exit).unwrap();
    }
}

/// Message sent from the orchestrator to the main loop.
#[derive(Debug)]
enum MainLoopMessage {
    CheckWorkspace,
    CheckCompleted {
        result: Vec<String>,
        revision: usize,
    },
    ApplyChanges(Vec<watch::ChangeEvent>),
    Exit,
}

fn setup_tracing(verbosity: Option<VerbosityLevel>) {
    let trace_level = match verbosity {
        None => Level::WARN,
        Some(VerbosityLevel::Info) => Level::INFO,
        Some(VerbosityLevel::Debug) => Level::DEBUG,
        Some(VerbosityLevel::Trace) => Level::TRACE,
    };

    let subscriber = Registry::default().with(
        tracing_tree::HierarchicalLayer::default()
            .with_indent_lines(true)
            .with_indent_amount(2)
            .with_bracketed_fields(true)
            .with_thread_ids(true)
            .with_targets(true)
            .with_writer(|| Box::new(std::io::stderr()))
            .with_timer(Uptime::default())
            .with_filter(LoggingFilter { trace_level }),
    );

    tracing::subscriber::set_global_default(subscriber).unwrap();
}

struct LoggingFilter {
    trace_level: Level,
}

impl LoggingFilter {
    fn is_enabled(&self, meta: &Metadata<'_>) -> bool {
        let filter = if meta.target().starts_with("red_knot") || meta.target().starts_with("ruff") {
            self.trace_level
        } else {
            Level::INFO
        };

        meta.level() <= &filter
    }
}

impl<S> Filter<S> for LoggingFilter {
    fn enabled(&self, meta: &Metadata<'_>, _cx: &Context<'_, S>) -> bool {
        self.is_enabled(meta)
    }

    fn callsite_enabled(&self, meta: &'static Metadata<'static>) -> Interest {
        if self.is_enabled(meta) {
            Interest::always()
        } else {
            Interest::never()
        }
    }

    fn max_level_hint(&self) -> Option<LevelFilter> {
        Some(LevelFilter::from_level(self.trace_level))
    }
}
