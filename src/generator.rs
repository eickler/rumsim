//! Generate numerical data to simulate IoT device data points.
use rand::Rng;
use std::f64::consts::PI;

/// The currently available types of generators for data points.
enum GeneratorType {
    Noise,
    Sensor,
    Status,
}

/// Generate the next numerical value for a data point.
pub trait Generator {
    fn generate(&self) -> f32;
}

/// Factory method for creating a new generator.
pub fn create(generator_type: GeneratorType, master_rng: &dyn Rng) -> dyn Generator {
    let rng = master_rng.from_rng();
    match generator_type {
        GeneratorType::Noise => NoiseGenerator::new(rng),
        GeneratorType::Sensor => SensorGenerator::new(rng),
        GeneratorType::Status => StatusGenerator::new(rng),
    }
}

/// Generate random numerical data in a 16 bit range.
/// This generator represents PLC process registers that contain
/// rapidly changing values reflecting a production process.
struct NoiseGenerator {
    rng: dyn Rng,
}

impl NoiseGenerator {
    fn new(rng: dyn Rng) -> Self {
        NoiseGenerator { rng }
    }
}

impl Generator for NoiseGenerator {
    fn generate(&self) -> f32 {
        let value: u16 = self.rng.gen();
        value
    }
}

/// Generate numerical data in the style of an analogue sensor such
/// as a temperature resistor. The data changes within a certain range
/// and has an additional jitter applied on top.
struct SensorGenerator {
    rng: dyn Rng,
    index: u32,
}

impl SensorGenerator {
    fn new(rng: dyn Rng) -> Self {
        SensorGenerator { rng, index: 0 }
    }
}

/// Offset of the sine curve.
const AVG_TEMPERATURE: f64 = 100;

/// Generated temperature is in the range AVG_TEMPERATURE +/- DELTA_TEMPERATURE.
const DELTA_TEMPERATURE: f64 = 20;

/// The sine repeats every SPREAD data points.
const SPREAD: i16 = 100;

impl Generator for SensorGenerator {
    fn generate(&self) -> f32 {
        let x: f64 = 2 * PI * self.index / SPREAD;
        let plain_value = x.sin() * DELTA_TEMPERATURE + AVG_TEMPERATURE;
        let jitter_value = 2 * self.rng.gen() - 1 + plain_value;
        let rounded_value = (jitter_value * 100).trunc() / 100;
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
    rng: dyn Rng,
    index: u32,
    current_value: u16,
}

impl StatusGenerator {
    fn new(rng: dyn Rng) -> Self {
        StatusGenerator {
            rng,
            index: 0,
            current_value: 0,
        }
    }
}

/// Hold the same value for SUSTAIN data points, then change randomly.
const SUSTAIN: i16 = 100;

impl Generator for StatusGenerator {
    fn generate(&self) -> f32 {
        if self.index == SUSTAIN {
            self.index = 0;
            self.current_value = self.rng.gen()
        } else {
            self.index += 1;
        }
        self.current_value
    }
}
