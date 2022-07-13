use std::env;

use tokio::fs::File;
use tokio::io::AsyncReadExt;

use reqwest::tls::Identity;
use warp::Filter;

// use std::io::Error;
// use std::path::Path;

const CA_CERT_FILE : &str = "/Users/dsavints/.athenz/ca.cert.pem";
const TLS_KEY_FILE : &str = "/Users/dsavints/.athenz/keycert";


async fn run_server() {
    let routes = warp::any().map(|| "Hello, mTLS World!");

    warp::serve(routes)
        .tls()
        .key_path("ca/localhost.key")
        .cert_path("ca/localhost.bundle.crt")
        .client_auth_required_path("ca/second_ca.crt")
        .run(([0, 0, 0, 0], 3030))
        .await
}

async fn run_client() -> Result<(), reqwest::Error> {
    let server_ca_file_loc = CA_CERT_FILE;
    let mut buf = Vec::new();
    File::open(server_ca_file_loc)
        .await
        .unwrap()
        .read_to_end(&mut buf)
        .await
        .unwrap();
    let cacert = reqwest::Certificate::from_pem(&buf)?;

    // #[cfg(feature = "native-tls")]
    // async fn get_identity() -> Identity {
    //     let client_p12_file_loc = CA_CERT_FILE;
    //     let mut buf = Vec::new();
    //     File::open(client_p12_file_loc)
    //         .await
    //         .unwrap()
    //         .read_to_end(&mut buf)
    //         .await
    //         .unwrap();
    //     reqwest::Identity::from_pkcs12_der(&buf, "123456").unwrap()
    // }

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
                Err(e) => panic!("{e}"),
            }
        reqwest::Identity::from_pem(&buf).unwrap()
    }

    let identity = get_identity().await;

    let client = reqwest::Client::builder()
        .tls_built_in_root_certs(false)
        .add_root_certificate(cacert)
        .identity(identity)
        .https_only(true)
        .build()?;

    let res = client.get("https://localhost:3030").send().await.unwrap();
    println!("Received:");
    println!("{:?}", res.text().await.unwrap());

    Ok(())
}

// async fn check_path() -> Result<String, std::io::Error> {
//     // Create a path to the desired file
//     let path = Path::new(CA_CERT_FILE);
//     let display = path.display();

//     // Open the path in read-only mode, returns `io::Result<File>`
//     let mut file = match File::open(&path).await {
//         Err(why) => panic!("couldn't open {}: {}", display, why),
//         Ok(file) => file,
//     };

//     // Read the file contents into a string, returns `io::Result<usize>`
//     let mut s = String::new();
//     match file.read_to_string(&mut s).await {
//         Err(e) => Err(e),
//         Ok(_) => Ok(s),
//     }

//     // `file` goes out of scope, and the file gets closed
// }

#[tokio::main]
async fn main() {
    // match check_path().await {
    //     Err(why) => panic!("couldn't read file: {}", why),
    //     Ok(s) => print!("CA file contains:\n{}", s),
    // }
    let args: Vec<String> = env::args().collect();
    if args[1] == "server" {
        let server = run_server();
        server.await;
    } else if args[1] == "client" {
        let client = run_client();
        client.await.unwrap();
    };
}