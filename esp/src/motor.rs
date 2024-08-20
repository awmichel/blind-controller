use core::ops::RangeInclusive;

use esp_hal::{
    clock::Clocks,
    gpio::{Event, Input, InputPin, Level, Output, OutputPin, Pull},
    mcpwm::operator::PwmPin,
    peripheral::Peripheral,
    peripherals::MCPWM0,
};
use esp_hal::{
    mcpwm::{
        operator::{PwmActions, PwmPinConfig, PwmUpdateMethod},
        timer::PwmWorkingMode,
        McPwm, PeripheralClockConfig,
    },
    prelude::*,
};

type MotorPWMPinA<'d, Pin> = PwmPin<'d, Pin, MCPWM0, 0, true>;

pub struct Motor<'d, In1Pin, In2Pin, PwmPin>
where
    In1Pin: OutputPin + 'd,
    In2Pin: OutputPin + 'd,
    PwmPin: OutputPin + 'd,
{
    in1: Output<'d, In1Pin>,
    in2: Output<'d, In2Pin>,
    pwm: MotorPWMPinA<'d, PwmPin>,
}

impl<'d, In1Pin: OutputPin, In2Pin: OutputPin, PwmPin: OutputPin>
    Motor<'d, In1Pin, In2Pin, PwmPin>
{
    const PERIOD: u16 = 256;

    pub fn take(
        in1: impl Peripheral<P = In1Pin> + 'd,
        in2: impl Peripheral<P = In2Pin> + 'd,
        pwm: impl Peripheral<P = PwmPin> + 'd,
        mcpwm: MCPWM0,
        clocks: &Clocks,
    ) -> Motor<'d, In1Pin, In2Pin, PwmPin> {
        let clock_cfg = PeripheralClockConfig::with_frequency(clocks, 32.MHz()).unwrap();
        let mut mcpwm = McPwm::new(mcpwm, clock_cfg);
        mcpwm.operator0.set_timer(&mcpwm.timer0);

        let pwm = mcpwm.operator0.with_pin_a(
            pwm,
            PwmPinConfig::new(PwmActions::UP_ACTIVE_HIGH, PwmUpdateMethod::SYNC_IMMEDIATLY),
        );

        let timer_clock_cfg = clock_cfg
            .timer_clock_with_frequency(Self::PERIOD, PwmWorkingMode::Increase, 10.kHz())
            .unwrap();
        mcpwm.timer0.start(timer_clock_cfg);

        Motor {
            in1: Output::new(in1, Level::Low),
            in2: Output::new(in2, Level::Low),
            pwm,
        }
    }

    pub fn with_encoder<PinA: InputPin, PinB: InputPin>(
        &self,
        pina: impl Peripheral<P = PinA> + 'd,
        pinb: impl Peripheral<P = PinB> + 'd,
    ) -> MotorEncoder<'d, PinA, PinB> {
        MotorEncoder::new(pina, pinb)
    }

    pub fn stop(&mut self) {
        self.in1.set_low();
        self.in2.set_low();
        self.pwm.set_timestamp(0);
    }

    pub fn drive(&mut self, speed: f32) {
        if !RangeInclusive::new(-1., 1.).contains(&speed) {
            panic!("speed must be between -1 and 1 inclusive.");
        }

        if speed.is_sign_positive() {
            self.in1.set_high();
            self.in2.set_low();
            self.pwm.set_timestamp((Self::PERIOD as f32 * speed) as u16);
        } else if speed.is_sign_negative() {
            self.in1.set_low();
            self.in2.set_high();
            self.pwm
                .set_timestamp((Self::PERIOD as f32 * speed * -1.) as u16);
        } else {
            self.stop();
        }
    }
}

pub struct MotorEncoder<'d, PinA: InputPin, PinB: InputPin> {
    enca: Input<'d, PinA>,
    encb: Input<'d, PinB>,
    pub counter: i32,
}

impl<'d, PinA: InputPin, PinB: InputPin> MotorEncoder<'d, PinA, PinB> {
    fn new(
        pina: impl Peripheral<P = PinA> + 'd,
        pinb: impl Peripheral<P = PinB> + 'd,
    ) -> MotorEncoder<'d, PinA, PinB> {
        let mut enca = Input::new(pina, Pull::None);
        enca.listen(Event::RisingEdge);
        MotorEncoder {
            enca,
            encb: Input::new(pinb, Pull::None),
            counter: 0,
        }
    }

    pub fn update(&mut self) -> bool {
        if self.enca.is_interrupt_set() {
            self.counter += if self.encb.get_level() == Level::High {
                -1
            } else {
                1
            };
            self.enca.clear_interrupt();

            return true;
        }

        false
    }
}
