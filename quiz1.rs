use std::boxed::Box;

trait Drawable {
    fn draw(&self);
}

struct Button {
    pub size: i32,
}

struct Scrollbar {
    pub location: i32,
}

impl Drawable for Button {
    fn draw(&self) {
        // dbg!("Drawing a button");
    }
}

impl Drawable for Scrollbar {
    fn draw(&self) {
        // dbg!("Drawing a scrollbar");
    }
}

struct DynScreen {
    drawables: Vec<Box<Drawable>>,
}

impl DynScreen {
    pub fn new() -> Self {
        DynScreen {
            drawables: Vec::new(),
        }
    }

    pub fn add(&mut self, c: Box<Drawable>) {
        self.drawables.push(c);
    }

    pub fn draw(&self) {
        for c in self.drawables.iter() {
            c.draw();
        }
    }
}

struct StaticScreen<T: Drawable> {
    drawables: Vec<T>,
}

impl<T: Drawable> StaticScreen<T> {
    pub fn new() -> Self {
        StaticScreen {
            drawables: Vec::new(),
        }
    }

    pub fn add(&mut self, c: T) {
        self.drawables.push(c);
    }

    pub fn draw(&self) {
        for c in self.drawables.iter() {
            c.draw();
        }
    }
}

// struct StaticScreen2 {
//     drawables: Vec<Drawable>,
// }

// impl StaticScreen2 {
//     pub fn new() -> Self {
//         StaticScreen2 {
//             drawables: Vec::new(),
//         }
//     }

//     pub fn add(&mut self, c: Drawable) {
//         self.drawables.push(c);
//     }

//     pub fn draw(&self) {
//         for c in self.drawables.iter() {
//             c.draw();
//         }
//     }
// }

fn main() {
    // Why this works?
    let mut screen = DynScreen::new();
    let button = Button { size: 32 };
    let sb = Scrollbar { location: 64 };
    screen.add(Box::new(button));
    screen.add(Box::new(sb));
    screen.draw();

    // But this doesn't?
    // let mut screen = StaticScreen::new();
    // let button = Button { size: 32 };
    // let sb = Scrollbar { location: 64 };
    // screen.add(button);
    // screen.add(sb);
    // screen.draw();

    // static screen 2
    // let mut screen = StaticScreen2::new();
    // let button = Button { size: 32 };
    // let sb = Scrollbar { location: 64 };
    // screen.add(button);
    // screen.add(sb);
    // screen.draw();
}
