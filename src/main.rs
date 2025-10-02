use crate::client::{ApiClient, JobRequestDTO, TleData};
use chrono::{DateTime, NaiveDate, NaiveDateTime, NaiveTime, Utc};
use clap::{Parser, Subcommand};
use inquire::Text;

mod client;
mod error;

#[derive(Parser, Debug)]
#[command(version, about = "Ground Station CLI", long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Add a new tracking job to the ground station
    #[command(name = "add-job")]
    AddJob,
}

struct UserInput {
    start_datetime: DateTime<Utc>,
    end_datetime: DateTime<Utc>,
    tle_data: TleData,
    rx_frequency: f64,
    tx_frequency: f64,
}

/// Parse user-friendly date/time format to UTC DateTime
fn parse_user_datetime(
    date_str: &str,
    time_str: &str,
) -> Result<DateTime<Utc>, Box<dyn std::error::Error>> {
    // Parse date in format YYYY-MM-DD
    let date = NaiveDate::parse_from_str(date_str, "%Y-%m-%d")?;

    // Parse time in format HH:MM or HH:MM:SS
    let time = if time_str.matches(':').count() == 1 {
        // Format HH:MM, add seconds
        NaiveTime::parse_from_str(&format!("{}:00", time_str), "%H:%M:%S")?
    } else {
        // Format HH:MM:SS
        NaiveTime::parse_from_str(time_str, "%H:%M:%S")?
    };

    let naive_datetime = NaiveDateTime::new(date, time);

    Ok(DateTime::<Utc>::from_naive_utc_and_offset(
        naive_datetime,
        Utc,
    ))
}

/// Collect datetime input from user
fn get_datetime_input(
    label: &str,
    date_placeholder: &str,
    time_placeholder: &str,
) -> Result<DateTime<Utc>, Box<dyn std::error::Error>> {
    let date = Text::new(&format!("📅 {} date:", label))
        .with_placeholder(date_placeholder)
        .prompt()?;

    let time = Text::new(&format!("🕐 {} time:", label))
        .with_placeholder(time_placeholder)
        .prompt()?;

    parse_user_datetime(&date, &time)
}

/// Collect TLE data from user
fn get_tle_input() -> Result<TleData, Box<dyn std::error::Error>> {
    let sat_name = Text::new("🛰️ Satellite name:")
        .with_placeholder("ISS (ZARYA)")
        .prompt()?;

    let tle_line1 = Text::new("📡 TLE Line 1:")
        .with_placeholder("1 25544U 98067A   25235.75642456  .00011222  00000+0  20339-3 0  9993")
        .prompt()?;

    let tle_line2 = Text::new("📡 TLE Line 2:")
        .with_placeholder("2 25544  51.6355 332.1708 0003307 260.2831  99.7785 15.50129787525648")
        .prompt()?;

    Ok(TleData {
        tle0: sat_name.trim().to_string(),
        tle1: tle_line1.trim().to_string(),
        tle2: tle_line2.trim().to_string(),
    })
}

/// Collect frequency input from user
fn get_frequency_input(label: &str, placeholder: &str) -> Result<f64, Box<dyn std::error::Error>> {
    let freq_str = Text::new(&format!("📡 {} frequency (Hz):", label))
        .with_placeholder(placeholder)
        .prompt()?;

    Ok(freq_str.parse()?)
}

/// Collect all job information from user
fn collect_job_info() -> Result<UserInput, Box<dyn std::error::Error>> {
    println!("🚀 Creating a new tracking job...\n");

    let start_datetime = get_datetime_input("Start", "2025-10-02", "12:00")?;
    let end_datetime = get_datetime_input("End", "2025-10-02", "12:15")?;
    let tle_data = get_tle_input()?;
    let rx_frequency = get_frequency_input("RX", "145800000")?;
    let tx_frequency = get_frequency_input("TX", "437500000")?;

    Ok(UserInput {
        start_datetime,
        end_datetime,
        tle_data,
        rx_frequency,
        tx_frequency,
    })
}

/// Submit job to API
async fn submit_job(
    client: &ApiClient,
    input: UserInput,
) -> Result<(), Box<dyn std::error::Error>> {
    let job = JobRequestDTO{
        start: input.start_datetime,
        end: input.end_datetime,
        tle: input.tle_data,
        rx_frequency: input.rx_frequency,
        tx_frequency: input.tx_frequency,
    };

    println!("\n📡 Submitting job to ground station...");

    let response = client.add_job(job).await?;
    println!("✅ Job submitted successfully: {}", response.status);

    Ok(())
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let client = match ApiClient::new() {
        Ok(client) => client,
        Err(e) => {
            eprintln!("Failed to initialize API client: {}", e);
            std::process::exit(1);
        }
    };

    match args.command {
        Commands::AddJob => {
            let input = match collect_job_info() {
                Ok(input) => input,
                Err(e) => {
                    eprintln!("Error collecting input: {}", e);
                    std::process::exit(1);
                }
            };

            if let Err(e) = submit_job(&client, input).await {
                eprintln!("Failed to submit job: {}", e);
                std::process::exit(1);
            }
        }
    }
}
