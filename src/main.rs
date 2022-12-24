use std::sync::Arc;
use std::thread;
use std::time::Duration;
use esp_idf_hal::adc::{AdcChannelDriver, AdcDriver, Atten11dB};
use esp_idf_hal::adc::config::Config;
use esp_idf_hal::peripherals::Peripherals;
use esp_idf_sys as _;
use esp_idf_hal::gpio::Gpio14;
use esp_idf_hal::gpio::Gpio12;
use esp_idf_hal::gpio::Gpio13;
use esp_idf_hal::ledc::*;
use esp_idf_hal::prelude::FromValueType;


fn main() -> anyhow::Result<()> {
    esp_idf_sys::link_patches();

    let peripherals = Peripherals::take().unwrap();
    let mut adc = AdcDriver::new(peripherals.adc2, &Config::new().calibration(true))?;
    let mut r_adc_pin: AdcChannelDriver<'_, Gpio13, Atten11dB<_>> = AdcChannelDriver::new(peripherals.pins.gpio13)?;
    let mut g_adc_pin: AdcChannelDriver<'_, Gpio12, Atten11dB<_>> = AdcChannelDriver::new(peripherals.pins.gpio12)?;
    let mut b_adc_pin: AdcChannelDriver<'_, Gpio14, Atten11dB<_>> = AdcChannelDriver::new(peripherals.pins.gpio14)?;

    let config = config::TimerConfig::new().frequency(255.kHz().into());
    let timer = Arc::new(LedcTimerDriver::new(peripherals.ledc.timer0, &config)?);
    let mut red_pwm = LedcDriver::new(peripherals.ledc.channel0, timer.clone(), peripherals.pins.gpio15)?;
    let mut green_pwm = LedcDriver::new(peripherals.ledc.channel1, timer.clone(), peripherals.pins.gpio2)?;
    let mut blue_pwm = LedcDriver::new(peripherals.ledc.channel2, timer.clone(), peripherals.pins.gpio4)?;

    let mut colors: [usize; 3] = [0, 0, 0];
    loop {
        for i in 0..3 {
            let pin = match i {
                0 => adc.read(&mut b_adc_pin),
                1 => adc.read(&mut g_adc_pin),
                2 => adc.read(&mut r_adc_pin),
                _ => Ok(0)
            };
            colors[i] = map(pin.unwrap() as isize, 0, 4096, 0, 255) as usize;
        }

        red_pwm.set_duty(256 - colors[0] as u32)?;
        green_pwm.set_duty(256 - colors[1] as u32)?;
        blue_pwm.set_duty(256 - colors[2] as u32)?;
        thread::sleep(Duration::from_millis(10));
    }
}

fn map(x: isize, in_min: isize, in_max: isize, out_min: isize, out_max: isize) -> isize {
    let run: isize = in_max - in_min;
    if run == 0 {
        println!("map(): Invalid input range min == max");
        return -1;
    }
    let rise = out_max - out_min;
    let delta = x - in_min;
    (delta * rise) / run + out_min
}
