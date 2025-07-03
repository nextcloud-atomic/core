use clap::{Args, Parser, Subcommand};
use tonic::Request;
use grpc_common::client::{retrieve_grpc_channel};
use grpc_nca_system::api;
use grpc_nca_system::api::credentials_client::CredentialsClient;
use grpc_nca_system::api::Empty;
use grpc_nca_system::api::nextcloud_client::NextcloudClient;
use grpc_nca_system::api::storage_client::StorageClient;
use grpc_nca_system::api::system_client::SystemClient;

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands
}

#[derive(Subcommand)]
enum Commands {
    Storage (StorageArgs),
    Credentials(CredentialsArgs),
    Nextcloud(NextcloudArgs),
    System(SystemArgs)
}

#[derive(Args)]
struct StorageArgs {
    #[command(subcommand)]
    command: StorageCommands
}

#[derive(Subcommand)]
enum StorageCommands {
    AddPassword {
        primary_password: String,
    }
}

#[derive(Args)]
struct CredentialsArgs {
    #[command(subcommand)]
    command: CredentialsCommands
}

#[derive(Subcommand)]
enum CredentialsCommands {
    GenerateSalt,
    SetNextcloudAdminPassword {
        admin_password: String
    },
    SetBackupPassword {
        primary_password: String
    },
    Init {
        primary_password: String
    }
}

#[derive(Args)]
struct NextcloudArgs {
    #[command(subcommand)]
    command: NextcloudCommands
}

#[derive(Subcommand)]
enum NextcloudCommands {
    Configure {
        domain: Option<String>,
        admin_password: Option<String>,
    },
    HardReset
}

#[derive(Args)]
struct SystemArgs {
    #[command(subcommand)]
    command: SystemCommands
}

#[derive(Subcommand)]
enum SystemCommands {
    UnlockFromSystemCredentials
}

#[tokio::main]
async fn main() -> Result<(), String> {
    let cli = Cli::parse();
    
    let channel = retrieve_grpc_channel(
        "NCA_SYSTEM_ADDRESS", 
        "NCATOMIC_SOCKETS_PATH",
        "/run/ncatomic",
        "nca-system.sock",
        "http://system.nextcloudatomic.local"
    ).await
        .map_err(|e| e.to_string())?;
    
    let success_msg: String = match cli.command {
        Commands::Credentials(args) => {
            let mut client = CredentialsClient::new(channel);
            // let mut client = ConfigClient::connect("http://127.0.0.1:5051").await
            //     .map_err(|e| format!("Failed to connect to grpc service: {e:?}"))?;
            match args.command {
                CredentialsCommands::GenerateSalt => {
                    client.generate_salt(Request::new(Empty{})).await
                        .map_err(|e| e.to_string())?;
                    Ok::<String, String>("Salt generated successfully".to_string())
                },
                CredentialsCommands::SetBackupPassword {primary_password} => {
                    client.set_backup_password(Request::new(api::PrimaryPassword{value: primary_password})).await
                        .map_err(|e| e.to_string())?;
                    Ok("Backup password set successfully".to_string())
                },
                CredentialsCommands::SetNextcloudAdminPassword {admin_password} => {
                    client.set_nextcloud_admin_password(Request::new(api::PrimaryPassword{value: admin_password})).await
                        .map_err(|e| e.to_string())?;
                    Ok("Successfully updated nextcloud admin password".to_string())
                },
                CredentialsCommands::Init {primary_password} => {
                    client.initialize_credentials(Request::new(api::PrimaryPassword{value: primary_password})).await
                        .map_err(|e| e.to_string())?;
                    Ok("Successfully initialized credentials".to_string())
                }
            }
        },
        Commands::Storage(args) => {
            let mut client = StorageClient::new(channel);
            // let mut client = StorageClient::connect("http://127.0.0.1:5051").await
            //     .map_err(|e| format!("Failed to connect to grpc service: {e:?}"))?;
            match args.command {
                StorageCommands::AddPassword { primary_password } => {
                    let pw_result = client.add_disk_encryption_password(Request::new(api::PrimaryPassword { value: primary_password })).await
                        .map_err(|e| e.to_string())?;
                    Ok::<String, String>(format!("Successfully added disk encryption password to disks: '{}'", pw_result.into_inner().password))
                }
            }
        },
        Commands::Nextcloud(args) => {
            let mut client = NextcloudClient::new(channel);
            match args.command {
                NextcloudCommands::Configure { domain, admin_password} => {
                    let _result = client.update_config(Request::new(api::NextcloudConfig{
                        domain,
                        admin_password
                    })).await
                        .map_err(|e| e.to_string())?.into_inner();
                    Ok::<String, String>("Successfully updated nextcloud config".to_string())
                },
                NextcloudCommands::HardReset => {
                    let result = client
                        .hard_reset(Request::new(api::Empty{}))
                        .await
                        .map_err(|e| e.to_string())?
                        .into_inner();
                    Ok::<String, String>(
                        format!("Successfully performed Nextcloud hard reset: \n{} --- \n{}",
                                result.stdout.unwrap_or("".to_string()),
                                result.stderr.unwrap_or("".to_string())))
                }
            }
        },
        Commands::System(args) => {
            let mut client = SystemClient::new(channel);
            match args.command {
                SystemCommands::UnlockFromSystemCredentials => {
                    client.unlock_from_systemd_credentials(Request::new(api::Empty{}))
                        .await
                        .map_err(|e| e.to_string())?;
                    Ok::<String, String>("Successfully unlocked system from systemd credentials".to_string())
                }
            }
        }
    }?;

    println!("{success_msg}");
    Ok(())
}
