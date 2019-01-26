use clap::App;

fn main() {

    let _ = App::new("Chouette Mail")
        .version("0.0.0")
        .author("Thomas Forgione <thomas@forgione.fr>")
        .about("A cool webmail written in Rust and Elm")
        .get_matches();

    chouette::start();

}
