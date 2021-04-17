#![no_std]
#![no_main]

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
    prelude::*,
};

use cortex_m::interrupt;

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

        let mut rcc = device
            .RCC
            .configure()
            .hsi48()
            .enable_crs(device.CRS)
            .sysclk(48.mhz())
            .pclk(24.mhz())
            .freeze(&mut device.FLASH);

        let gpioa = device.GPIOA.split(&mut rcc);
        let (key_a, key_b, rot_a, rot_b, _, tach1) = interrupt::free(|cs| {
            (
                gpioa.pa3.into_pull_up_input(cs),                            // key_a
                gpioa.pa4.into_pull_up_input(cs),                            // key_b
                gpioa.pa6.into_pull_up_input(cs),                            // rot_a
                gpioa.pa5.into_pull_up_input(cs),                            // rot_b
                gpioa.pa8.into_alternate_af2(cs).internal_pull_up(cs, true), // pwm3, TIM1_CH1
                gpioa.pa15.into_floating_input(cs),                          // tach1
            )
        });

        let gpiob = device.GPIOB.split(&mut rcc);
        let (vsense, _, _, _, tach2, tach3, tach4) = interrupt::free(|cs| {
            (
                gpiob.pb0.into_analog(cs),                                   // vsense
                gpiob.pb3.into_alternate_af2(cs).internal_pull_up(cs, true), // pwm1, TIM2_CH2
                gpiob.pb5.into_alternate_af1(cs).internal_pull_up(cs, true), // pwm2, TIM3_CH2
                gpiob.pb4.into_alternate_af1(cs).internal_pull_up(cs, true), // pwm4, TIM3_CH1
                gpiob.pb7.into_floating_input(cs),                           // tach2
                gpiob.pb8.into_floating_input(cs),                           // tach3
                gpiob.pb6.into_floating_input(cs),                           // tach4
            )
        });

        // Enable external interrupts
        let syscfg = device.SYSCFG;
        syscfg.exticr1.write(|w| w.exti3().pa3()); // key_a

        #[rustfmt::skip]
        syscfg.exticr2.write(|w| {
            w
                .exti4().pa4() // key_b
                .exti6().pb6() // tach4
                .exti7().pb7() // tach2
        });

        syscfg.exticr3.write(|w| w.exti8().pb8()); // tach3
        syscfg.exticr4.write(|w| w.exti15().pa15()); // tach1

        // Set interrupt mask for all the above
        let exti = device.EXTI;

        #[rustfmt::skip]
        exti.imr.write(|w| {
            w
                .mr3().set_bit() // key_a
                .mr4().set_bit() // key_b
                .mr6().set_bit() // tach4
                .mr7().set_bit() // tach2
                .mr8().set_bit() // tach3
                .mr15().set_bit() // tach1
        });

        // Set interrupt falling edge trigger
        #[rustfmt::skip]
        exti.ftsr.write(|w| {
            w
                .tr3().set_bit() // key_a
                .tr4().set_bit() // key_b
                .tr6().set_bit() // tach4
                .tr7().set_bit() // tach2
                .tr8().set_bit() // tach3
                .tr15().set_bit() // tach1
        });

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
