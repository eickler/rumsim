//! Generate numerical data to simulate IoT device data points.
//! TODO: I wanted this originally to be seedable, but the thread-safe RNG is not seedable.
use rand::{rngs::StdRng, Rng, SeedableRng};
use std::f64::consts::PI;

/// The currently available types of generators for data points.
pub enum GeneratorType {
    Noise,
    Sensor,
    Status,
}

/// Generate the next numerical value for a data point.
pub trait Generator {
    fn name(&mut self) -> &str;
    fn generate(&mut self) -> f64;
}

/// Factory method for creating a new generator.
pub fn create(generator_type: GeneratorType, seed: u64) -> Box<dyn Generator> {
    match generator_type {
        GeneratorType::Noise => Box::new(NoiseGenerator::new(seed)),
        GeneratorType::Sensor => Box::new(SensorGenerator::new(seed)),
        GeneratorType::Status => Box::new(StatusGenerator::new(seed)),
    }
}

/// Generate random numerical data in a 16 bit range.
/// This generator represents PLC process registers that contain
/// rapidly changing values reflecting a production process.
struct NoiseGenerator {
    name: String,
    rng: StdRng,
}

impl NoiseGenerator {
    fn new(seed: u64) -> Self {
        let seed_str = seed.to_string();
        let name = "noise_".to_owned() + &seed_str;
        NoiseGenerator {
            name,
            rng: StdRng::seed_from_u64(seed),
        }
    }
}

impl Generator for NoiseGenerator {
    fn name(&mut self) -> &str {
        &self.name
    }

    fn generate(&mut self) -> f64 {
        let value: u16 = self.rng.gen();
        value.into()
    }
}

/// Generate numerical data in the style of an analogue sensor such
/// as a temperature resistor. The data changes within a certain range
/// and has an additional jitter applied on top.
struct SensorGenerator {
    name: String,
    rng: StdRng,
    index: u32,
}

impl SensorGenerator {
    fn new(seed: u64) -> Self {
        let seed_str = seed.to_string();
        let name = "sensor_".to_owned() + &seed_str;
        SensorGenerator {
            name,
            rng: StdRng::seed_from_u64(seed),
            index: 0,
        }
    }
}

/// Offset of the sine curve.
const AVG_TEMPERATURE: f64 = 100.0;

/// Generated temperature is in the range AVG_TEMPERATURE +/- DELTA_TEMPERATURE.
const DELTA_TEMPERATURE: f64 = 20.0;

/// The jitter added
const JITTER: f64 = 2.0;

/// The sine repeats every SPREAD data points.
const SPREAD: u32 = 100;

impl Generator for SensorGenerator {
    fn name(&mut self) -> &str {
        &self.name
    }

    fn generate(&mut self) -> f64 {
        let x: f64 = 2.0 * PI * f64::from(self.index) / f64::from(SPREAD);
        let plain_value = x.sin() * DELTA_TEMPERATURE + AVG_TEMPERATURE;
        let jitter_value: f64 = JITTER * 2.0 * self.rng.gen::<f64>() - JITTER + plain_value;
        let rounded_value = (jitter_value * 100.0).trunc() / 100.0;
        if self.index == SPREAD {
            self.index = 0;
        } else {
            self.index += 1;
        }
        rounded_value
    }
}

/// Generate data in the style of PLC status registers. The data is
/// mostly constant with an occasional change reflecting, e.g., an
/// alarm condition or a reconfiguration.
struct StatusGenerator {
    name: String,
    rng: StdRng,
    index: u16,
    current_value: u16,
}

impl StatusGenerator {
    fn new(seed: u64) -> Self {
        let seed_str = seed.to_string();
        let name = "status_".to_owned() + &seed_str;
        StatusGenerator {
            name,
            rng: StdRng::seed_from_u64(seed),
            index: 0,
            current_value: 0,
        }
    }
}

/// Hold the same value for SUSTAIN data points, then change randomly.
const SUSTAIN: u16 = 100;

impl Generator for StatusGenerator {
    fn name(&mut self) -> &str {
        &self.name
    }

    fn generate(&mut self) -> f64 {
        if self.index == SUSTAIN {
            self.index = 0;
            self.current_value = self.rng.gen()
        } else {
            self.index += 1;
        }
        self.current_value.into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_noise_generator() {
        let mut gen = NoiseGenerator::new(1);
        let value = gen.generate();
        assert!((0.0..u16::MAX as f64).contains(&value));
    }

    #[test]
    fn test_sensor_generator() {
        let mut gen = SensorGenerator::new(1);
        let mut value = gen.generate();

        assert!((AVG_TEMPERATURE - JITTER..AVG_TEMPERATURE + JITTER).contains(&value));

        for _i in 0..SPREAD - 1 {
            value = gen.generate();
        }

        assert!((AVG_TEMPERATURE - JITTER..AVG_TEMPERATURE + JITTER).contains(&value));
    }

    #[test]
    fn test_status_generator() {
        let mut gen = StatusGenerator::new(1);
        let start_value = gen.generate();

        for _i in 0..SUSTAIN - 1 {
            let value = gen.generate();
            assert_eq!(start_value, value);
        }

        let next_value = gen.generate(); // With a fixed seed, we can avoid the 1/65536 chance that the same value is generated.
        assert_ne!(start_value, next_value);
    }

    #[test]
    fn test_factory() {
        // TODO: Can I test the type that is returned by the factory?
        let mut noise = create(GeneratorType::Noise, 1);
        noise.generate();
        let mut sensor = create(GeneratorType::Sensor, 1);
        sensor.generate();
        let mut status = create(GeneratorType::Status, 1);
        status.generate();
    }
}
