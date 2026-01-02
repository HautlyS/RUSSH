//! russh SSH CLI
//!
//! Command-line interface for the russh SSH application.
//!
//! # Requirements Coverage
//! - Requirement 7.1: CLI interface

use clap::{Parser, Subcommand};
use russh_ssh::session::{SessionManager, SessionProfile};
use russh_ssh::session::profile::AuthConfig;
use russh_ssh::ssh::{SshClient, SshConfig, AuthMethod, PortForward, PortForwarder};
use std::path::PathBuf;
use std::time::Duration;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Parser)]
#[command(name = "russh")]
#[command(author, version, about = "russh SSH - Secure P2P SSH connections", long_about = None)]
struct Cli {
    /// Enable verbose output
    #[arg(short, long)]
    verbose: bool,

    /// Configuration directory
    #[arg(short, long, default_value = "~/.russh")]
    config_dir: String,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Connect to a remote host
    Connect {
        /// Host to connect to (user@host:port or profile name)
        #[arg(value_name = "TARGET")]
        target: String,

        /// Use password authentication
        #[arg(short, long)]
        password: bool,

        /// Path to private key
        #[arg(short, long)]
        identity: Option<PathBuf>,

        /// Local port forward (local:remote_host:remote_port)
        #[arg(short = 'L', long)]
        local_forward: Vec<String>,

        /// Execute command instead of shell
        #[arg(short, long)]
        command: Option<String>,
    },
    /// Manage session profiles
    Profile {
        #[command(subcommand)]
        action: ProfileAction,
    },
    /// Show version and system information
    Version,
}

#[derive(Subcommand)]
enum ProfileAction {
    /// List all profiles
    List,
    /// Add a new profile
    Add {
        /// Profile name
        name: String,
        /// Host address
        host: String,
        /// Username
        #[arg(short, long)]
        user: String,
        /// Port
        #[arg(short, long, default_value = "22")]
        port: u16,
    },
    /// Remove a profile
    Remove {
        /// Profile name
        name: String,
    },
    /// Show profile details
    Show {
        /// Profile name
        name: String,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    let filter = if std::env::var("RUST_LOG").is_ok() {
        tracing_subscriber::EnvFilter::from_default_env()
    } else {
        tracing_subscriber::EnvFilter::new("warn")
    };

    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(filter)
        .init();

    let cli = Cli::parse();

    if cli.verbose {
        tracing::info!("Verbose mode enabled");
    }

    // Expand config directory
    let config_dir = shellexpand::tilde(&cli.config_dir).to_string();
    let config_path = PathBuf::from(&config_dir);
    
    // Ensure config directory exists
    if !config_path.exists() {
        tokio::fs::create_dir_all(&config_path).await?;
    }

    let profiles_path = config_path.join("profiles.json");
    let manager = SessionManager::with_storage(profiles_path.clone());
    
    // Load existing profiles
    if let Err(e) = manager.load().await {
        if cli.verbose {
            tracing::warn!("Could not load profiles: {}", e);
        }
    }

    match cli.command {
        Some(Commands::Connect { target, password, identity, local_forward, command }) => {
            connect(&manager, &target, password, identity, local_forward, command).await?;
        }
        Some(Commands::Profile { action }) => {
            handle_profile_action(&manager, action).await?;
            manager.save().await?;
        }
        Some(Commands::Version) => {
            println!("russh SSH version {}", env!("CARGO_PKG_VERSION"));
            println!("Built with Rust");
            println!();
            println!("Features:");
            println!("  - Async SSH connections (async-ssh2-tokio)");
            println!("  - P2P networking (Iroh)");
            println!("  - End-to-end encryption (BLAKE3 + Ring)");
            println!("  - Virtual distributed filesystem");
            println!("  - Session management");
        }
        None => {
            println!("russh SSH - Secure P2P SSH connections");
            println!();
            println!("Use --help for usage information");
            println!();
            println!("Quick start:");
            println!("  russh connect user@host       Connect to a host");
            println!("  russh profile list            List saved profiles");
            println!("  russh profile add NAME HOST   Add a new profile");
        }
    }

    Ok(())
}

async fn connect(
    manager: &SessionManager,
    target: &str,
    use_password: bool,
    identity: Option<PathBuf>,
    local_forwards: Vec<String>,
    command: Option<String>,
) -> anyhow::Result<()> {
    // Parse target: could be profile name or user@host:port
    let (host, port, username) = if target.contains('@') {
        parse_target(target)?
    } else {
        // Try to find profile by name
        if let Some(profile) = manager.get_profile_by_name(target).await {
            (profile.host, profile.port, profile.username)
        } else {
            anyhow::bail!("Unknown profile or invalid target: {}", target);
        }
    };

    println!("Connecting to {}@{}:{}...", username, host, port);

    // Determine auth method
    let auth = if use_password {
        println!("Password: ");
        let password = rpassword::read_password()?;
        AuthMethod::Password(password)
    } else if let Some(key_path) = identity {
        AuthMethod::PublicKey {
            key_path,
            passphrase: None,
        }
    } else {
        // Try default key locations
        let home = dirs::home_dir().unwrap_or_default();
        let default_keys = [
            home.join(".ssh/id_ed25519"),
            home.join(".ssh/id_rsa"),
        ];
        
        let key_path = default_keys.iter()
            .find(|p| p.exists())
            .cloned();
        
        match key_path {
            Some(path) => {
                println!("Using key: {}", path.display());
                AuthMethod::PublicKey {
                    key_path: path,
                    passphrase: None,
                }
            }
            None => {
                println!("No key found, using password authentication");
                println!("Password: ");
                let password = rpassword::read_password()?;
                AuthMethod::Password(password)
            }
        }
    };

    let config = SshConfig {
        host: host.clone(),
        port,
        username: username.clone(),
        auth,
        timeout: Duration::from_secs(30),
    };

    let mut client = SshClient::new();
    client.connect(&config).await?;

    println!("Connected!");

    // Set up port forwards
    for forward_spec in local_forwards {
        if let Some(forward) = parse_local_forward(&forward_spec) {
            match client.start_forward(forward.clone()).await {
                Ok(handle) => {
                    if let PortForward::Local { local_port, remote_host, remote_port } = &forward {
                        println!("Forwarding localhost:{} -> {}:{}", local_port, remote_host, remote_port);
                    }
                }
                Err(e) => {
                    eprintln!("Failed to set up forward: {}", e);
                }
            }
        }
    }

    // Execute command or start shell
    if let Some(cmd) = command {
        let result = client.execute(&cmd).await?;
        print!("{}", result.stdout_string());
        eprint!("{}", result.stderr_string());
        std::process::exit(result.exit_code);
    } else {
        println!("Interactive shell not yet implemented in CLI");
        println!("Use -c 'command' to execute commands");
    }

    client.disconnect().await?;
    Ok(())
}

fn parse_target(target: &str) -> anyhow::Result<(String, u16, String)> {
    // Format: user@host:port or user@host
    let parts: Vec<&str> = target.split('@').collect();
    if parts.len() != 2 {
        anyhow::bail!("Invalid target format. Use: user@host[:port]");
    }

    let username = parts[0].to_string();
    let host_port: Vec<&str> = parts[1].split(':').collect();
    
    let host = host_port[0].to_string();
    let port = if host_port.len() > 1 {
        host_port[1].parse()?
    } else {
        22
    };

    Ok((host, port, username))
}

fn parse_local_forward(spec: &str) -> Option<PortForward> {
    // Format: local_port:remote_host:remote_port
    let parts: Vec<&str> = spec.split(':').collect();
    if parts.len() != 3 {
        return None;
    }

    let local_port: u16 = parts[0].parse().ok()?;
    let remote_host = parts[1].to_string();
    let remote_port: u16 = parts[2].parse().ok()?;

    Some(PortForward::Local {
        local_port,
        remote_host,
        remote_port,
    })
}

async fn handle_profile_action(manager: &SessionManager, action: ProfileAction) -> anyhow::Result<()> {
    match action {
        ProfileAction::List => {
            let profiles = manager.list_profiles().await;
            if profiles.is_empty() {
                println!("No profiles saved.");
                println!("Use 'russh profile add' to create one.");
            } else {
                println!("Saved profiles:");
                println!();
                for profile in profiles {
                    println!("  {} - {}@{}:{}", 
                        profile.name, 
                        profile.username, 
                        profile.host, 
                        profile.port
                    );
                    if let Some(desc) = &profile.description {
                        println!("    {}", desc);
                    }
                }
            }
        }
        ProfileAction::Add { name, host, user, port } => {
            let profile = SessionProfile::new(name.clone(), host.clone(), user.clone())
                .with_port(port)
                .with_auth(AuthConfig::Agent);
            
            manager.add_profile(profile).await;
            println!("Profile '{}' added: {}@{}:{}", name, user, host, port);
        }
        ProfileAction::Remove { name } => {
            if let Some(profile) = manager.get_profile_by_name(&name).await {
                manager.remove_profile(&profile.id).await?;
                println!("Profile '{}' removed.", name);
            } else {
                println!("Profile '{}' not found.", name);
            }
        }
        ProfileAction::Show { name } => {
            if let Some(profile) = manager.get_profile_by_name(&name).await {
                println!("Profile: {}", profile.name);
                println!("  Host: {}:{}", profile.host, profile.port);
                println!("  User: {}", profile.username);
                if let Some(desc) = &profile.description {
                    println!("  Description: {}", desc);
                }
                println!("  Created: {}", profile.created_at);
                if let Some(last) = profile.last_used {
                    println!("  Last used: {}", last);
                }
                println!("  Use count: {}", profile.use_count);
            } else {
                println!("Profile '{}' not found.", name);
            }
        }
    }
    Ok(())
}
