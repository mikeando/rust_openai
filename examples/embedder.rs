use std::env;

use rust_openai::embedding;
use tokio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();
    let openai_api_key = env::var("OPENAI_API_KEY").unwrap();
    eprintln!("{:?}", openai_api_key);

    let text = "Input text to embed, encoded as a string or array of tokens. To embed multiple inputs in a single request, pass an array of strings or array of token arrays. The input must not exceed the max input tokens for the model";
    let r = embedding::make_uncached_embedding_request(text, &openai_api_key).await;

    println!("{:#?}", r);

    Ok(())
}
