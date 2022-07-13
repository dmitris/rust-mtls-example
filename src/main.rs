use std::env;

use tokio::fs::File;
use tokio::io::AsyncReadExt;

use reqwest::tls::Identity;
use warp::Filter;

const TLS_KEY_FILE : &str = "ca/client_0.pem";

async fn run_server() {
    let routes = warp::any().map(|| "Hello, mTLS World!");

    warp::serve(routes)
        .tls()
        .key_path("ca/localhost.key")
        .cert_path("ca/localhost.bundle.crt")
        // .client_auth_required_path("ca/second_ca.crt")
        .run(([0, 0, 0, 0], 3030))
        .await
}

async fn run_client() -> Result<(), reqwest::Error> {
    // let server_ca_file_loc = CA_CERT_FILE;
    let server_ca_file_loc = "ca/ca.crt";
    let mut buf = Vec::new();
    File::open(server_ca_file_loc)
        .await
        .unwrap()
        .read_to_end(&mut buf)
        .await
        .unwrap();
    let cacert = reqwest::Certificate::from_pem(&buf)?;

    // #[cfg(feature = "rustls-tls")]
    async fn get_identity() -> Identity {
        // panic!("I don't know why 'Identity' with rustls-tls does not work.");
        let client_pem_file_loc = TLS_KEY_FILE;
        let mut buf = Vec::new();
        match File::open(client_pem_file_loc)
            .await
            .unwrap()
            .read_to_end(&mut buf)
            .await {
                Ok(_) => (),
                Err(e) => panic!("{}", e),
            }
        reqwest::Identity::from_pem(&buf).unwrap()
    }

    let identity = get_identity().await;

    let client = reqwest::Client::builder()
        .tls_built_in_root_certs(false)
        .add_root_certificate(cacert)
        // .danger_accept_invalid_certs(true) // uncommenting this fixes the issue, but... not a safe thing to do
        .identity(identity)
        .https_only(true)
        .build()?;

    let res = client.get("https://localhost:3030").send().await.unwrap();
    println!("Received:");
    println!("{:?}", res.text().await.unwrap());

    Ok(())
}

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    if args[1] == "server" {
        let server = run_server();
        server.await;
    } else if args[1] == "client" {
        let client = run_client();
        client.await.unwrap();
    };
}