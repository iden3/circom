pub struct Log {
    pub no_linear: usize,
    pub no_non_linear: usize,
    pub no_labels: usize,
    pub no_wires: usize,
    pub no_public_inputs: usize,
    pub no_private_inputs: usize,
    pub no_public_outputs: usize,
    pub no_private_outputs: usize,
}

impl Log {
    pub fn new() -> Log {
        Log {
            no_linear: 0,
            no_non_linear: 0,
            no_public_inputs: 0,
            no_private_inputs: 0,
            no_public_outputs: 0,
            no_private_outputs: 0,
            no_wires: 0,
            no_labels: 0,
        }
    }

    pub fn print(log: &Log) {
        println!("non-linear constraints: {}", log.no_non_linear);
        println!("linear constraints: {}", log.no_linear);
        println!("public inputs: {}", log.no_public_inputs);
        println!("public outputs: {}", log.no_public_outputs);
        println!("private inputs: {}", log.no_private_inputs);
        println!("private outputs: {}", log.no_private_outputs);
        println!("wires: {}", log.no_wires);
        println!("labels: {}", log.no_labels);
    }
}
