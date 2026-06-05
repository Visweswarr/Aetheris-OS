wit_bindgen::generate!({
    world: "hello-world",
    path: "wit",
});

use exports::polymera::hello::greeter::Guest;

struct HelloComponent;

impl Guest for HelloComponent {
    fn greet(name: String) -> String {
        format!("Hello, {name}! — from a Polymera WASM component")
    }
}

export!(HelloComponent);
