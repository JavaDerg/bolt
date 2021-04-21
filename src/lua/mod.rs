use mlua::Function;


pub fn load_fn(src: &'static str) -> Function<'static> {
    // SAFETY: There is none; TODO: Not leak memory?!
    let lua = Box::leak(Box::new(mlua::Lua::new()));
    lua.load(src.as_bytes()).eval().unwrap()
}