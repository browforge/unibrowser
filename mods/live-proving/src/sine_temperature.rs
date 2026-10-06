use std::f64::consts::TAU;

use unibrowser_live::{DataSample, DataSource, DataValue, SourceId, SourceTiming};

pub struct SineTemperatureSource {
    id: SourceId,
    base_c: f64,
    amplitude_c: f64,
    period_sec: f64,
}

impl Default for SineTemperatureSource {
    fn default() -> Self {
        Self {
            id: SourceId("live-proving/sine-temperature".into()),
            base_c: 20.0,
            amplitude_c: 5.0,
            period_sec: 4.0,
        }
    }
}

impl DataSource for SineTemperatureSource {
    fn id(&self) -> &SourceId {
        &self.id
    }

    fn timing(&self) -> SourceTiming {
        SourceTiming::live()
    }

    fn poll(&mut self, at_sec: f64) -> Vec<DataSample> {
        let t = at_sec % self.period_sec;
        let value = self.base_c
            + self.amplitude_c * (TAU * t / self.period_sec).sin();
        vec![DataSample {
            channel: "temperature".into(),
            value: DataValue::Number(value),
            pts_sec: at_sec,
        }]
    }
}
