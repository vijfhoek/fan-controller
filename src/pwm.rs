use stm32f0xx_hal::{pac, rcc::Rcc};

pub enum PwmChannel {
    Pwm1,
    Pwm2,
    Pwm3,
    Pwm4,
}

pub struct Pwm {
    tim1: pac::TIM1,
    tim2: pac::TIM2,
    tim3: pac::TIM3,
}

impl Pwm {
    pub fn new(tim1: pac::TIM1, tim2: pac::TIM2, tim3: pac::TIM3, rcc: &mut Rcc) -> Self {
        Self::setup_tim1(&tim1, rcc);
        Self::setup_tim2(&tim2, rcc);
        Self::setup_tim3(&tim3, rcc);
        Self { tim1, tim2, tim3 }
    }

    fn setup_tim1(tim1: &pac::TIM1, rcc: &mut Rcc) {
        // Enable peripheral clock
        rcc.regs.apb2enr.modify(|_, w| w.tim1en().set_bit());
        rcc.regs.apb2rstr.modify(|_, w| w.tim1rst().set_bit());
        rcc.regs.apb2rstr.modify(|_, w| w.tim1rst().clear_bit());

        // Setup peripheral
        #[rustfmt::skip]
        tim1.ccmr1_output().modify(|_, w| {
            w
                // Output high when TIM1_CNT < TIM1_CCR1
                .oc1m().pwm_mode1()
                // CC1 channel is configured as output
                .cc1s().output()
        });

        // Capture/Compare 1 Output Enable
        tim1.ccer.write(|w| w.cc1e().set_bit());

        // Set the Main Output Enable bit
        tim1.bdtr.write(|w| w.moe().set_bit());

        // Enable update generation
        tim1.egr.write(|w| w.ug().set_bit());

        // Configure pre-scaler for 48 MHz / 1 = 48 MHz
        tim1.psc.write(|w| w.psc().bits(0));
        // Configure auto-reload register for 1920 / 1 MHz = 25 kHz
        tim1.arr.write(|w| w.arr().bits(1920 - 1));
        // Set half duty cycle (half of ARR)
        tim1.ccr1.write(|w| w.ccr().bits(1920 / 2));

        // Enable the counter
        tim1.cr1.modify(|_, w| w.cen().set_bit());
    }

    fn setup_tim2(tim2: &pac::TIM2, rcc: &mut Rcc) {
        // Enable peripheral clock
        rcc.regs.apb1enr.modify(|_, w| w.tim2en().set_bit());
        rcc.regs.apb1rstr.modify(|_, w| w.tim2rst().set_bit());
        rcc.regs.apb1rstr.modify(|_, w| w.tim2rst().clear_bit());

        // Setup peripheral
        #[rustfmt::skip]
        tim2.ccmr1_output().write(|w| {
            w
                // Output high when TIM2_CNT < TIM2_CCR2
                .oc2m().pwm_mode1()
                // CC2 channel is configured as output
                .cc2s().output()
        });

        // Capture/Compare 2 Output Enable
        tim2.ccer.write(|w| w.cc2e().set_bit());

        // TODO No Main Output Enable?

        // Enable update generation
        tim2.egr.write(|w| w.ug().set_bit());

        // Configure pre-scaler for 48 MHz / 1 = 48 MHz
        tim2.psc.write(|w| w.psc().bits(0));
        // Configure auto-reload register for 1920 / 1 MHz = 25 kHz
        tim2.arr.write(|w| w.arr().bits(1920 - 1));
        // Set half duty cycle (half of ARR)
        tim2.ccr2.write(|w| w.ccr().bits(1920 / 2));

        // Enable the counter
        tim2.cr1.write(|w| w.cen().set_bit());
    }

    fn setup_tim3(tim3: &pac::TIM3, rcc: &mut Rcc) {
        // Enable peripheral clock
        rcc.regs.apb1enr.modify(|_, w| w.tim3en().set_bit());
        rcc.regs.apb1rstr.modify(|_, w| w.tim3rst().set_bit());
        rcc.regs.apb1rstr.modify(|_, w| w.tim3rst().clear_bit());

        // Setup peripheral
        #[rustfmt::skip]
        tim3.ccmr1_output().write(|w| {
            w
                // Output high when TIM3_CNT < TIM3_CCR1
                .oc1m().pwm_mode1()
                // CC1 channel is configured as output
                .cc1s().output()
                // Output high when TIM3_CNT < TIM3_CCR2
                .oc2m().pwm_mode1()
                // CC2 channel is configured as output
                .cc2s().output()
        });

        // Capture/Compare 2 Output Enable
        tim3.ccer.modify(|_, w| w.cc2e().set_bit());

        // TODO No Main Output Enable?

        // Enable update generation
        tim3.egr.write(|w| w.ug().set_bit());

        // Configure pre-scaler for 48 MHz / 1 = 48 MHz
        tim3.psc.write(|w| w.psc().bits(0));
        // Configure auto-reload register for 1920 / 1 MHz = 25 kHz
        tim3.arr.write(|w| w.arr().bits(1920 - 1));
        // Set half duty cycle (half of ARR)
        tim3.ccr2.write(|w| w.ccr().bits(1920 / 2));

        // Enable the counter
        tim3.cr1.modify(|_, w| w.cen().set_bit());
    }

    pub fn set_duty(&self, channel: PwmChannel, value: u16) {
        match channel {
            PwmChannel::Pwm1 => self.tim2.ccr2.write(|w| w.ccr().bits(value as u32)),
            PwmChannel::Pwm2 => self.tim3.ccr1.write(|w| w.ccr().bits(value)),
            PwmChannel::Pwm3 => self.tim1.ccr1.write(|w| w.ccr().bits(value)),
            PwmChannel::Pwm4 => self.tim3.ccr2.write(|w| w.ccr().bits(value)),
        }
    }
}
