//! Dump exactly the library-resolved Core that the pinned GPU lowerer loads.
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::args().nth(1).ok_or("expected an X_eTaL file")?;
    let source = std::fs::read_to_string(&path)?;
    let loaded = xetal_program::load(&path, &source)
        .map_err(|d| std::io::Error::other(format!("{}: {}", d.code, d.message)))?;
    print!("{}", loaded.program);
    Ok(())
}
