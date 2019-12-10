use intcode::Signal;
use std::{
    convert::TryFrom,
    io::{self, Read, Write},
    sync::mpsc,
};

fn level1(original_vm: &intcode::VM) -> aoc::Result<u32> {
    let mut phase_settings = [0, 1, 2, 3, 4];
    let heap = permutohedron::Heap::new(&mut phase_settings);
    let mut thruster_signal = 0;

    for permutation in heap {
        let mut amplified_input = 0;
        for &phase in &permutation {
            let mut vm = intcode::VM::with_mem(original_vm.mem());
            let (input, output) = vm.setup_io();
            input.send(Signal::Value(phase)).unwrap();
            input.send(Signal::Value(amplified_input)).unwrap();
            vm.run()?;
            amplified_input = match output.recv().unwrap() {
                Signal::Value(x) => x,
                Signal::Halting => {
                    return aoc::err!("Amplifier halted before giving output")
                }
            };
        }

        let output = u32::try_from(amplified_input)?;
        thruster_signal = u32::max(thruster_signal, output);
    }

    Ok(thruster_signal)
}

fn level2(original_vm: &intcode::VM) -> aoc::Result<u32> {
    let mut phase_settings = [5, 6, 7, 8, 9];
    let mut thruster_signal = 0;
    let heap = permutohedron::Heap::new(&mut phase_settings);
    for permutation in heap {
        let (init_tx, init_rx) = mpsc::channel();
        let mut tx = init_tx.clone();
        let mut rx = init_rx;
        for &phase in &permutation {
            let mut vm = intcode::VM::with_mem(original_vm.mem());
            let (new_tx, new_rx) = mpsc::channel();
            let cloned_tx = new_tx.clone();
            vm.connect_io(rx, cloned_tx)?;
            rayon::spawn(move || vm.run().unwrap());
            tx.send(Signal::Value(phase)).unwrap();
            tx = new_tx;
            rx = new_rx;
        }

        init_tx.send(Signal::Value(0)).unwrap();
        let mut amplified_input = 0;
        for sig in rx.iter() {
            let _ = init_tx.send(sig);
            match sig {
                Signal::Value(x) => amplified_input = x,
                Signal::Halting => break,
            }
        }

        let result = u32::try_from(amplified_input)?;
        thruster_signal = u32::max(result, thruster_signal);
    }

    log::info!("{}", rayon::current_num_threads());
    Ok(thruster_signal)
}

fn solve() -> aoc::Result<()> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;
    let vm = intcode::VM::with_program(&input)?;

    let some = level1(&vm)?;
    writeln!(io::stderr(), "level 1: {}", some)?;

    let thing = level2(&vm)?;
    writeln!(io::stderr(), "level 2: {}", thing)?;

    // stdout is used to submit solutions
    writeln!(io::stdout(), "{}", thing)?;
    Ok(())
}

fn main() -> aoc::Result<()> {
    env_logger::init();
    if let Err(e) = solve() {
        let stderr = io::stderr();
        let mut w = stderr.lock();
        writeln!(w, "Error: {}", e)?;
        while let Some(e) = e.source() {
            writeln!(w, "\t{}", e)?;
        }

        std::process::exit(-1)
    }

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    const INPUT: &str = include_str!("../input.txt");

    #[test_log::new]
    fn sanity() -> aoc::Result<()> {
        let vm = intcode::VM::with_program(INPUT)?;
        let result = level1(&vm)?;
        assert_eq!(result, 18812);

        let result = level2(&vm)?;
        assert_eq!(result, 25534964);
        Ok(())
    }
}
