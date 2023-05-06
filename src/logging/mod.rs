pub fn init_library_logger(level: log::LevelFilter) {
  let mut builder = env_logger::Builder::from_default_env();
  builder.filter(None, level).init();
}
