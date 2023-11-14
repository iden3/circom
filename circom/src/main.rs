use ansi_term::Colour;

fn main() {
    let result = start();
    if result.is_err() {
        eprintln!("{}", Colour::Red.paint("previous errors were found"));
        std::process::exit(1);
    } else {
        println!("{}", Colour::Green.paint("Everything went okay, circom safe"));
        //std::process::exit(0);
    }
}
fn start() -> Result<(), ()> {
    use circom::input_user::Input;
    use circom::circom_compiler;

    let user_input = Input::new()?;
    circom_compiler(user_input)
}
