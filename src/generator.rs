//! Generate numerical data to simulate IoT device data points.
use rand::{rngs::ThreadRng, Rng};
use std::f64::consts::PI;

/// The currently available types of generators for data points.
pub enum GeneratorType {
    Noise,
    Sensor,
    Status,
}

/// Generate the next numerical value for a data point.
pub trait Generator {
    fn generate(&mut self) -> f64;
}

/// Factory method for creating a new generator.
pub fn create(generator_type: GeneratorType) -> Box<dyn Generator> {
    let rng = ThreadRng::default(); // Originally, I wanted this to be seeded explicitly,  but couldn't get it done.
    match generator_type {
        GeneratorType::Noise => Box::new(NoiseGenerator::new(rng)),
        GeneratorType::Sensor => Box::new(SensorGenerator::new(rng)),
        GeneratorType::Status => Box::new(StatusGenerator::new(rng)),
    }
}

/// Generate random numerical data in a 16 bit range.
/// This generator represents PLC process registers that contain
/// rapidly changing values reflecting a production process.
struct NoiseGenerator {
    rng: ThreadRng,
}

impl NoiseGenerator {
    fn new(rng: ThreadRng) -> Self {
        NoiseGenerator { rng }
    }
}

impl Generator for NoiseGenerator {
    fn generate(&mut self) -> f64 {
        let value: u16 = self.rng.gen();
        value.into()
    }
}

/// Generate numerical data in the style of an analogue sensor such
/// as a temperature resistor. The data changes within a certain range
/// and has an additional jitter applied on top.
struct SensorGenerator {
    rng: ThreadRng,
    index: u32,
}

impl SensorGenerator {
    fn new(rng: ThreadRng) -> Self {
        SensorGenerator { rng, index: 0 }
    }
}

/// Offset of the sine curve.
const AVG_TEMPERATURE: f64 = 100.0;

/// Generated temperature is in the range AVG_TEMPERATURE +/- DELTA_TEMPERATURE.
const DELTA_TEMPERATURE: f64 = 20.0;

/// The sine repeats every SPREAD data points.
const SPREAD: u32 = 100;

impl Generator for SensorGenerator {
    fn generate(&mut self) -> f64 {
        let x: f64 = 2.0 * PI * f64::from(self.index) / f64::from(SPREAD);
        let plain_value = x.sin() * DELTA_TEMPERATURE + AVG_TEMPERATURE;
        let jitter_value: f64 = 2.0 * self.rng.gen::<f64>() - 1.0 + plain_value;
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
    rng: ThreadRng,
    index: u16,
    current_value: u16,
}

impl StatusGenerator {
    fn new(rng: ThreadRng) -> Self {
        StatusGenerator {
            rng,
            index: 0,
            current_value: 0,
        }
    }
}

/// Hold the same value for SUSTAIN data points, then change randomly.
const SUSTAIN: u16 = 100;

impl Generator for StatusGenerator {
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
