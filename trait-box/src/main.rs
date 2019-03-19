use std::boxed::Box;
use std::thread::spawn;

trait BaseTrait {}

trait Printer {
    fn print(&self);
}

impl BaseTrait for Printer{}

struct PrintImpl;
impl Printer for PrintImpl {
    fn print(&self) {
        println!("works")
    }
}

struct PrinterHolder {
    printer: TraitBox<Printer>,
}

type TraitBox<T: BaseTrait> = Box<T + Send + Sync>;

impl PrinterHolder {
    fn print(&self) {
        self.printer.print();
    }
}

fn main() {
    println!("Hello, world!");
    let printer = PrintImpl;
    let holder = PrinterHolder {
        printer: Box::new(printer),
    };

    let handle = spawn(move || {
        holder.print();
    });

    handle.join();
}
