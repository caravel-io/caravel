## Caravel Export usage

```
cargo new --lib your_caravel_lib
cargo add caravel_export_poc
cargo add anyhow
cargo add serde --features derive
cargo add serde_json
```

Make sure your Cargo.toml has the below entry in the [lib] section.
```
crate-type = ["cdylib"]
```

Put the following proc macros on the struct you'd like to expose to Caravel in your library.
```rust
#[caravel_resource]
#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct File {
    pub path: PathBuf,
    pub state: FileState,
    pub owner: Option<String>,
    pub group: Option<String>,
}
```


Then you must implement validate and apply function on your resourse with the following signature.
```rust
impl File {
    fn validate(&self) -> Result<()> {
        Ok(())
    }
    fn apply(&self) -> Result<()> {
       Ok(())
    }
}
```

When you compile the lib and use it with Caravel, the resulting .so,.dylib,.dll should be named the same as your resource.
Example for dummy library above: File.so

Then you can drop it in the caravel_modules directory of your Caravel Project!
