use shadow_rs::ShadowBuilder;

fn main() {
    ShadowBuilder::builder().build().unwrap();

    minijinja_embed::embed_templates!("templates");
}
