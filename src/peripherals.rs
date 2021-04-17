use cortex_m::interrupt;
use stm32f0xx_hal::{
    gpio::{
        gpioa::{PA15, PA3, PA4, PA5, PA6},
        gpiob::{PB0, PB6, PB7, PB8},
        Analog, Floating, Input, PullUp,
    },
    pac,
    prelude::*,
    rcc::Rcc,
};

pub fn configure_rcc(rcc: pac::RCC, crs: pac::CRS, flash: &mut pac::FLASH) -> Rcc {
    rcc.configure()
        .hsi48()
        .enable_crs(crs)
        .sysclk(48.mhz())
        .pclk(24.mhz())
        .freeze(flash)
}

pub fn configure_gpioa(
    gpioa: pac::GPIOA,
    rcc: &mut Rcc,
) -> (
    PA3<Input<PullUp>>,
    PA4<Input<PullUp>>,
    PA6<Input<PullUp>>,
    PA5<Input<PullUp>>,
    PA15<Input<Floating>>,
) {
    let gpioa = gpioa.split(rcc);

    interrupt::free(|cs| {
        gpioa.pa8.into_alternate_af2(cs).internal_pull_up(cs, true); // pwm3 (TIM1_CH1)

        (
            gpioa.pa3.into_pull_up_input(cs),   // key_a
            gpioa.pa4.into_pull_up_input(cs),   // key_b
            gpioa.pa6.into_pull_up_input(cs),   // rot_a
            gpioa.pa5.into_pull_up_input(cs),   // rot_b
            gpioa.pa15.into_floating_input(cs), // tach1
        )
    })
}

pub fn configure_gpiob(
    gpiob: pac::GPIOB,
    rcc: &mut Rcc,
) -> (
    PB0<Analog>,
    PB7<Input<Floating>>,
    PB8<Input<Floating>>,
    PB6<Input<Floating>>,
) {
    let gpiob = gpiob.split(rcc);

    interrupt::free(|cs| {
        gpiob.pb3.into_alternate_af2(cs).internal_pull_up(cs, true); // pwm1, TIM2_CH2
        gpiob.pb5.into_alternate_af1(cs).internal_pull_up(cs, true); // pwm2, TIM3_CH2
        gpiob.pb4.into_alternate_af1(cs).internal_pull_up(cs, true); // pwm4, TIM3_CH1

        (
            gpiob.pb0.into_analog(cs),         // vsense
            gpiob.pb7.into_floating_input(cs), // tach2
            gpiob.pb8.into_floating_input(cs), // tach3
            gpiob.pb6.into_floating_input(cs), // tach4
        )
    })
}

pub fn configure_exti(syscfg: pac::SYSCFG, exti: pac::EXTI) -> pac::EXTI {
    // Enable external interrupts
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

    exti
}
