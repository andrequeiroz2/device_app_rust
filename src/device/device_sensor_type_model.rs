use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum SensorType {
    HumidityTemperature(HumidityTemperatureModels),
    Humidity(HumidityModels),
    Temperature(TemperatureModels),
    Pressure(PressureModels),
    Gas(GasModels),
    Particle(ParticleModels),
    Magnetic(MagneticModels),
    Axis3(Axis3Models),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum HumidityTemperatureModels {
    DHT11,
    DHT22,
    AM2302,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum HumidityModels {
    HIH4030,
    SHT21,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum TemperatureModels {
    LM35,
    DS18B20,
    TMP36,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum PressureModels {
    BMP180,
    BMP280,
    BME280,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum GasModels {
    MQ2,
    MQ7,
    MQ135,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ParticleModels {
    GP2Y1010AU0F,
    PMS5003,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum MagneticModels {
    HMC5883L,
    QMC5883,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Axis3Models {
    MPU6050,
    MPU9250,
    LSM6DS3,
}