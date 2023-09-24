//! Generate numerical data to simulate IoT device data points.
use rand::{rngs::StdRng, Rng};
use std::f64::consts::PI;

/// The currently available types of generators for data points.
pub enum GeneratorType {
    Noise,
    Sensor,
    Status,
}

/// Generate the next numerical value for a data point.
pub trait Generator {
    fn generate(&mut self, rng: &mut StdRng) -> (&str, f64);
}

/// Factory method for creating a new generator.
pub fn create_generator(generator_type: GeneratorType, id: u16) -> Box<dyn Generator> {
    match generator_type {
        GeneratorType::Noise => Box::new(NoiseGenerator::new(id)),
        GeneratorType::Sensor => Box::new(SensorGenerator::new(id)),
        GeneratorType::Status => Box::new(StatusGenerator::new(id)),
    }
}

/// Generate random numerical data in a 16 bit range.
/// This generator represents PLC process registers that contain
/// rapidly changing values reflecting a production process.
struct NoiseGenerator {
    name: String,
}

impl NoiseGenerator {
    fn new(id: u16) -> Self {
        let mut name = String::from("noise_");
        name.push_str(&id.to_string());
        NoiseGenerator { name }
    }
}

impl Generator for NoiseGenerator {
    fn generate(&mut self, rng: &mut StdRng) -> (&str, f64) {
        let value: u16 = rng.gen();
        (&self.name, value.into())
    }
}

/// Generate numerical data in the style of an analogue sensor such
/// as a temperature resistor. The data changes within a certain range
/// and has an additional jitter applied on top.
struct SensorGenerator {
    name: String,
    index: u32,
}

impl SensorGenerator {
    fn new(id: u16) -> Self {
        let mut name = String::from("sensor_");
        name.push_str(&id.to_string());
        SensorGenerator { name, index: 0 }
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
    fn generate(&mut self, rng: &mut StdRng) -> (&str, f64) {
        let x: f64 = 2.0 * PI * f64::from(self.index) / f64::from(SPREAD);
        let plain_value = x.sin() * DELTA_TEMPERATURE + AVG_TEMPERATURE;
        let jitter_value: f64 = JITTER * 2.0 * rng.gen::<f64>() - JITTER + plain_value;
        let rounded_value = (jitter_value * 100.0).trunc() / 100.0;
        if self.index == SPREAD {
            self.index = 0;
        } else {
            self.index += 1;
        }
        (&self.name, rounded_value)
    }
}

/// Generate data in the style of PLC status registers. The data is
/// mostly constant with an occasional change reflecting, e.g., an
/// alarm condition or a reconfiguration.
struct StatusGenerator {
    name: String,
    index: u16,
    current_value: u16,
}

impl StatusGenerator {
    fn new(id: u16) -> Self {
        let mut name = String::from("status_");
        name.push_str(&id.to_string());
        StatusGenerator {
            name,
            index: 0,
            current_value: 0,
        }
    }
}

/// Hold the same value for SUSTAIN data points, then change randomly.
const SUSTAIN: u16 = 100;

impl Generator for StatusGenerator {
    fn generate(&mut self, rng: &mut StdRng) -> (&str, f64) {
        if self.index == SUSTAIN {
            self.index = 0;
            self.current_value = rng.gen()
        } else {
            self.index += 1;
        }
        (&self.name, self.current_value.into())
    }
}

#[cfg(test)]
mod tests {
    use rand::SeedableRng;

    use super::*;

    #[test]
    fn test_noise_generator() {
        let mut gen = NoiseGenerator::new(1);
        let (_name, value) = gen.generate(&mut StdRng::from_entropy());
        assert!((0.0..u16::MAX as f64).contains(&value));
    }

    #[test]
    fn test_sensor_generator() {
        let mut rng = StdRng::from_entropy();
        let mut gen = SensorGenerator::new(1);
        let (mut _name, mut value) = gen.generate(&mut rng);

        assert!((AVG_TEMPERATURE - JITTER..AVG_TEMPERATURE + JITTER).contains(&value));

        for _i in 0..SPREAD - 1 {
            (_name, value) = gen.generate(&mut rng);
        }

        assert!((AVG_TEMPERATURE - JITTER..AVG_TEMPERATURE + JITTER).contains(&value));
    }

    #[test]
    fn test_status_generator() {
        let mut rng = StdRng::from_entropy();
        let mut gen = StatusGenerator::new(1);
        let (_name, start_value) = gen.generate(&mut rng);

        for _i in 0..SUSTAIN - 1 {
            let (_name, value) = gen.generate(&mut rng);
            assert_eq!(start_value, value);
        }

        let (_name, next_value) = gen.generate(&mut rng);
        assert_ne!(start_value, next_value);
    }

    #[test]
    fn test_factory() {
        let mut rng = StdRng::from_entropy();
        // TODO: Can I test the type that is returned by the factory?
        let mut noise = create_generator(GeneratorType::Noise, 1);
        noise.generate(&mut rng);
        let mut sensor = create_generator(GeneratorType::Sensor, 1);
        sensor.generate(&mut rng);
        let mut status = create_generator(GeneratorType::Status, 1);
        status.generate(&mut rng);
    }
}
