use api::openapi::ApiDoc;
use utoipa::OpenApi;
use std::fs;
use std::path::PathBuf;

fn main() {
    let manifest_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set"));
    let workspace_root = manifest_dir.parent().unwrap().parent().unwrap();

    let docs_gen_dir = workspace_root.join("docs").join("generated");
    fs::create_dir_all(&docs_gen_dir).expect("Failed to create docs/generated directory");

    // Generate OpenAPI json
    let openapi_json = ApiDoc::openapi().to_pretty_json().expect("Failed to serialize OpenAPI to JSON");
    fs::write(docs_gen_dir.join("openapi.json"), openapi_json).expect("Failed to write openapi.json");

    // Generate GraphQL schema
    let schema = async_graphql::Schema::build(
        api::graphql::Query,
        api::graphql::Mutation,
        async_graphql::EmptySubscription,
    )
    .finish();
    
    fs::write(docs_gen_dir.join("schema.graphql"), schema.sdl()).expect("Failed to write schema.graphql");

    println!("Successfully generated API docs in docs/generated/");
}
