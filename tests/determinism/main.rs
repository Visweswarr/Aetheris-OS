use std::path::PathBuf;
use std::env;
use clap::{App, Arg, SubCommand};
use determinism_harness::{DeterminismHarness, DeterminismConfig, SeedStrategy, TestCase};

mod harness;
use harness::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = App::new("Determinism Test Runner")
        .version("1.0")
        .about("Run determinism tests for Polymera OS components")
        .arg(
            Arg::with_name("config")
                .short("c")
                .long("config")
                .value_name("FILE")
                .help("Configuration file path")
                .takes_value(true)
        )
        .arg(
            Arg::with_name("cases-dir")
                .short("d")
                .long("cases-dir")
                .value_name("DIR")
                .help("Test cases directory")
                .default_value("cases")
                .takes_value(true)
        )
        .arg(
            Arg::with_name("output-dir")
                .short("o")
                .long("output-dir")
                .value_name("DIR")
                .help("Output directory for results")
                .default_value("determinism_output")
                .takes_value(true)
        )
        .arg(
            Arg::with_name("golden-dir")
                .short("g")
                .long("golden-dir")
                .value_name("DIR")
                .help("Golden files directory")
                .default_value("determinism_golden")
                .takes_value(true)
        )
        .arg(
            Arg::with_name("num-runs")
                .short("n")
                .long("num-runs")
                .value_name("NUM")
                .help("Number of test runs per case")
                .default_value("3")
                .takes_value(true)
        )
        .arg(
            Arg::with_name("update-golden")
                .long("update-golden")
                .help("Update golden files on mismatch")
        )
        .arg(
            Arg::with_name("seed-strategy")
                .long("seed-strategy")
                .value_name("STRATEGY")
                .help("Seed strategy: fixed, system-time, capture, random")
                .default_value("fixed")
                .takes_value(true)
        )
        .arg(
            Arg::with_name("fixed-seed")
                .long("fixed-seed")
                .value_name("SEED")
                .help("Fixed seed value for fixed strategy")
                .default_value("42")
                .takes_value(true)
        )
        .subcommand(
            SubCommand::with_name("run")
                .about("Run all determinism tests")
        )
        .subcommand(
            SubCommand::with_name("test")
                .about("Run a specific test case")
                .arg(
                    Arg::with_name("test-id")
                        .required(true)
                        .help("Test case ID to run")
                )
        )
        .subcommand(
            SubCommand::with_name("list")
                .about("List available test cases")
        )
        .subcommand(
            SubCommand::with_name("report")
                .about("Generate report from existing results")
        )
        .subcommand(
            SubCommand::with_name("create-case")
                .about("Create a new test case")
                .arg(
                    Arg::with_name("id")
                        .required(true)
                        .help("Test case ID")
                )
                .arg(
                    Arg::with_name("name")
                        .required(true)
                        .help("Test case name")
                )
                .arg(
                    Arg::with_name("command")
                        .required(true)
                        .help("Command to execute")
                )
                .arg(
                    Arg::with_name("args")
                        .help("Command arguments (space-separated)")
                )
        )
        .get_matches();

    // Parse configuration
    let config = parse_config(&matches)?;
    
    // Create harness
    let mut harness = DeterminismHarness::new(config);
    
    // Load test cases
    let cases_dir = PathBuf::from(matches.value_of("cases-dir").unwrap());
    if cases_dir.exists() {
        harness.load_test_cases(&cases_dir)?;
        println!("Loaded {} test cases from {}", harness.test_cases.len(), cases_dir.display());
    } else {
        println!("Test cases directory {} does not exist, creating default cases", cases_dir.display());
        create_default_test_cases(&mut harness);
    }

    // Handle subcommands
    match matches.subcommand() {
        ("run", Some(_)) => {
            run_all_tests(&harness)?;
        }
        ("test", Some(sub_matches)) => {
            let test_id = sub_matches.value_of("test-id").unwrap();
            run_single_test(&harness, test_id)?;
        }
        ("list", Some(_)) => {
            list_test_cases(&harness);
        }
        ("report", Some(_)) => {
            generate_report(&harness)?;
        }
        ("create-case", Some(sub_matches)) => {
            let id = sub_matches.value_of("id").unwrap();
            let name = sub_matches.value_of("name").unwrap();
            let command = sub_matches.value_of("command").unwrap();
            let args = sub_matches.value_of("args")
                .map(|s| s.split_whitespace().map(|s| s.to_string()).collect())
                .unwrap_or_default();
            
            create_test_case(&mut harness, id, name, command, args)?;
        }
        _ => {
            println!("No subcommand specified. Use --help for usage information.");
        }
    }

    Ok(())
}

fn parse_config(matches: &clap::ArgMatches) -> Result<DeterminismConfig, Box<dyn std::error::Error>> {
    let mut config = DeterminismConfig::default();
    
    // Parse output directory
    if let Some(output_dir) = matches.value_of("output-dir") {
        config.output_dir = PathBuf::from(output_dir);
    }
    
    // Parse golden directory
    if let Some(golden_dir) = matches.value_of("golden-dir") {
        config.golden_dir = PathBuf::from(golden_dir);
    }
    
    // Parse number of runs
    if let Some(num_runs) = matches.value_of("num-runs") {
        config.num_runs = num_runs.parse()?;
    }
    
    // Parse update golden flag
    config.update_golden = matches.is_present("update-golden");
    
    // Parse seed strategy
    let seed_strategy = matches.value_of("seed-strategy").unwrap();
    config.seed_strategy = match seed_strategy {
        "fixed" => {
            let seed: u64 = matches.value_of("fixed-seed").unwrap().parse()?;
            SeedStrategy::Fixed(seed)
        }
        "system-time" => SeedStrategy::SystemTime,
        "capture" => SeedStrategy::Capture,
        "random" => SeedStrategy::Random,
        _ => {
            eprintln!("Invalid seed strategy: {}. Using fixed strategy.", seed_strategy);
            let seed: u64 = matches.value_of("fixed-seed").unwrap().parse()?;
            SeedStrategy::Fixed(seed)
        }
    };
    
    Ok(config)
}

fn create_default_test_cases(harness: &mut DeterminismHarness) {
    // Add some default test cases
    let echo_test = TestCase {
        id: "echo-test".to_string(),
        name: "Echo Command Test".to_string(),
        command: "echo".to_string(),
        args: vec!["Hello, World!".to_string()],
        working_dir: None,
        env: std::collections::HashMap::new(),
        input: None,
        expected_exit_code: Some(0),
        timeout: Some(10),
        seed: Some(42),
    };
    harness.add_test_case(echo_test);
    
    let date_test = TestCase {
        id: "date-test".to_string(),
        name: "Date Command Test".to_string(),
        command: "date".to_string(),
        args: vec!["+%s".to_string()], // Unix timestamp
        working_dir: None,
        env: std::collections::HashMap::new(),
        input: None,
        expected_exit_code: Some(0),
        timeout: Some(10),
        seed: Some(42),
    };
    harness.add_test_case(date_test);
    
    let pwd_test = TestCase {
        id: "pwd-test".to_string(),
        name: "PWD Command Test".to_string(),
        command: "pwd".to_string(),
        args: vec![],
        working_dir: None,
        env: std::collections::HashMap::HashMap::new(),
        input: None,
        expected_exit_code: Some(0),
        timeout: Some(10),
        seed: Some(42),
    };
    harness.add_test_case(pwd_test);
}

fn run_all_tests(harness: &DeterminismHarness) -> Result<(), Box<dyn std::error::Error>> {
    println!("Running all determinism tests...");
    println!("Configuration: {:?}", harness.config);
    println!("");
    
    let start_time = std::time::Instant::now();
    
    match harness.run_all_tests() {
        Ok(results) => {
            let duration = start_time.elapsed();
            println!("✅ All tests completed successfully!");
            println!("Total results: {}", results.len());
            println!("Duration: {:?}", duration);
            
            // Generate and save report
            let report = harness.generate_report(&results)?;
            harness.save_report(&report)?;
            
            println!("Report saved to: {}", harness.config.output_dir.join("determinism_report.md").display());
        }
        Err(e) => {
            eprintln!("❌ Test execution failed: {}", e);
            return Err(e.into());
        }
    }
    
    Ok(())
}

fn run_single_test(harness: &DeterminismHarness, test_id: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("Running test case: {}", test_id);
    
    // Find the test case
    let test_case = harness.test_cases.get(test_id)
        .ok_or_else(|| format!("Test case '{}' not found", test_id))?;
    
    println!("Test: {}", test_case.name);
    println!("Command: {} {}", test_case.command, test_case.args.join(" "));
    println!("");
    
    let start_time = std::time::Instant::now();
    
    match harness.run_test_case_deterministic(test_case) {
        Ok(results) => {
            let duration = start_time.elapsed();
            println!("✅ Test completed successfully!");
            println!("Runs: {}", results.len());
            println!("Duration: {:?}", duration);
            
            // Show results summary
            for (i, result) in results.iter().enumerate() {
                println!("Run {}: hash={}, exit_code={}, time={}ms", 
                    i + 1, result.output_hash, result.exit_code, result.execution_time);
            }
            
            // Check determinism
            if results.len() > 1 {
                let first_hash = &results[0].output_hash;
                let all_identical = results.iter().all(|r| r.output_hash == *first_hash);
                
                if all_identical {
                    println!("✅ **DETERMINISTIC**: All runs produced identical output");
                } else {
                    println!("❌ **NON-DETERMINISTIC**: Output varies between runs");
                }
            }
        }
        Err(e) => {
            eprintln!("❌ Test execution failed: {}", e);
            return Err(e.into());
        }
    }
    
    Ok(())
}

fn list_test_cases(harness: &DeterminismHarness) {
    println!("Available test cases:");
    println!("");
    
    for test_case in harness.test_cases.values() {
        println!("ID: {}", test_case.id);
        println!("Name: {}", test_case.name);
        println!("Command: {} {}", test_case.command, test_case.args.join(" "));
        if let Some(ref working_dir) = test_case.working_dir {
            println!("Working Directory: {}", working_dir.display());
        }
        println!("Timeout: {}s", test_case.timeout.unwrap_or(harness.config.max_execution_time));
        println!("Seed: {:?}", test_case.seed);
        println!("---");
    }
}

fn generate_report(harness: &DeterminismHarness) -> Result<(), Box<dyn std::error::Error>> {
    println!("Generating determinism report...");
    
    // Check if output directory exists and contains results
    if !harness.config.output_dir.exists() {
        eprintln!("Output directory does not exist: {}", harness.config.output_dir.display());
        return Ok(());
    }
    
    // This would typically load existing results from files
    // For now, we'll create a simple report
    let report = format!(
        "# Determinism Test Report\n\n\
        Generated: {}\n\
        Output Directory: {}\n\
        Golden Directory: {}\n\n\
        No results found to report on.\n\
        Run tests first using the 'run' subcommand.\n",
        chrono::Utc::now().to_rfc3339(),
        harness.config.output_dir.display(),
        harness.config.golden_dir.display()
    );
    
    harness.save_report(&report)?;
    println!("Report saved to: {}", harness.config.output_dir.join("determinism_report.md").display());
    
    Ok(())
}

fn create_test_case(
    harness: &mut DeterminismHarness, 
    id: &str, 
    name: &str, 
    command: &str, 
    args: Vec<String>
) -> Result<(), Box<dyn std::error::Error>> {
    let test_case = TestCase {
        id: id.to_string(),
        name: name.to_string(),
        command: command.to_string(),
        args,
        working_dir: None,
        env: std::collections::HashMap::new(),
        input: None,
        expected_exit_code: Some(0),
        timeout: Some(30),
        seed: Some(42),
    };
    
    harness.add_test_case(test_case);
    
    println!("✅ Test case '{}' created successfully!", id);
    println!("Name: {}", name);
    println!("Command: {} {}", command, args.join(" "));
    
    Ok(())
}
