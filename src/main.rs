#![no_std]
#![no_main]

mod peripherals;
mod pwm;

use cortex_m_semihosting::hprintln;
use panic_semihosting as _;
use rtic::app;

use stm32f0xx_hal::{
    gpio::{
        gpioa::{PA15, PA3, PA4, PA5, PA6},
        gpiob::{PB0, PB6, PB7, PB8},
        Analog, Floating, Input, PullUp,
    },
    pac,
};

#[app(device = stm32f0xx_hal::pac, peripherals = true)]
const APP: () = {
    struct Resources {
        exti: pac::EXTI,
        vsense: PB0<Analog>,

        key_a: PA3<Input<PullUp>>,
        key_b: PA4<Input<PullUp>>,
        rot_a: PA6<Input<PullUp>>,
        rot_b: PA5<Input<PullUp>>,

        tach1: PA15<Input<Floating>>,
        tach2: PB7<Input<Floating>>,
        tach3: PB8<Input<Floating>>,
        tach4: PB6<Input<Floating>>,

        pwm: pwm::Pwm,

        #[init([0; 4])]
        tachs: [u32; 4],
    }

    #[init]
    fn init(context: init::Context) -> init::LateResources {
        let mut device: pac::Peripherals = context.device;

        let mut rcc = peripherals::configure_rcc(device.RCC, device.CRS, &mut device.FLASH);

        let (key_a, key_b, rot_a, rot_b, tach1) =
            peripherals::configure_gpioa(device.GPIOA, &mut rcc);

        let (vsense, tach2, tach3, tach4) = peripherals::configure_gpiob(device.GPIOB, &mut rcc);

        let exti = peripherals::configure_exti(device.SYSCFG, device.EXTI);

        let pwm = pwm::Pwm::new(device.TIM1, device.TIM2, device.TIM3, &mut rcc);
        pwm.set_duty(pwm::PwmChannel::Pwm1, 20);
        pwm.set_duty(pwm::PwmChannel::Pwm2, 40);
        pwm.set_duty(pwm::PwmChannel::Pwm3, 60);
        pwm.set_duty(pwm::PwmChannel::Pwm4, 80);

        init::LateResources {
            exti,
            vsense,

            key_a,
            key_b,
            rot_a,
            rot_b,

            tach1,
            tach2,
            tach3,
            tach4,

            pwm,
        }
    }

    #[idle]
    fn idle(_: idle::Context) -> ! {
        loop {
            cortex_m::asm::nop();
            cortex_m::asm::wfi();
        }
    }

    #[task(binds = EXTI2_3, resources = [exti])]
    fn exti2_3_interrupt(ctx: exti2_3_interrupt::Context) {
        let pr = &ctx.resources.exti.pr;
        let bits = pr.read();

        if bits.pif3().bit() {
            hprintln!("EXTI2_3 key_a").ok();
            pr.write(|w| w.pif3().set_bit());
        }
    }

    #[task(binds = EXTI4_15, resources = [exti, tachs])]
    fn exti_4_15_interrupt(ctx: exti_4_15_interrupt::Context) {
        let pr = &ctx.resources.exti.pr;
        let bits = pr.read();

        if bits.pif4().bit() {
            hprintln!("EXTI4_15 key_b").ok();
            pr.write(|w| w.pif4().set_bit());
        }

        if bits.pif6().bit() {
            ctx.resources.tachs[3] += 1;
            pr.write(|w| w.pif6().set_bit());
        }

        if bits.pif7().bit() {
            ctx.resources.tachs[1] += 1;
            pr.write(|w| w.pif7().set_bit());
        }

        if bits.pif8().bit() {
            ctx.resources.tachs[2] += 1;
            pr.write(|w| w.pif8().set_bit());
        }

        if bits.pif15().bit() {
            ctx.resources.tachs[0] += 1;
            pr.write(|w| w.pif15().set_bit());
        }
    }
};
