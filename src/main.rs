//! Fasco - Fast AI chat client powered by Cerebras GLM-4.7.

use anyhow::Result;
use futures::StreamExt;
use rustyline::Editor;
use rustyline::history::DefaultHistory;
use std::fs::OpenOptions;
use std::io::Write;
use tracing::{error, info, warn};
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::EnvFilter;

mod client;
mod models;

use client::CerebrasClient;
use models::Message;

/// System prompt for the assistant.
const SYSTEM_PROMPT: &str = "You are a concise, helpful assistant. Be direct and brief.";

/// ANSI color codes for terminal output.
mod colors {
    pub const RESET: &str = "\x1b[0m";
    pub const BOLD: &str = "\x1b[1m";
    pub const DIM: &str = "\x1b[2m";
    pub const GREEN: &str = "\x1b[32m";
    pub const BLUE: &str = "\x1b[34m";
    pub const YELLOW: &str = "\x1b[33m";
    pub const RED: &str = "\x1b[31m";
    pub const CYAN: &str = "\x1b[36m";
}

/// Interactive REPL for chat.
struct Repl {
    /// Cerebras API client
    client: CerebrasClient,
    /// Conversation history
    history: Vec<Message>,
    /// Readline editor
    rl: Editor<(), DefaultHistory>,
    /// Message counter for display
    message_count: usize,
}

impl Repl {
    /// Create a new REPL instance.
    fn new(api_key: String) -> Result<Self> {
        let client = CerebrasClient::new(api_key);
        let mut rl = Editor::new()?;
        let history_path = dirs::home_dir()
            .map(|p| p.join(".fasco-history"))
            .ok_or_else(|| anyhow::anyhow!("could not determine home directory"))?;

        // Load command history
        if history_path.exists() {
            let _ = rl.load_history(&history_path);
        }

        let mut repl = Self {
            client,
            history: Vec::new(),
            rl,
            message_count: 0,
        };

        // Add system prompt to history
        repl.history.push(Message::system(SYSTEM_PROMPT));

        Ok(repl)
    }

    /// Print the welcome banner.
    fn print_welcome() {
        println!();
        println!(
            "{}┌─────────────────────────────────────────────┐{}",
            colors::BOLD,
            colors::RESET
        );
        println!(
            "{}│{}  Fasco {}• Fast AI Chat (GLM-4.7){}        │{}",
            colors::CYAN,
            colors::RESET,
            colors::DIM,
            colors::CYAN,
            colors::RESET
        );
        println!(
            "{}└─────────────────────────────────────────────┘{}",
            colors::BOLD,
            colors::RESET
        );
        println!();
        println!(
            "{}Commands:{} /help • /clear • /exit",
            colors::DIM,
            colors::RESET
        );
        println!();
    }

    /// Run the REPL loop.
    async fn run(&mut self) -> Result<()> {
        Self::print_welcome();

        loop {
            let turn_number = (self.message_count / 2) + 1;
            let prompt = format!("{}[{}]{}> ", colors::BLUE, turn_number, colors::RESET);

            let input: String = match self.rl.readline(&prompt) {
                Ok(line) => line,
                Err(rustyline::error::ReadlineError::Interrupted) => {
                    info!("received interrupt, continuing");
                    println!("\n{}Use /exit to quit.{}", colors::DIM, colors::RESET);
                    continue;
                }
                Err(rustyline::error::ReadlineError::Eof) => {
                    info!("received EOF, exiting");
                    break;
                }
                Err(e) => {
                    return Err(e.into());
                }
            };

            let input = input.trim();
            if input.is_empty() {
                continue;
            }

            self.rl.add_history_entry(input)?;

            match input {
                "/exit" => {
                    info!("user requested exit");
                    break;
                }
                "/clear" => {
                    self.clear_history();
                    println!("\n{}Conversation cleared.{}", colors::DIM, colors::RESET);
                    println!();
                    continue;
                }
                "/help" => {
                    self.show_help();
                    continue;
                }
                _ => {
                    if let Err(e) = self.chat(input).await {
                        error!("chat error: {}", e);
                        println!(
                            "\n{}Error: {}{}{}\n",
                            colors::RED,
                            colors::BOLD,
                            e,
                            colors::RESET
                        );
                    }
                }
            }
        }

        self.save_history();
        println!("\n{}Goodbye!{}", colors::DIM, colors::RESET);
        Ok(())
    }

    /// Clear the conversation history.
    fn clear_history(&mut self) {
        self.history.clear();
        self.history.push(Message::system(SYSTEM_PROMPT));
        self.message_count = 0;
    }

    /// Show help message.
    fn show_help(&self) {
        println!();
        println!("{}Available Commands:{}", colors::BOLD, colors::RESET);
        println!(
            "  {}/help{}   - Show this help message",
            colors::CYAN,
            colors::RESET
        );
        println!(
            "  {}/clear{}  - Clear conversation history",
            colors::CYAN,
            colors::RESET
        );
        println!(
            "  {}/exit{}   - Exit the program",
            colors::CYAN,
            colors::RESET
        );
        println!();
    }

    /// Process a user message and stream the response.
    async fn chat(&mut self, user_input: &str) -> Result<()> {
        // Add user message to history
        self.history.push(Message::user(user_input));
        self.message_count += 1;

        println!();

        // Stream the response
        let mut full_response = String::new();
        let stream = self.client.chat_stream(self.history.clone()).await?;
        futures::pin_mut!(stream);

        while let Some(result) = stream.next().await {
            match result {
                Ok(ref delta) => {
                    print!("{}", delta);
                    std::io::stdout().flush().ok();
                    full_response.push_str(delta);
                }
                Err(ref e) => {
                    warn!("stream error: {}", e);
                }
            }
        }

        println!();
        println!();

        // Add assistant response to history
        self.history.push(Message::assistant(full_response));
        self.message_count += 1;

        Ok(())
    }

    /// Save command history to disk.
    fn save_history(&mut self) {
        if let Some(home) = dirs::home_dir() {
            let history_path = home.join(".fasco-history");
            let _ = self.rl.save_history(&history_path);
        }
    }
}

/// Initialize file logging to a temp directory.
fn init_file_logging() -> Option<WorkerGuard> {
    let log_dir = std::env::temp_dir().join("fasco");
    let _ = std::fs::create_dir_all(&log_dir);

    let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
    let log_path = log_dir.join(format!("fasco_{}.log", timestamp));

    // Create or append to log file
    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
        .ok()?;

    // Update symlink to latest log
    #[cfg(unix)]
    {
        use std::os::unix::fs::symlink;
        let latest_path = log_dir.join("latest");
        let _ = std::fs::remove_file(&latest_path);
        let _ = symlink(&log_path, &latest_path);
    }

    let (non_blocking, guard) = tracing_appender::non_blocking(file);

    tracing_subscriber::fmt()
        .with_writer(non_blocking)
        .with_env_filter(EnvFilter::from_default_env().add_directive(tracing::Level::INFO.into()))
        .init();

    Some(guard)
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize file logging
    let _guard = init_file_logging();

    // Get API key from environment
    let api_key = std::env::var("CEREBRAS_API_KEY").or_else(|_| {
        // Try loading from .env file
        dotenvy::dotenv().ok();
        std::env::var("CEREBRAS_API_KEY")
    });

    let api_key = match api_key {
        Ok(key) => key,
        Err(_) => {
            eprintln!(
                "\n{}Error:{} CEREBRAS_API_KEY not found.",
                colors::RED,
                colors::RESET
            );
            eprintln!(
                "\nGet your API key from: {}https://cloud.cerebras.ai/{}\n",
                colors::CYAN,
                colors::RESET
            );
            eprintln!("{}Then run:{}", colors::DIM, colors::RESET);
            eprintln!(
                "  {}export{} CEREBRAS_API_KEY=sk-...",
                colors::GREEN,
                colors::RESET
            );
            eprintln!(
                "  {}or{} create a .env file: CEREBRAS_API_KEY=sk-...\n",
                colors::GREEN,
                colors::RESET
            );
            std::process::exit(1);
        }
    };

    // Validate API key format
    if !api_key.starts_with("sk-") {
        warn!("API key does not start with 'sk-' - may be invalid");
    }

    info!("starting Fasco with GLM-4.7 model");

    // Run the REPL
    let mut repl = Repl::new(api_key)?;
    repl.run().await?;

    Ok(())
}
