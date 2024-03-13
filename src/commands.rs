use std::time::Duration;

use crate::simulation::SimulationParameters;

#[derive(PartialEq, Debug)]
pub enum Command {
    Start(SimulationParameters),
    Stop,
}

#[derive(PartialEq, Debug)]
pub enum CommandErr {
    EmptyCommand,
    InvalidArguments,
    UnknownCommand,
}

fn parse_start(parts: &Vec<&str>) -> Result<Command, CommandErr> {
    if parts.len() != 4 && parts.len() != 5 {
        return Err(CommandErr::InvalidArguments);
    }
    let devices = parts[1]
        .parse::<usize>()
        .map_err(|_| CommandErr::InvalidArguments)?;
    let data_points = parts[2]
        .parse::<usize>()
        .map_err(|_| CommandErr::InvalidArguments)?;
    let wait_time_secs = parts[3]
        .parse::<u16>()
        .map_err(|_| CommandErr::InvalidArguments)?;
    let mut seed: u64 = 1;
    if parts.len() == 5 {
        seed = parts[4]
            .parse::<u64>()
            .map_err(|_| CommandErr::InvalidArguments)?;
    }
    Ok(Command::Start(SimulationParameters {
        devices,
        data_points,
        wait_time: Duration::from_secs(wait_time_secs as u64),
        seed,
    }))
}

pub fn parse(command_str: &String) -> Result<Command, CommandErr> {
    let parts: Vec<&str> = command_str.split_whitespace().collect();
    if parts.is_empty() {
        return Err(CommandErr::EmptyCommand);
    }

    match parts[0] {
        "start" => parse_start(&parts),
        "stop" => Ok(Command::Stop),
        _ => Err(CommandErr::UnknownCommand),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stop_command() {
        let command = &"stop".to_string();
        let reference = Ok(Command::Stop);
        assert_eq!(reference, parse(command));
    }

    #[test]
    fn start_command() {
        {
            let command = "start 10 20 30".to_string();
            let reference = Ok(Command::Start(SimulationParameters {
                devices: 10,
                data_points: 20,
                wait_time: Duration::from_secs(30),
                seed: 1,
            }));
            assert_eq!(reference, parse(&command));
        }
        {
            let command = "start".to_string();
            let reference = Err(CommandErr::InvalidArguments);
            assert_eq!(reference, parse(&command));
        }
        {
            let command = "start 10".to_string();
            let reference = Err(CommandErr::InvalidArguments);
            assert_eq!(reference, parse(&command));
        }
        {
            let command = "start foo".to_string();
            let reference = Err(CommandErr::InvalidArguments);
            assert_eq!(reference, parse(&command));
        }
    }

    #[test]
    fn no_command() {
        let command = "".to_string();
        let reference = Err(CommandErr::EmptyCommand);
        assert_eq!(reference, parse(&command));
    }

    #[test]
    fn not_implemented_command() {
        let command = "foo 100!".to_string();
        let reference = Err(CommandErr::UnknownCommand);
        assert_eq!(reference, parse(&command));
    }
}
