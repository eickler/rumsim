#[derive(PartialEq, Debug)]
pub struct StartParam {
    pub devices: u16,
    pub data_points: u16,
    pub wait_time_secs: u16,
}

#[derive(PartialEq, Debug)]
pub enum Command {
    Start(StartParam),
    Stop,
}

#[derive(PartialEq, Debug)]
pub enum CommandErr {
    EmptyCommand,
    InvalidArguments,
    UnknownCommand,
}

fn parse_start(parts: &Vec<&str>) -> Result<Command, CommandErr> {
    if parts.len() != 4 {
        return Err(CommandErr::InvalidArguments);
    }
    let devices = parts[1]
        .parse::<u16>()
        .map_err(|_| CommandErr::InvalidArguments)?;
    let data_points = parts[2]
        .parse::<u16>()
        .map_err(|_| CommandErr::InvalidArguments)?;
    let wait_time = parts[3]
        .parse::<u16>()
        .map_err(|_| CommandErr::InvalidArguments)?;
    Ok(Command::Start(StartParam {
        devices,
        data_points,
        wait_time_secs: wait_time,
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
            let reference = Ok(Command::Start(StartParam {
                devices: 10,
                data_points: 20,
                wait_time_secs: 30,
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
