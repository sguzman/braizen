use url::Url;

fn main() {
    let url = Url::parse("ws://127.0.0.1:0/ws").unwrap();
    println!("Port: {:?}", url.port());
}
