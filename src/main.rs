#![no_std]
#![no_main]

use cortex_m_semihosting::hprintln;
use panic_semihosting as _;
use rtic::app;

use stm32f0xx_hal::{
    gpio::{
        gpioa::{PA15, PA3, PA4, PA5, PA6, PA8},
        gpiob::{PB0, PB6, PB7, PB8},
        Alternate, Analog, Input, PullUp, AF2,
    },
    pac,
    prelude::*,
    time::{Hertz, KiloHertz},
    timers::Timer,
    usb,
};

use cortex_m::interrupt::free as disable_interrupts;

#[app(device = stm32f0xx_hal::pac, peripherals = true)]
const APP: () = {
    struct Resources {
        exti: pac::EXTI,
        vsense: PB0<Analog>,

        key_a: PA3<Input<PullUp>>,
        key_b: PA4<Input<PullUp>>,
        rot_a: PA6<Input<PullUp>>,
        rot_b: PA5<Input<PullUp>>,

        pwm3: PA8<Alternate<AF2>>,

        tach1: PA15<Input<PullUp>>,
        tach2: PB7<Input<PullUp>>,
        tach3: PB8<Input<PullUp>>,
        tach4: PB6<Input<PullUp>>,

        #[init([0;4])]
        tachs: [u32; 4],
    }

    #[init]
    fn init(context: init::Context) -> init::LateResources {
        // RTT handler
        // rtt_init_print!();

        // Alias peripherals
        let mut device: pac::Peripherals = context.device;

        // This enables clock for SYSCFG and remaps USB pins to PA9 and PA10.
        usb::remap_pins(&mut device.RCC, &mut device.SYSCFG);

        hprintln!("Initializing peripherals").ok();
        let mut rcc = device
            .RCC
            .configure()
            .usbsrc(stm32f0xx_hal::rcc::USBClockSource::HSI48)
            .hsi48()
            .enable_crs(device.CRS)
            .sysclk(48.mhz())
            .pclk(24.mhz())
            .freeze(&mut device.FLASH);

        let gpioa = device.GPIOA.split(&mut rcc);
        let (key_a, key_b, rot_a, rot_b, pwm3, tach1) = disable_interrupts(|cs| {
            (
                gpioa.pa3.into_pull_up_input(cs),                            // key_a
                gpioa.pa4.into_pull_up_input(cs),                            // key_b
                gpioa.pa6.into_pull_up_input(cs),                            // rot_a
                gpioa.pa5.into_pull_up_input(cs),                            // rot_b
                gpioa.pa8.into_open_drain_output(cs).into_alternate_af2(cs), // pwm3
                gpioa.pa15.into_pull_up_input(cs),                           // tach1
            )
        });

        let gpiob = device.GPIOB.split(&mut rcc);
        let (vsense, tach2, tach3, tach4) = disable_interrupts(|cs| {
            (
                gpiob.pb0.into_analog(cs),        // vsense
                gpiob.pb7.into_pull_up_input(cs), // tach2
                gpiob.pb8.into_pull_up_input(cs), // tach3
                gpiob.pb6.into_pull_up_input(cs), // tach4
            )
        });

        // Enable external interrupts
        let syscfg = device.SYSCFG;
        syscfg.exticr1.write(|w| w.exti3().pa3()); // key_a
        syscfg.exticr2.write(|w| {
            w.exti4().pa4(); // key_b
            w.exti6().pb6(); // tach4
            w.exti7().pb7() // tach2
        });
        syscfg.exticr3.write(|w| w.exti8().pb8()); // tach3
        syscfg.exticr4.write(|w| w.exti15().pa15()); // tach1

        // Set interrupt mask for all the above
        let exti = device.EXTI;
        exti.imr.write(|w| {
            w.mr3().set_bit(); // key_a
            w.mr4().set_bit(); // key_b
            w.mr6().set_bit(); // tach4
            w.mr7().set_bit(); // tach2
            w.mr8().set_bit(); // tach3
            w.mr15().set_bit() // tach1
        });

        // Set interrupt falling edge trigger
        exti.ftsr.write(|w| {
            w.tr3().set_bit(); // key_a
            w.tr4().set_bit(); // key_b
            w.tr6().set_bit(); // tach4
            w.tr7().set_bit(); // tach2
            w.tr8().set_bit(); // tach3
            w.tr15().set_bit() // tach1
        });

        // Setup PWM timers
        let tim1 = device.TIM1;
        tim1.psc.write(|w| w.psc().bits(0));
        tim1.arr.write(|w| w.arr().bits(1919));
        tim1.ccr1.write(|w| w.ccr().bits(800));
        tim1.ccmr1_output()
            .write(|w| w.oc1pe().set_bit().oc1m().pwm_mode1());
        tim1.ccer.write(|w| w.cc1e().set_bit());
        tim1.bdtr.write(|w| w.moe().set_bit());
        tim1.egr.write(|w| w.ug().set_bit());

        hprintln!("Done").ok();
        init::LateResources {
            exti,
            vsense,
            key_a,
            key_b,
            rot_a,
            rot_b,
            pwm3,
            tach1,
            tach2,
            tach3,
            tach4,
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
