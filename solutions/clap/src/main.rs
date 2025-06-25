use clap::{App, load_yaml};
use std::env;
use std::process;

fn main() {
    // Arguments
    let yaml = load_yaml!("cli.yaml");
    let m = App::from_yaml(yaml).get_matches();

    match m.value_of("argument1") {}

    // Configuration
    println!("{}", dotenv!("PORT"));

    // Environment Valiables
    let key = "HOME";

    match env::var_os(key) {
        Some(val) => println!("{}: {:?}", key, val),
        None => println!("{} is not defined in the environment", key),
    }

    // Error Handling
    // -- Panic
    panic!("this is panic");

    // -- Result
    enum MyErr {
        Reason1,
        Reason2,
    };

    fn foo() -> Result<(), MyErr> {
        match bar {
            Some(_) => {}
            None => Err(MyErr::Reason1),
        }
    }
    fn hoo() {
        match foo() {
            Ok(_) => reply(),
            Err(e) => println!(e),
        }
    }

    // -- Error Message
    enum MyErr {
        Reason1(String),
        Reason2(String,u32),
    }
    impl fmt::Display for MyErr {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            match *self {
                MyErr::Reason1(ref s) => write!(f, "`{}` is the error", s),
                MyErr::Reason2(ref s, ref num) => write!("`{}` and `{}` are error", s, num),
            }
        }
    }

    Err(e) => println!("{}", e);

    // Exit Code
    process::exit(1);
}
