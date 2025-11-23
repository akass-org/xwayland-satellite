use vergen_gitcl::{Emitter, GitclBuilder};

fn main() -> Result<(), anyhow::Error> {
    // let builder = GitclBuilder::default().describe(true, true, None).build()?;
    let builder = GitclBuilder::default().sha(true).build()?;
    Emitter::default().add_instructions(&builder)?.emit()
}
